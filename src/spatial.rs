// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::fmt;

use crate::error::RenderError;

/// Three-dimensional point in the DICOM patient Left-Posterior-Superior (`LPS`) millimetre frame.
#[derive(Clone, Copy, PartialEq)]
pub struct LpsPointMm {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl LpsPointMm {
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl fmt::Debug for LpsPointMm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LpsPointMm(<redacted>)")
    }
}

/// Continuous 2D image-plane coordinate `(col, row)` where `(0.0, 0.0)` is the top-left pixel centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelIndex2D {
    pub col: f64,
    pub row: f64,
}

impl PixelIndex2D {
    #[must_use]
    pub const fn new(col: f64, row: f64) -> Self {
        Self { col, row }
    }
}

/// Validated `f64` DICOM Image Plane (`PS3.3 C.7.6.2.1.1`) geometry in patient LPS millimetres.
#[derive(Clone, Copy, PartialEq)]
pub struct PatientPlane {
    origin_lps_mm: LpsPointMm,
    col_axis_cosine_r: [f64; 3],
    row_axis_cosine_c: [f64; 3],
    normal_cosine_n: [f64; 3],
    spacing_between_rows_mm: f64,
    spacing_between_cols_mm: f64,
    r_dot_r: f64,
    c_dot_c: f64,
    r_dot_c: f64,
    det_gram: f64,
}

impl fmt::Debug for PatientPlane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PatientPlane")
            .field("origin_lps_mm", &"<redacted>")
            .field("col_axis_cosine_r", &self.col_axis_cosine_r)
            .field("row_axis_cosine_c", &self.row_axis_cosine_c)
            .field("normal_cosine", &self.normal_cosine_n)
            .field("spacing_between_rows_mm", &self.spacing_between_rows_mm)
            .field("spacing_between_cols_mm", &self.spacing_between_cols_mm)
            .finish()
    }
}

impl PatientPlane {
    pub const COSINE_NORM_TOL: f64 = 1.0e-6;
    pub const ORTHOGONALITY_TOL: f64 = 1.0e-6;
    pub const MIN_SPACING_MM: f64 = 1.0e-6;

    pub fn new(
        origin_lps_mm: LpsPointMm,
        col_axis_cosine_r: [f64; 3],
        row_axis_cosine_c: [f64; 3],
        spacing_between_rows_mm: f64,
        spacing_between_cols_mm: f64,
    ) -> Result<Self, RenderError> {
        if !origin_lps_mm.x.is_finite()
            || !origin_lps_mm.y.is_finite()
            || !origin_lps_mm.z.is_finite()
            || !col_axis_cosine_r.iter().all(|v| v.is_finite())
            || !row_axis_cosine_c.iter().all(|v| v.is_finite())
            || !spacing_between_rows_mm.is_finite()
            || !spacing_between_cols_mm.is_finite()
        {
            return Err(RenderError::InvalidDescriptor(
                "non-finite patient plane coordinate, cosine, or spacing",
            ));
        }

        let r_dot_r = dot3(col_axis_cosine_r, col_axis_cosine_r);
        let c_dot_c = dot3(row_axis_cosine_c, row_axis_cosine_c);
        let r_dot_c = dot3(col_axis_cosine_r, row_axis_cosine_c);
        if !r_dot_r.is_finite() || !c_dot_c.is_finite() || !r_dot_c.is_finite() {
            return Err(RenderError::InvalidDescriptor(
                "patient plane direction cosine squared norm overflowed f64",
            ));
        }

        if spacing_between_rows_mm < Self::MIN_SPACING_MM
            || spacing_between_cols_mm < Self::MIN_SPACING_MM
        {
            return Err(RenderError::UnsupportedGeometry(
                "pixel spacing must be >= 1e-6 mm",
            ));
        }

        let r_norm = r_dot_r.sqrt();
        let c_norm = c_dot_c.sqrt();
        if (r_norm - 1.0).abs() > Self::COSINE_NORM_TOL
            || (c_norm - 1.0).abs() > Self::COSINE_NORM_TOL
        {
            return Err(RenderError::UnsupportedGeometry(
                "patient plane direction cosines must have unit norm within 1e-6",
            ));
        }

        if r_dot_c.abs() > Self::ORTHOGONALITY_TOL {
            return Err(RenderError::UnsupportedGeometry(
                "patient plane row and column direction cosines must be orthogonal within 1e-6",
            ));
        }

        let det_gram = r_dot_r * c_dot_c - r_dot_c * r_dot_c;
        let cross = cross3(col_axis_cosine_r, row_axis_cosine_c);
        let cross_norm_sq = dot3(cross, cross);
        let cross_norm = cross_norm_sq.sqrt();
        if !det_gram.is_finite()
            || det_gram <= 1.0e-12
            || !cross_norm_sq.is_finite()
            || !cross_norm.is_finite()
            || cross_norm <= 1.0e-12
        {
            return Err(RenderError::UnsupportedGeometry(
                "degenerate patient plane Gram determinant or normal vector",
            ));
        }

        let normal_cosine_n = [
            cross[0] / cross_norm,
            cross[1] / cross_norm,
            cross[2] / cross_norm,
        ];
        if !normal_cosine_n.iter().all(|v| v.is_finite()) {
            return Err(RenderError::UnsupportedGeometry(
                "non-finite normalized patient plane normal vector",
            ));
        }

        Ok(Self {
            origin_lps_mm,
            col_axis_cosine_r,
            row_axis_cosine_c,
            normal_cosine_n,
            spacing_between_rows_mm,
            spacing_between_cols_mm,
            r_dot_r,
            c_dot_c,
            r_dot_c,
            det_gram,
        })
    }

    /// Constructs a [`PatientPlane`] from DICOM `ImagePositionPatient`, `ImageOrientationPatient`
    /// (`[Rx, Ry, Rz, Cx, Cy, Cz]` where `R` steps along increasing `col` and `C` steps along
    /// increasing `row`), and `PixelSpacing` (`[row_spacing_dr, col_spacing_dc]`).
    pub fn from_dicom_image_plane(
        image_position_patient: [f64; 3],
        image_orientation_patient: [f64; 6],
        pixel_spacing_row_col: [f64; 2],
    ) -> Result<Self, RenderError> {
        let origin = LpsPointMm::new(
            image_position_patient[0],
            image_position_patient[1],
            image_position_patient[2],
        );
        let r = [
            image_orientation_patient[0],
            image_orientation_patient[1],
            image_orientation_patient[2],
        ];
        let c = [
            image_orientation_patient[3],
            image_orientation_patient[4],
            image_orientation_patient[5],
        ];
        Self::new(
            origin,
            r,
            c,
            pixel_spacing_row_col[0],
            pixel_spacing_row_col[1],
        )
    }

    #[must_use]
    pub const fn origin_lps_mm(&self) -> LpsPointMm {
        self.origin_lps_mm
    }

    #[must_use]
    pub const fn col_axis_cosine_r(&self) -> [f64; 3] {
        self.col_axis_cosine_r
    }

    #[must_use]
    pub const fn row_axis_cosine_c(&self) -> [f64; 3] {
        self.row_axis_cosine_c
    }

    #[must_use]
    pub const fn spacing_between_rows_mm(&self) -> f64 {
        self.spacing_between_rows_mm
    }

    #[must_use]
    pub const fn spacing_between_cols_mm(&self) -> f64 {
        self.spacing_between_cols_mm
    }

    #[must_use]
    pub const fn normal_cosine(&self) -> [f64; 3] {
        self.normal_cosine_n
    }

    pub fn pixel_to_patient(&self, idx: PixelIndex2D) -> Result<LpsPointMm, RenderError> {
        if !idx.col.is_finite() || !idx.row.is_finite() {
            return Err(RenderError::InvalidDescriptor(
                "non-finite PixelIndex2D coordinate",
            ));
        }
        let u_mm = idx.col * self.spacing_between_cols_mm;
        let v_mm = idx.row * self.spacing_between_rows_mm;
        let x = self.origin_lps_mm.x
            + u_mm * self.col_axis_cosine_r[0]
            + v_mm * self.row_axis_cosine_c[0];
        let y = self.origin_lps_mm.y
            + u_mm * self.col_axis_cosine_r[1]
            + v_mm * self.row_axis_cosine_c[1];
        let z = self.origin_lps_mm.z
            + u_mm * self.col_axis_cosine_r[2]
            + v_mm * self.row_axis_cosine_c[2];
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(RenderError::InvalidDescriptor(
                "projected patient LPS coordinate overflowed f64",
            ));
        }
        Ok(LpsPointMm { x, y, z })
    }

    pub fn patient_to_pixel(&self, point: LpsPointMm) -> Result<(PixelIndex2D, f64), RenderError> {
        if !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite() {
            return Err(RenderError::InvalidDescriptor(
                "non-finite LpsPointMm coordinate",
            ));
        }
        let delta = [
            point.x - self.origin_lps_mm.x,
            point.y - self.origin_lps_mm.y,
            point.z - self.origin_lps_mm.z,
        ];
        if !delta.iter().all(|v| v.is_finite()) {
            return Err(RenderError::InvalidDescriptor(
                "patient plane displacement vector overflowed f64",
            ));
        }
        let delta_dot_r = dot3(delta, self.col_axis_cosine_r);
        let delta_dot_c = dot3(delta, self.row_axis_cosine_c);
        let signed_normal_distance_mm = dot3(delta, self.normal_cosine_n);
        if !delta_dot_r.is_finite()
            || !delta_dot_c.is_finite()
            || !signed_normal_distance_mm.is_finite()
        {
            return Err(RenderError::InvalidDescriptor(
                "patient plane projection inner product overflowed f64",
            ));
        }

        let u_mm = (self.c_dot_c * delta_dot_r - self.r_dot_c * delta_dot_c) / self.det_gram;
        let v_mm = (self.r_dot_r * delta_dot_c - self.r_dot_c * delta_dot_r) / self.det_gram;
        let col = u_mm / self.spacing_between_cols_mm;
        let row = v_mm / self.spacing_between_rows_mm;
        if !col.is_finite() || !row.is_finite() {
            return Err(RenderError::InvalidDescriptor(
                "projected pixel index coordinate overflowed f64",
            ));
        }
        Ok((PixelIndex2D { col, row }, signed_normal_distance_mm))
    }
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
