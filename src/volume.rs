// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::fmt;

use crate::error::{LimitPoolKind, RenderError};
use crate::image::{BitPrecision, SampleType, SourceIdentity};
use crate::resource::ResourceLimits;
use crate::spatial::LpsPointMm;

/// Discrete 3D voxel grid dimensions `(cols, rows, slices)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VolumeDimensions {
    pub cols: u32,
    pub rows: u32,
    pub slices: u32,
}

impl VolumeDimensions {
    #[must_use]
    pub const fn new(cols: u32, rows: u32, slices: u32) -> Self {
        Self { cols, rows, slices }
    }
}

/// Orientation handedness of a regular 3D voxel-to-LPS basis in the right-handed LPS frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Handedness {
    RightHanded,
    LeftHanded,
}

/// Unvalidated regular 3D volume geometry and scalar metadata descriptor.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq)]
pub struct VolumeDescriptor {
    pub dimensions: VolumeDimensions,
    pub sample_type: SampleType,
    pub precision: BitPrecision,
    pub index_to_lps_affine: [[f64; 4]; 4],
    pub identity: SourceIdentity,
}

impl VolumeDescriptor {
    #[must_use]
    pub const fn new(
        dimensions: VolumeDimensions,
        sample_type: SampleType,
        precision: BitPrecision,
        index_to_lps_affine: [[f64; 4]; 4],
        identity: SourceIdentity,
    ) -> Self {
        Self {
            dimensions,
            sample_type,
            precision,
            index_to_lps_affine,
            identity,
        }
    }
}

impl fmt::Debug for VolumeDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = &self.index_to_lps_affine;
        let linear_3x3 = [
            [m[0][0], m[0][1], m[0][2]],
            [m[1][0], m[1][1], m[1][2]],
            [m[2][0], m[2][1], m[2][2]],
        ];
        f.debug_struct("VolumeDescriptor")
            .field("dimensions", &self.dimensions)
            .field("sample_type", &self.sample_type)
            .field("precision", &self.precision)
            .field("linear_3x3", &linear_3x3)
            .field("translation_lps_mm", &"<redacted>")
            .field("bottom_row", &m[3])
            .field("identity", &self.identity)
            .finish()
    }
}

/// Validated, constant-memory regular orthogonal 3D volume geometry with precomputed `f64` inverse.
#[derive(Clone, Copy, PartialEq)]
pub struct CheckedVolume {
    dimensions: VolumeDimensions,
    voxel_count: usize,
    byte_len: usize,
    sample_type: SampleType,
    precision: BitPrecision,
    axis_spacings_mm: [f64; 3],
    handedness: Handedness,
    index_to_lps_affine: [[f64; 4]; 4],
    lps_to_index_affine: [[f64; 4]; 4],
    identity: SourceIdentity,
}

impl fmt::Debug for CheckedVolume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = &self.index_to_lps_affine;
        let linear_3x3 = [
            [m[0][0], m[0][1], m[0][2]],
            [m[1][0], m[1][1], m[1][2]],
            [m[2][0], m[2][1], m[2][2]],
        ];
        f.debug_struct("CheckedVolume")
            .field("dimensions", &self.dimensions)
            .field("voxel_count", &self.voxel_count)
            .field("byte_len", &self.byte_len)
            .field("sample_type", &self.sample_type)
            .field("precision", &self.precision)
            .field("axis_spacings_mm", &self.axis_spacings_mm)
            .field("handedness", &self.handedness)
            .field("linear_3x3", &linear_3x3)
            .field("translation_lps_mm", &"<redacted>")
            .field("identity", &self.identity)
            .finish()
    }
}

impl CheckedVolume {
    #[must_use]
    pub const fn dimensions(&self) -> VolumeDimensions {
        self.dimensions
    }

    #[must_use]
    pub const fn voxel_count(&self) -> usize {
        self.voxel_count
    }

    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.byte_len
    }

    #[must_use]
    pub const fn sample_type(&self) -> SampleType {
        self.sample_type
    }

    #[must_use]
    pub const fn precision(&self) -> BitPrecision {
        self.precision
    }

    #[must_use]
    pub const fn axis_spacings_mm(&self) -> [f64; 3] {
        self.axis_spacings_mm
    }

    #[must_use]
    pub const fn handedness(&self) -> Handedness {
        self.handedness
    }

    #[must_use]
    pub const fn index_to_lps_affine(&self) -> &[[f64; 4]; 4] {
        &self.index_to_lps_affine
    }

    #[must_use]
    pub const fn lps_to_index_affine(&self) -> &[[f64; 4]; 4] {
        &self.lps_to_index_affine
    }

    #[must_use]
    pub const fn identity(&self) -> SourceIdentity {
        self.identity
    }

    pub fn voxel_to_patient(
        &self,
        col: f64,
        row: f64,
        slice: f64,
    ) -> Result<LpsPointMm, RenderError> {
        if !col.is_finite() || !row.is_finite() || !slice.is_finite() {
            return Err(RenderError::InvalidDescriptor(
                "non-finite voxel index coordinate",
            ));
        }
        let m = &self.index_to_lps_affine;
        let x = m[0][0] * col + m[0][1] * row + m[0][2] * slice + m[0][3];
        let y = m[1][0] * col + m[1][1] * row + m[1][2] * slice + m[1][3];
        let z = m[2][0] * col + m[2][1] * row + m[2][2] * slice + m[2][3];
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(RenderError::InvalidDescriptor(
                "voxel-to-LPS coordinate overflowed f64",
            ));
        }
        Ok(LpsPointMm::new(x, y, z))
    }

    pub fn patient_to_voxel(&self, point: LpsPointMm) -> Result<[f64; 3], RenderError> {
        if !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite() {
            return Err(RenderError::InvalidDescriptor(
                "non-finite LpsPointMm coordinate",
            ));
        }
        let inv = &self.lps_to_index_affine;
        let col = inv[0][0] * point.x + inv[0][1] * point.y + inv[0][2] * point.z + inv[0][3];
        let row = inv[1][0] * point.x + inv[1][1] * point.y + inv[1][2] * point.z + inv[1][3];
        let slice = inv[2][0] * point.x + inv[2][1] * point.y + inv[2][2] * point.z + inv[2][3];
        if !col.is_finite() || !row.is_finite() || !slice.is_finite() {
            return Err(RenderError::InvalidDescriptor(
                "LPS-to-voxel coordinate overflowed f64",
            ));
        }
        Ok([col, row, slice])
    }
}

/// Validates a regular orthogonal 3D [`VolumeDescriptor`] in constant CPU memory (`O(1)`) without
/// initializing any GPU device or allocating voxel storage.
pub fn admit_volume(
    descriptor: VolumeDescriptor,
    limits: &ResourceLimits,
) -> Result<CheckedVolume, RenderError> {
    const MIN_SPACING_MM: f64 = 1.0e-6;
    const ORTHOGONALITY_TOL: f64 = 1.0e-6;
    const BOTTOM_ROW_TOL: f64 = 1.0e-12;
    const MIN_NORMALIZED_DET: f64 = 1.0e-3;

    // 1. Validate sample type and canonical 16-bit precision.
    if !matches!(descriptor.sample_type, SampleType::I16 | SampleType::U16)
        || descriptor.precision.bits_allocated != 16
        || descriptor.precision.bits_stored == 0
        || descriptor.precision.bits_stored > 16
    {
        return Err(RenderError::UnsupportedPixelFormat {
            bits_allocated: descriptor.precision.bits_allocated,
            bits_stored: descriptor.precision.bits_stored,
            components: 1,
        });
    }
    if descriptor.precision.high_bit != descriptor.precision.bits_stored - 1 {
        return Err(RenderError::InvalidDescriptor(
            "volume high_bit must equal bits_stored - 1",
        ));
    }

    // 2. Validate dimensions and checked voxel/byte ceilings.
    let VolumeDimensions { cols, rows, slices } = descriptor.dimensions;
    if cols == 0 || rows == 0 || slices == 0 {
        return Err(RenderError::InvalidDescriptor(
            "volume cols, rows, and slices must all be >= 1",
        ));
    }
    let max_dim = limits.max_volume_dim as usize;
    for dim in [cols, rows, slices] {
        let dim_usize = usize::try_from(dim)
            .map_err(|_| RenderError::InvalidDescriptor("volume dimension exceeds host usize"))?;
        if dim > limits.max_volume_dim {
            return Err(RenderError::ResourceLimitExceeded {
                pool: LimitPoolKind::VolumeDimension,
                requested: dim_usize,
                available: max_dim,
                limit: max_dim,
            });
        }
    }

    let cols_usize = cols as usize;
    let rows_usize = rows as usize;
    let slices_usize = slices as usize;
    let voxel_count = cols_usize
        .checked_mul(rows_usize)
        .and_then(|v| v.checked_mul(slices_usize))
        .ok_or(RenderError::InvalidDescriptor(
            "volume voxel count overflow",
        ))?;
    let byte_len = voxel_count
        .checked_mul(2)
        .ok_or(RenderError::InvalidDescriptor(
            "volume byte length overflow",
        ))?;
    if byte_len > limits.max_volume_bytes {
        return Err(RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::VolumeBytes,
            requested: byte_len,
            available: limits.max_volume_bytes,
            limit: limits.max_volume_bytes,
        });
    }

    // 3. Verify finite 4x4 affine entries and homogeneous bottom row [0, 0, 0, 1].
    let m = descriptor.index_to_lps_affine;
    if !m.iter().flatten().all(|v| v.is_finite()) {
        return Err(RenderError::InvalidDescriptor(
            "volume index_to_lps_affine contains non-finite entries",
        ));
    }
    if m[3][0].abs() > BOTTOM_ROW_TOL
        || m[3][1].abs() > BOTTOM_ROW_TOL
        || m[3][2].abs() > BOTTOM_ROW_TOL
        || (m[3][3] - 1.0).abs() > BOTTOM_ROW_TOL
    {
        return Err(RenderError::InvalidDescriptor(
            "volume affine bottom row must equal [0, 0, 0, 1] within 1e-12",
        ));
    }

    // 4. Extract basis columns and require finite norms >= 1e-6 mm.
    let v0 = [m[0][0], m[1][0], m[2][0]];
    let v1 = [m[0][1], m[1][1], m[2][1]];
    let v2 = [m[0][2], m[1][2], m[2][2]];
    let dx_sq = dot3(v0, v0);
    let dy_sq = dot3(v1, v1);
    let dz_sq = dot3(v2, v2);
    let dx = dx_sq.sqrt();
    let dy = dy_sq.sqrt();
    let dz = dz_sq.sqrt();
    if !dx_sq.is_finite()
        || !dy_sq.is_finite()
        || !dz_sq.is_finite()
        || !dx.is_finite()
        || !dy.is_finite()
        || !dz.is_finite()
        || dx < MIN_SPACING_MM
        || dy < MIN_SPACING_MM
        || dz < MIN_SPACING_MM
    {
        return Err(RenderError::UnsupportedGeometry(
            "volume basis column norm is non-finite or < 1e-6 mm",
        ));
    }

    // 5. Normalize basis vectors and reject sheared / gantry-tilted non-orthogonal grids.
    let u0 = [v0[0] / dx, v0[1] / dx, v0[2] / dx];
    let u1 = [v1[0] / dy, v1[1] / dy, v1[2] / dy];
    let u2 = [v2[0] / dz, v2[1] / dz, v2[2] / dz];
    if !u0.iter().all(|v| v.is_finite())
        || !u1.iter().all(|v| v.is_finite())
        || !u2.iter().all(|v| v.is_finite())
    {
        return Err(RenderError::UnsupportedGeometry(
            "non-finite normalized volume basis vector",
        ));
    }
    let d01 = dot3(u0, u1);
    let d02 = dot3(u0, u2);
    let d12 = dot3(u1, u2);
    if !d01.is_finite()
        || !d02.is_finite()
        || !d12.is_finite()
        || d01.abs() > ORTHOGONALITY_TOL
        || d02.abs() > ORTHOGONALITY_TOL
        || d12.abs() > ORTHOGONALITY_TOL
    {
        return Err(RenderError::UnsupportedGeometry(
            "sheared or gantry-tilted non-orthogonal volume grid is not admitted in FS-01",
        ));
    }

    // 6. Conditioning check on 3x3 determinant and handedness classification.
    let c00 = m[1][1] * m[2][2] - m[1][2] * m[2][1];
    let c01 = m[1][2] * m[2][0] - m[1][0] * m[2][2];
    let c02 = m[1][0] * m[2][1] - m[1][1] * m[2][0];
    let det3 = m[0][0] * c00 + m[0][1] * c01 + m[0][2] * c02;
    let vol_scale = dx * dy * dz;
    if !det3.is_finite()
        || !vol_scale.is_finite()
        || vol_scale <= 0.0
        || (det3.abs() / vol_scale) < MIN_NORMALIZED_DET
    {
        return Err(RenderError::UnsupportedGeometry(
            "singular or ill-conditioned 3D volume affine matrix",
        ));
    }
    let handedness = if det3 > 0.0 {
        Handedness::RightHanded
    } else {
        Handedness::LeftHanded
    };

    // 7. Analytic f64 cofactor inverse of the 3x3 block and -M_inv * t translation column.
    let inv00 = c00 / det3;
    let inv01 = (m[0][2] * m[2][1] - m[0][1] * m[2][2]) / det3;
    let inv02 = (m[0][1] * m[1][2] - m[0][2] * m[1][1]) / det3;

    let inv10 = c01 / det3;
    let inv11 = (m[0][0] * m[2][2] - m[0][2] * m[2][0]) / det3;
    let inv12 = (m[0][2] * m[1][0] - m[0][0] * m[1][2]) / det3;

    let inv20 = c02 / det3;
    let inv21 = (m[0][1] * m[2][0] - m[0][0] * m[2][1]) / det3;
    let inv22 = (m[0][0] * m[1][1] - m[0][1] * m[1][0]) / det3;

    let tx = m[0][3];
    let ty = m[1][3];
    let tz = m[2][3];
    let it0 = -(inv00 * tx + inv01 * ty + inv02 * tz);
    let it1 = -(inv10 * tx + inv11 * ty + inv12 * tz);
    let it2 = -(inv20 * tx + inv21 * ty + inv22 * tz);

    let lps_to_index_affine = [
        [inv00, inv01, inv02, it0],
        [inv10, inv11, inv12, it1],
        [inv20, inv21, inv22, it2],
        [0.0, 0.0, 0.0, 1.0],
    ];
    if !lps_to_index_affine.iter().flatten().all(|v| v.is_finite()) {
        return Err(RenderError::UnsupportedGeometry(
            "inverted volume affine matrix overflowed f64",
        ));
    }

    Ok(CheckedVolume {
        dimensions: descriptor.dimensions,
        voxel_count,
        byte_len,
        sample_type: descriptor.sample_type,
        precision: descriptor.precision,
        axis_spacings_mm: [dx, dy, dz],
        handedness,
        index_to_lps_affine: m,
        lps_to_index_affine,
        identity: descriptor.identity,
    })
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
