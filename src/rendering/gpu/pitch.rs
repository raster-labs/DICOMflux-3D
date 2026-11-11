// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use crate::error::RenderError;
use crate::image::CheckedFrame;

const ROW_PITCH_ALIGNMENT: usize = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
const BUFFER_SIZE_ALIGNMENT: usize = wgpu::COPY_BUFFER_ALIGNMENT as usize;
const STACK_ROW_SCRATCH_BYTES: usize = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PaddedBufferLayout {
    pub width: u32,
    pub height: u32,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: usize,
    pub total_buffer_bytes: usize,
}

fn build_padded_layout(
    width: u32,
    height: u32,
    bytes_per_pixel: usize,
) -> Result<PaddedBufferLayout, RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::InvalidDescriptor(
            "GPU staging layout dimensions must be positive",
        ));
    }

    let w_usize = usize::try_from(width)
        .map_err(|_| RenderError::InvalidDescriptor("frame width exceeds host usize"))?;
    let h_usize = usize::try_from(height)
        .map_err(|_| RenderError::InvalidDescriptor("frame height exceeds host usize"))?;

    let unpadded = w_usize
        .checked_mul(bytes_per_pixel)
        .ok_or(RenderError::InvalidDescriptor(
            "unpadded GPU row byte width overflow",
        ))?;

    let row_rem = unpadded % ROW_PITCH_ALIGNMENT;
    let padded = if row_rem == 0 {
        unpadded
    } else {
        unpadded.checked_add(ROW_PITCH_ALIGNMENT - row_rem).ok_or(
            RenderError::InvalidDescriptor("256-byte padded row stride overflow"),
        )?
    };

    let _ = u32::try_from(padded).map_err(|_| {
        RenderError::InvalidDescriptor("256-byte padded row stride exceeds u32 copy limit")
    })?;

    let raw_total = padded
        .checked_mul(h_usize)
        .ok_or(RenderError::InvalidDescriptor(
            "GPU staging buffer byte size overflow",
        ))?;

    let buf_rem = raw_total % BUFFER_SIZE_ALIGNMENT;
    let total = if buf_rem == 0 {
        raw_total
    } else {
        raw_total
            .checked_add(BUFFER_SIZE_ALIGNMENT - buf_rem)
            .ok_or(RenderError::InvalidDescriptor(
                "4-byte aligned GPU buffer byte size overflow",
            ))?
    };

    let _ = u64::try_from(total).map_err(|_| {
        RenderError::InvalidDescriptor("GPU staging buffer size exceeds u64 BufferAddress")
    })?;

    Ok(PaddedBufferLayout {
        width,
        height,
        unpadded_bytes_per_row: unpadded,
        padded_bytes_per_row: padded,
        total_buffer_bytes: total,
    })
}

pub(crate) fn checked_upload_layout(
    width: u32,
    height: u32,
) -> Result<PaddedBufferLayout, RenderError> {
    build_padded_layout(width, height, 2)
}

pub(crate) fn checked_readback_layout(
    width: u32,
    height: u32,
) -> Result<PaddedBufferLayout, RenderError> {
    build_padded_layout(width, height, 1)
}

/// Writes `frame` rows in canonical little-endian byte order with zero-filled 256-byte row tails
/// directly into a `mapped_at_creation` `wgpu::BufferViewMut` without allocating a full-frame
/// temporary CPU heap copy.
pub(crate) fn write_frame_to_upload_staging(
    frame: &CheckedFrame,
    layout: &PaddedBufferLayout,
    view: &mut wgpu::BufferViewMut,
) {
    debug_assert_eq!(frame.width(), layout.width);
    debug_assert_eq!(frame.height(), layout.height);
    debug_assert_eq!(view.len(), layout.total_buffer_bytes);

    let unpadded = layout.unpadded_bytes_per_row;
    let padded = layout.padded_bytes_per_row;

    if padded <= STACK_ROW_SCRATCH_BYTES {
        let mut row_scratch = [0u8; STACK_ROW_SCRATCH_BYTES];
        for row in 0..layout.height {
            let row_start = (row as usize) * padded;
            frame.write_row_le_bytes(row, &mut row_scratch[..unpadded]);
            row_scratch[unpadded..padded].fill(0);
            view.slice(row_start..row_start + padded)
                .copy_from_slice(&row_scratch[..padded]);
        }
    } else {
        let mut row_scratch = vec![0u8; padded];
        for row in 0..layout.height {
            let row_start = (row as usize) * padded;
            frame.write_row_le_bytes(row, &mut row_scratch[..unpadded]);
            row_scratch[unpadded..padded].fill(0);
            view.slice(row_start..row_start + padded)
                .copy_from_slice(&row_scratch);
        }
    }

    let rows_end = (layout.height as usize) * padded;
    if rows_end < layout.total_buffer_bytes {
        let zero_tail = [0u8; BUFFER_SIZE_ALIGNMENT];
        let rem = layout.total_buffer_bytes - rows_end;
        view.slice(rows_end..layout.total_buffer_bytes)
            .copy_from_slice(&zero_tail[..rem]);
    }
}

/// Unpacks 256-byte row-aligned `R8Uint` readback bytes into a tight `width * height` `Vec<u8>`.
pub(crate) fn unpack_readback_to_tight(
    mapped: &[u8],
    layout: &PaddedBufferLayout,
    dst: &mut Vec<u8>,
) -> Result<(), RenderError> {
    if mapped.len() < layout.total_buffer_bytes {
        return Err(RenderError::InternalError(
            "mapped readback buffer shorter than expected padded layout",
        ));
    }

    dst.clear();
    for row in 0..(layout.height as usize) {
        let start = row * layout.padded_bytes_per_row;
        let end = start + layout.unpadded_bytes_per_row;
        dst.extend_from_slice(&mapped[start..end]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_and_prime_dimensions_align_to_256_bytes_per_row() {
        let up_4x4 = checked_upload_layout(4, 4).expect("4x4 upload layout");
        assert_eq!(up_4x4.unpadded_bytes_per_row, 8);
        assert_eq!(up_4x4.padded_bytes_per_row, 256);
        assert_eq!(up_4x4.total_buffer_bytes, 1024);

        let rb_63x37 = checked_readback_layout(63, 37).expect("63x37 readback layout");
        assert_eq!(rb_63x37.unpadded_bytes_per_row, 63);
        assert_eq!(rb_63x37.padded_bytes_per_row, 256);
        assert_eq!(rb_63x37.total_buffer_bytes, 256 * 37);

        let up_256 = checked_upload_layout(128, 2).expect("exact 256B upload row");
        assert_eq!(up_256.unpadded_bytes_per_row, 256);
        assert_eq!(up_256.padded_bytes_per_row, 256);
        assert_eq!(up_256.total_buffer_bytes, 512);
    }

    #[test]
    fn zero_and_overflowing_dimensions_reject_cleanly() {
        assert!(matches!(
            checked_upload_layout(0, 16),
            Err(RenderError::InvalidDescriptor(_))
        ));
        assert!(matches!(
            checked_readback_layout(16, 0),
            Err(RenderError::InvalidDescriptor(_))
        ));
        assert!(matches!(
            checked_upload_layout(u32::MAX, u32::MAX),
            Err(RenderError::InvalidDescriptor(_))
        ));
    }

    #[test]
    fn unpack_readback_strips_row_padding() {
        let layout = checked_readback_layout(3, 2).expect("3x2 readback layout");
        let mut padded = vec![0xAAu8; layout.total_buffer_bytes];
        padded[0..3].copy_from_slice(&[10, 20, 30]);
        padded[256..259].copy_from_slice(&[40, 50, 60]);

        let mut tight = Vec::with_capacity(6);
        unpack_readback_to_tight(&padded, &layout, &mut tight).expect("unpack succeeds");
        assert_eq!(tight, vec![10, 20, 30, 40, 50, 60]);
    }
}
