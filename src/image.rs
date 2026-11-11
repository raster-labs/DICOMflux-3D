// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::fmt;
use std::num::NonZeroU64;
use std::sync::Arc;

use crate::error::{LimitPoolKind, RenderError};
use crate::resource::{BudgetGuard, ResourceTracker};

/// Privacy-safe numeric token identifying an upstream pixel buffer owner and generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceIdentity {
    pub owner_id: NonZeroU64,
    pub generation: u64,
}

impl SourceIdentity {
    #[must_use]
    pub const fn new(owner_id: NonZeroU64, generation: u64) -> Self {
        Self {
            owner_id,
            generation,
        }
    }
}

struct SharedSliceOwner<B: ?Sized>(Arc<B>);

impl<B: ?Sized + AsRef<[u8]>> AsRef<[u8]> for SharedSliceOwner<B> {
    fn as_ref(&self) -> &[u8] {
        (*self.0).as_ref()
    }
}

impl<B: ?Sized + AsRef<[i16]>> AsRef<[i16]> for SharedSliceOwner<B> {
    fn as_ref(&self) -> &[i16] {
        (*self.0).as_ref()
    }
}

impl<B: ?Sized + AsRef<[u16]>> AsRef<[u16]> for SharedSliceOwner<B> {
    fn as_ref(&self) -> &[u16] {
        (*self.0).as_ref()
    }
}

#[derive(Clone)]
enum FrameBacking {
    Bytes(Arc<dyn AsRef<[u8]> + Send + Sync>),
    WordsI16(Arc<dyn AsRef<[i16]> + Send + Sync>),
    WordsU16(Arc<dyn AsRef<[u16]> + Send + Sync>),
}

impl FrameBacking {
    const fn kind_label(&self) -> &'static str {
        match self {
            Self::Bytes(_) => "Bytes",
            Self::WordsI16(_) => "WordsI16",
            Self::WordsU16(_) => "WordsU16",
        }
    }
}

/// Reference-counted immutable ownership lease over a decoded pixel buffer.
#[derive(Clone)]
pub struct DecodedFrameLease {
    backing: FrameBacking,
    logical_byte_len: usize,
    retained_capacity_bytes: usize,
    identity: SourceIdentity,
}

impl fmt::Debug for DecodedFrameLease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecodedFrameLease")
            .field("identity", &self.identity)
            .field("backing_kind", &self.backing.kind_label())
            .field("logical_byte_len", &self.logical_byte_len)
            .field("retained_capacity_bytes", &self.retained_capacity_bytes)
            .field("payload", &"<redacted>")
            .finish()
    }
}

impl DecodedFrameLease {
    #[must_use]
    pub fn from_arc_bytes<B: ?Sized + AsRef<[u8]> + Send + Sync + 'static>(
        backing: Arc<B>,
        identity: SourceIdentity,
    ) -> Self {
        let logical_byte_len = (*backing).as_ref().len();
        Self {
            backing: FrameBacking::Bytes(Arc::new(SharedSliceOwner(backing))),
            logical_byte_len,
            retained_capacity_bytes: logical_byte_len,
            identity,
        }
    }

    pub fn from_arc_i16<B: ?Sized + AsRef<[i16]> + Send + Sync + 'static>(
        backing: Arc<B>,
        identity: SourceIdentity,
    ) -> Result<Self, RenderError> {
        let logical_byte_len =
            (*backing)
                .as_ref()
                .len()
                .checked_mul(2)
                .ok_or(RenderError::InvalidDescriptor(
                    "i16 slice logical byte length overflow",
                ))?;
        Ok(Self {
            backing: FrameBacking::WordsI16(Arc::new(SharedSliceOwner(backing))),
            logical_byte_len,
            retained_capacity_bytes: logical_byte_len,
            identity,
        })
    }

    pub fn from_arc_u16<B: ?Sized + AsRef<[u16]> + Send + Sync + 'static>(
        backing: Arc<B>,
        identity: SourceIdentity,
    ) -> Result<Self, RenderError> {
        let logical_byte_len =
            (*backing)
                .as_ref()
                .len()
                .checked_mul(2)
                .ok_or(RenderError::InvalidDescriptor(
                    "u16 slice logical byte length overflow",
                ))?;
        Ok(Self {
            backing: FrameBacking::WordsU16(Arc::new(SharedSliceOwner(backing))),
            logical_byte_len,
            retained_capacity_bytes: logical_byte_len,
            identity,
        })
    }

    pub fn from_arc_bytes_with_capacity<B: ?Sized + AsRef<[u8]> + Send + Sync + 'static>(
        backing: Arc<B>,
        retained_capacity_bytes: usize,
        identity: SourceIdentity,
    ) -> Result<Self, RenderError> {
        let logical_byte_len = (*backing).as_ref().len();
        if retained_capacity_bytes < logical_byte_len {
            return Err(RenderError::InvalidDescriptor(
                "retained_capacity_bytes is smaller than logical_byte_len",
            ));
        }
        Ok(Self {
            backing: FrameBacking::Bytes(Arc::new(SharedSliceOwner(backing))),
            logical_byte_len,
            retained_capacity_bytes,
            identity,
        })
    }

    pub fn from_arc_i16_with_capacity<B: ?Sized + AsRef<[i16]> + Send + Sync + 'static>(
        backing: Arc<B>,
        retained_capacity_bytes: usize,
        identity: SourceIdentity,
    ) -> Result<Self, RenderError> {
        let logical_byte_len =
            (*backing)
                .as_ref()
                .len()
                .checked_mul(2)
                .ok_or(RenderError::InvalidDescriptor(
                    "i16 slice logical byte length overflow",
                ))?;
        if retained_capacity_bytes < logical_byte_len {
            return Err(RenderError::InvalidDescriptor(
                "retained_capacity_bytes is smaller than logical_byte_len",
            ));
        }
        Ok(Self {
            backing: FrameBacking::WordsI16(Arc::new(SharedSliceOwner(backing))),
            logical_byte_len,
            retained_capacity_bytes,
            identity,
        })
    }

    pub fn from_arc_u16_with_capacity<B: ?Sized + AsRef<[u16]> + Send + Sync + 'static>(
        backing: Arc<B>,
        retained_capacity_bytes: usize,
        identity: SourceIdentity,
    ) -> Result<Self, RenderError> {
        let logical_byte_len =
            (*backing)
                .as_ref()
                .len()
                .checked_mul(2)
                .ok_or(RenderError::InvalidDescriptor(
                    "u16 slice logical byte length overflow",
                ))?;
        if retained_capacity_bytes < logical_byte_len {
            return Err(RenderError::InvalidDescriptor(
                "retained_capacity_bytes is smaller than logical_byte_len",
            ));
        }
        Ok(Self {
            backing: FrameBacking::WordsU16(Arc::new(SharedSliceOwner(backing))),
            logical_byte_len,
            retained_capacity_bytes,
            identity,
        })
    }

    #[must_use]
    pub fn from_vec_u8(bytes: Vec<u8>, identity: SourceIdentity) -> Self {
        let logical_byte_len = bytes.len();
        let retained_capacity_bytes = bytes.capacity();
        Self {
            backing: FrameBacking::Bytes(Arc::new(bytes)),
            logical_byte_len,
            retained_capacity_bytes,
            identity,
        }
    }

    pub fn from_vec_i16(words: Vec<i16>, identity: SourceIdentity) -> Result<Self, RenderError> {
        let logical_byte_len = words
            .len()
            .checked_mul(2)
            .ok_or(RenderError::InvalidDescriptor(
                "i16 vector logical byte length overflow",
            ))?;
        let retained_capacity_bytes =
            words
                .capacity()
                .checked_mul(2)
                .ok_or(RenderError::InvalidDescriptor(
                    "i16 vector capacity byte length overflow",
                ))?;
        Ok(Self {
            backing: FrameBacking::WordsI16(Arc::new(words)),
            logical_byte_len,
            retained_capacity_bytes,
            identity,
        })
    }

    pub fn from_vec_u16(words: Vec<u16>, identity: SourceIdentity) -> Result<Self, RenderError> {
        let logical_byte_len = words
            .len()
            .checked_mul(2)
            .ok_or(RenderError::InvalidDescriptor(
                "u16 vector logical byte length overflow",
            ))?;
        let retained_capacity_bytes =
            words
                .capacity()
                .checked_mul(2)
                .ok_or(RenderError::InvalidDescriptor(
                    "u16 vector capacity byte length overflow",
                ))?;
        Ok(Self {
            backing: FrameBacking::WordsU16(Arc::new(words)),
            logical_byte_len,
            retained_capacity_bytes,
            identity,
        })
    }

    #[must_use]
    pub const fn identity(&self) -> SourceIdentity {
        self.identity
    }

    #[must_use]
    pub const fn logical_byte_len(&self) -> usize {
        self.logical_byte_len
    }

    #[must_use]
    pub const fn retained_capacity_bytes(&self) -> usize {
        self.retained_capacity_bytes
    }

    fn raw_u16_at_even_byte(&self, byte_idx: usize) -> u16 {
        debug_assert_eq!(byte_idx & 1, 0);
        match &self.backing {
            FrameBacking::Bytes(b) => {
                let bytes = (**b).as_ref();
                u16::from_le_bytes([bytes[byte_idx], bytes[byte_idx + 1]])
            }
            FrameBacking::WordsI16(w) => {
                let words = (**w).as_ref();
                words[byte_idx / 2] as u16
            }
            FrameBacking::WordsU16(w) => {
                let words = (**w).as_ref();
                words[byte_idx / 2]
            }
        }
    }
}

/// Scalar sample representation declared by a frame or volume descriptor.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SampleType {
    I16,
    U16,
    U8,
    I32,
    F32,
}

/// High-level colour model declared by a frame descriptor.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorModel {
    Grayscale,
    PaletteIndexed,
    Color,
}

/// Stored-container pixel padding specification evaluated prior to modality rescale.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelPadding {
    None,
    Single(i32),
    Range { start: i32, end: i32 },
}

impl PixelPadding {
    #[must_use]
    pub const fn matches(self, sample: i32) -> bool {
        match self {
            Self::None => false,
            Self::Single(v) => sample == v,
            Self::Range { start, end } => sample >= start && sample <= end,
        }
    }
}

/// Canonical DICOM bit-allocation and precision metadata (`BitsAllocated`, `BitsStored`, `HighBit`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitPrecision {
    pub bits_allocated: u8,
    pub bits_stored: u8,
    pub high_bit: u8,
}

impl BitPrecision {
    #[must_use]
    pub const fn new(bits_allocated: u8, bits_stored: u8, high_bit: u8) -> Self {
        Self {
            bits_allocated,
            bits_stored,
            high_bit,
        }
    }
}

/// Layout and bit-precision descriptor for a decoded pixel frame.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameDescriptor {
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
    pub byte_offset: usize,
    pub row_stride_bytes: usize,
    pub sample_type: SampleType,
    pub precision: BitPrecision,
    pub components: u8,
    pub color_model: ColorModel,
    pub pixel_padding: PixelPadding,
}

impl FrameDescriptor {
    #[must_use]
    pub const fn new(
        width: u32,
        height: u32,
        row_stride_bytes: usize,
        sample_type: SampleType,
        precision: BitPrecision,
    ) -> Self {
        Self {
            width,
            height,
            frame_count: 1,
            byte_offset: 0,
            row_stride_bytes,
            sample_type,
            precision,
            components: 1,
            color_model: ColorModel::Grayscale,
            pixel_padding: PixelPadding::None,
        }
    }
}

/// Validated, immutable 16-bit single-frame greyscale input admitted under a [`ResourceTracker`].
#[derive(Clone)]
pub struct CheckedFrame {
    descriptor: FrameDescriptor,
    lease: DecodedFrameLease,
    #[allow(dead_code)]
    source_budget: Arc<BudgetGuard>,
}

impl fmt::Debug for CheckedFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CheckedFrame")
            .field("descriptor", &self.descriptor)
            .field("identity", &self.lease.identity())
            .field("logical_byte_len", &self.lease.logical_byte_len())
            .field(
                "retained_capacity_bytes",
                &self.lease.retained_capacity_bytes(),
            )
            .field("pixels", &"<redacted>")
            .finish()
    }
}

impl CheckedFrame {
    #[must_use]
    pub const fn descriptor(&self) -> &FrameDescriptor {
        &self.descriptor
    }

    #[must_use]
    pub const fn identity(&self) -> SourceIdentity {
        self.lease.identity()
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.descriptor.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.descriptor.height
    }

    #[must_use]
    pub const fn sample_type(&self) -> SampleType {
        self.descriptor.sample_type
    }

    #[must_use]
    pub const fn precision(&self) -> BitPrecision {
        self.descriptor.precision
    }

    #[must_use]
    pub const fn pixel_padding(&self) -> PixelPadding {
        self.descriptor.pixel_padding
    }

    #[must_use]
    pub fn sample_i32(&self, col: u32, row: u32) -> i32 {
        assert!(
            col < self.descriptor.width && row < self.descriptor.height,
            "sample coordinates out of frame bounds"
        );
        let byte_idx = self.descriptor.byte_offset
            + (row as usize) * self.descriptor.row_stride_bytes
            + (col as usize) * 2;
        let raw = self.lease.raw_u16_at_even_byte(byte_idx);
        match self.descriptor.sample_type {
            SampleType::I16 => i32::from(raw as i16),
            SampleType::U16 => i32::from(raw),
            _ => unreachable!("CheckedFrame admits only I16 and U16 sample types"),
        }
    }

    #[allow(dead_code)]
    pub(crate) const fn lease(&self) -> &DecodedFrameLease {
        &self.lease
    }

    #[allow(dead_code)]
    pub(crate) const fn source_budget(&self) -> &Arc<BudgetGuard> {
        &self.source_budget
    }

    #[allow(dead_code)]
    pub(crate) fn write_row_le_bytes(&self, row: u32, dst: &mut [u8]) {
        assert!(
            row < self.descriptor.height,
            "row index out of frame bounds"
        );
        let width_words = self.descriptor.width as usize;
        let row_byte_len = width_words * 2;
        assert!(
            dst.len() >= row_byte_len,
            "destination slice shorter than active row byte width"
        );
        let row_byte_start =
            self.descriptor.byte_offset + (row as usize) * self.descriptor.row_stride_bytes;

        match &self.lease.backing {
            FrameBacking::Bytes(b) => {
                let bytes = (**b).as_ref();
                dst[..row_byte_len]
                    .copy_from_slice(&bytes[row_byte_start..row_byte_start + row_byte_len]);
            }
            FrameBacking::WordsI16(w) => {
                let words = (**w).as_ref();
                let word_start = row_byte_start / 2;
                for (chunk, &word) in dst[..row_byte_len]
                    .chunks_exact_mut(2)
                    .zip(&words[word_start..word_start + width_words])
                {
                    chunk.copy_from_slice(&word.to_le_bytes());
                }
            }
            FrameBacking::WordsU16(w) => {
                let words = (**w).as_ref();
                let word_start = row_byte_start / 2;
                for (chunk, &word) in dst[..row_byte_len]
                    .chunks_exact_mut(2)
                    .zip(&words[word_start..word_start + width_words])
                {
                    chunk.copy_from_slice(&word.to_le_bytes());
                }
            }
        }
    }
}

/// Validates a 16-bit single-frame greyscale [`FrameDescriptor`] and [`DecodedFrameLease`] against
/// canonical layout rules and [`ResourceTracker`] budgets without mutating source bytes.
pub fn admit_frame(
    descriptor: FrameDescriptor,
    lease: DecodedFrameLease,
    tracker: &ResourceTracker,
) -> Result<CheckedFrame, RenderError> {
    // 1. Frame count, colour model, components, sample type, and 16-bit allocation checks.
    if descriptor.frame_count == 0 {
        return Err(RenderError::InvalidDescriptor("frame_count must be >= 1"));
    }
    if descriptor.color_model != ColorModel::Grayscale {
        return Err(RenderError::UnsupportedPresentation(
            "colour and palette-indexed models are not admitted in FS-01",
        ));
    }
    if descriptor.frame_count > 1
        || descriptor.components != 1
        || !matches!(descriptor.sample_type, SampleType::I16 | SampleType::U16)
        || descriptor.precision.bits_allocated != 16
    {
        return Err(RenderError::UnsupportedPixelFormat {
            bits_allocated: descriptor.precision.bits_allocated,
            bits_stored: descriptor.precision.bits_stored,
            components: descriptor.components,
        });
    }

    // 2. Canonical precision checks (`1 <= bits_stored <= 16` and `high_bit == bits_stored - 1`).
    if descriptor.precision.bits_stored == 0 || descriptor.precision.bits_stored > 16 {
        return Err(RenderError::UnsupportedPixelFormat {
            bits_allocated: descriptor.precision.bits_allocated,
            bits_stored: descriptor.precision.bits_stored,
            components: descriptor.components,
        });
    }
    if descriptor.precision.high_bit != descriptor.precision.bits_stored - 1 {
        return Err(RenderError::InvalidDescriptor(
            "high_bit must equal bits_stored - 1 for canonical 16-bit storage",
        ));
    }

    // 3. Validate pixel_padding against the 16-bit container domain.
    let (container_min, container_max) = match descriptor.sample_type {
        SampleType::I16 => (i32::from(i16::MIN), i32::from(i16::MAX)),
        SampleType::U16 => (i32::from(u16::MIN), i32::from(u16::MAX)),
        _ => unreachable!(),
    };
    match descriptor.pixel_padding {
        PixelPadding::None => {}
        PixelPadding::Single(val) => {
            if !(container_min..=container_max).contains(&val) {
                return Err(RenderError::InvalidDescriptor(
                    "pixel padding value lies outside 16-bit container domain",
                ));
            }
        }
        PixelPadding::Range { start, end } => {
            if start > end
                || !(container_min..=container_max).contains(&start)
                || !(container_min..=container_max).contains(&end)
            {
                return Err(RenderError::InvalidDescriptor(
                    "pixel padding range is inverted or outside 16-bit container domain",
                ));
            }
        }
    }

    // 4. Non-zero dimension check and FrameDimension ceiling check.
    if descriptor.width == 0 || descriptor.height == 0 {
        return Err(RenderError::InvalidDescriptor(
            "frame width and height must be >= 1",
        ));
    }
    let width_usize = usize::try_from(descriptor.width)
        .map_err(|_| RenderError::InvalidDescriptor("frame width exceeds host usize"))?;
    let height_usize = usize::try_from(descriptor.height)
        .map_err(|_| RenderError::InvalidDescriptor("frame height exceeds host usize"))?;
    let _ = tracker.reserve(LimitPoolKind::FrameDimension, width_usize)?;
    let _ = tracker.reserve(LimitPoolKind::FrameDimension, height_usize)?;

    // 5. Alignment, stride, and F0 checked plane-extent validation.
    if (descriptor.byte_offset & 1) != 0 || (descriptor.row_stride_bytes & 1) != 0 {
        return Err(RenderError::InvalidDescriptor(
            "16-bit frame byte_offset and row_stride_bytes must be even",
        ));
    }
    let active_row_bytes = width_usize
        .checked_mul(2)
        .ok_or(RenderError::InvalidDescriptor(
            "frame row byte width overflow",
        ))?;
    if descriptor.row_stride_bytes < active_row_bytes {
        return Err(RenderError::InvalidDescriptor(
            "row_stride_bytes is smaller than width * 2",
        ));
    }
    let last_row_offset = (height_usize - 1)
        .checked_mul(descriptor.row_stride_bytes)
        .ok_or(RenderError::InvalidDescriptor(
            "frame plane stride extent overflow",
        ))?;
    let last_sample_offset = (width_usize - 1)
        .checked_mul(2)
        .and_then(|v| v.checked_add(2))
        .ok_or(RenderError::InvalidDescriptor(
            "frame final sample offset overflow",
        ))?;
    let plane_end_byte = descriptor
        .byte_offset
        .checked_add(last_row_offset)
        .and_then(|v| v.checked_add(last_sample_offset))
        .ok_or(RenderError::InvalidDescriptor(
            "frame plane end byte offset overflow",
        ))?;

    if lease.logical_byte_len() > lease.retained_capacity_bytes()
        || plane_end_byte > lease.logical_byte_len()
    {
        return Err(RenderError::InvalidDescriptor(
            "frame plane extent exceeds lease byte length",
        ));
    }

    // 6. Pre-scan RetainedSource budget reservation.
    let source_guard = tracker.reserve(
        LimitPoolKind::RetainedSource,
        lease.retained_capacity_bytes(),
    )?;

    // 7. Sub-16-bit canonical upper-bit verification (exempting declared container padding).
    let bits_stored = descriptor.precision.bits_stored;
    if bits_stored < 16 {
        let shift = 16 - bits_stored;
        for row in 0..height_usize {
            let row_base = descriptor.byte_offset + row * descriptor.row_stride_bytes;
            for col in 0..width_usize {
                let raw_u16 = lease.raw_u16_at_even_byte(row_base + col * 2);
                match descriptor.sample_type {
                    SampleType::I16 => {
                        let raw_i16 = raw_u16 as i16;
                        let s = i32::from(raw_i16);
                        if !descriptor.pixel_padding.matches(s)
                            && ((raw_i16 << shift) >> shift) != raw_i16
                        {
                            return Err(RenderError::InvalidDescriptor(
                                "signed sub-16-bit sample has non-sign-extended upper bits",
                            ));
                        }
                    }
                    SampleType::U16 => {
                        let s = i32::from(raw_u16);
                        if !descriptor.pixel_padding.matches(s) && (raw_u16 >> bits_stored) != 0 {
                            return Err(RenderError::InvalidDescriptor(
                                "unsigned sub-16-bit sample has non-zero upper bits",
                            ));
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    Ok(CheckedFrame {
        descriptor,
        lease,
        source_budget: Arc::new(source_guard),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_row_le_bytes_matches_across_all_backings() {
        let tracker = ResourceTracker::with_default_limits();
        let id = SourceIdentity::new(NonZeroU64::new(1).unwrap(), 1);
        let prec = BitPrecision::new(16, 16, 15);
        let desc = FrameDescriptor::new(2, 2, 4, SampleType::I16, prec);

        let i16_words: Arc<[i16]> = Arc::from([-2048_i16, 2047, -1, 0].as_slice());
        let f_i16 = admit_frame(
            desc,
            DecodedFrameLease::from_arc_i16(i16_words, id).unwrap(),
            &tracker,
        )
        .unwrap();
        assert_eq!(f_i16.lease().logical_byte_len(), 8);
        assert_eq!(f_i16.source_budget().amount(), 8);

        let mut row0 = [0_u8; 4];
        let mut row1 = [0_u8; 4];
        f_i16.write_row_le_bytes(0, &mut row0);
        f_i16.write_row_le_bytes(1, &mut row1);

        let mut expected_row0 = [0_u8; 4];
        expected_row0[0..2].copy_from_slice(&(-2048_i16).to_le_bytes());
        expected_row0[2..4].copy_from_slice(&2047_i16.to_le_bytes());
        assert_eq!(row0, expected_row0);

        let raw_bytes: Arc<[u8]> = Arc::from([row0, row1].concat().into_boxed_slice());
        let f_bytes = admit_frame(
            desc,
            DecodedFrameLease::from_arc_bytes(raw_bytes, id),
            &tracker,
        )
        .unwrap();
        let mut row0_b = [0_u8; 4];
        f_bytes.write_row_le_bytes(0, &mut row0_b);
        assert_eq!(row0_b, expected_row0);

        let u16_words: Arc<[u16]> = Arc::from([10_u16, 50000, 1, 65535].as_slice());
        let f_u16 = admit_frame(
            FrameDescriptor::new(2, 2, 4, SampleType::U16, prec),
            DecodedFrameLease::from_arc_u16(u16_words, id).unwrap(),
            &tracker,
        )
        .unwrap();
        let mut row0_u = [0_u8; 4];
        f_u16.write_row_le_bytes(0, &mut row0_u);
        assert_eq!(&row0_u[0..2], &10_u16.to_le_bytes());
        assert_eq!(&row0_u[2..4], &50000_u16.to_le_bytes());
    }
}
