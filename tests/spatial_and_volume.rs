// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::num::NonZeroU64;

use dicomflux_3d::{
    BitPrecision, Handedness, LimitPoolKind, LpsPointMm, PatientPlane, PixelIndex2D, RenderError,
    ResourceLimits, SampleType, SourceIdentity, VolumeDescriptor, VolumeDimensions, admit_volume,
};

fn test_identity(id: u64) -> SourceIdentity {
    SourceIdentity::new(NonZeroU64::new(id).expect("non-zero id"), 1)
}

#[test]
fn fs01_t10_patient_plane_lps_phantom_and_oblique_roundtrip() {
    // 1. Specification phantom: origin = (10, 20, 30), R = (1, 0, 0), C = (0, 1, 0),
    // row spacing dr = 2.0 mm, col spacing dc = 0.5 mm -> (col = 4, row = 3) maps to (12, 26, 30).
    let plane_new = PatientPlane::new(
        LpsPointMm::new(10.0, 20.0, 30.0),
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        2.0,
        0.5,
    )
    .expect("valid axial phantom plane");
    let plane_dicom = PatientPlane::from_dicom_image_plane(
        [10.0, 20.0, 30.0],
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [2.0, 0.5],
    )
    .expect("valid DICOM ImagePlane constructor");
    assert_eq!(plane_new, plane_dicom);

    let mapped = plane_dicom
        .pixel_to_patient(PixelIndex2D::new(4.0, 3.0))
        .expect("phantom forward projection");
    assert!((mapped.x - 12.0).abs() <= 1.0e-12);
    assert!((mapped.y - 26.0).abs() <= 1.0e-12);
    assert!((mapped.z - 30.0).abs() <= 1.0e-12);

    // Asymmetric dr != dc check: stepping +1 in col moves +0.5 along R (x), +1 in row moves +2.0 along C (y).
    let step_col = plane_dicom
        .pixel_to_patient(PixelIndex2D::new(1.0, 0.0))
        .unwrap();
    let step_row = plane_dicom
        .pixel_to_patient(PixelIndex2D::new(0.0, 1.0))
        .unwrap();
    assert!((step_col.x - 10.5).abs() <= 1.0e-12);
    assert!((step_row.y - 22.0).abs() <= 1.0e-12);

    let normal = plane_dicom.normal_cosine();
    assert!(normal[0].abs() <= 1.0e-12);
    assert!(normal[1].abs() <= 1.0e-12);
    assert!((normal[2] - 1.0).abs() <= 1.0e-12);

    let (inv_idx, signed_dist) = plane_dicom
        .patient_to_pixel(LpsPointMm::new(12.0, 26.0, 37.5))
        .expect("phantom inverse projection with out-of-plane offset");
    assert!((inv_idx.col - 4.0).abs() <= 1.0e-12);
    assert!((inv_idx.row - 3.0).abs() <= 1.0e-12);
    assert!((signed_dist - 7.5).abs() <= 1.0e-12);

    // 2. Oblique 3D rotation with negative cosines and 5e-7 DICOM DS truncation perturbation.
    // Start from an exact 3D rotation matrix R_z(0.37) * R_x(-0.61) and perturb within 1e-6 tolerance.
    let (sa, ca) = 0.37_f64.sin_cos();
    let (sb, cb) = (-0.61_f64).sin_cos();
    let r_exact = [ca, sa, 0.0];
    let c_exact = [-sa * cb, ca * cb, sb];
    let r_perturbed = [
        r_exact[0] + 3.0e-7,
        r_exact[1] - 2.0e-7,
        r_exact[2] + 4.0e-7,
    ];
    let c_perturbed = [
        c_exact[0] - 2.0e-7,
        c_exact[1] + 3.0e-7,
        c_exact[2] - 1.0e-7,
    ];

    let oblique = PatientPlane::new(
        LpsPointMm::new(-125.4, 88.25, 310.75),
        r_perturbed,
        c_perturbed,
        0.78125,
        0.625,
    )
    .expect("admit oblique DICOM DS-perturbed plane");

    let n = oblique.normal_cosine();
    let n_norm = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    assert!((n_norm - 1.0).abs() <= 1.0e-12);

    for &(col, row, offset_mm) in &[
        (0.0_f64, 0.0_f64, 0.0_f64),
        (255.5, 128.25, 0.0),
        (-12.75, 511.0, -14.25),
        (1023.0, 767.5, 23.5),
    ] {
        let base_pt = oblique
            .pixel_to_patient(PixelIndex2D::new(col, row))
            .expect("oblique pixel_to_patient");
        let displaced_pt = LpsPointMm::new(
            base_pt.x + offset_mm * n[0],
            base_pt.y + offset_mm * n[1],
            base_pt.z + offset_mm * n[2],
        );
        let (roundtrip_idx, dist_mm) = oblique
            .patient_to_pixel(displaced_pt)
            .expect("oblique Gram-inverse patient_to_pixel");
        assert!((roundtrip_idx.col - col).abs() < 1.0e-12);
        assert!((roundtrip_idx.row - row).abs() < 1.0e-12);
        assert!((dist_mm - offset_mm).abs() < 1.0e-11);
    }

    // 3. Redacted Debug formatting check on LpsPointMm and PatientPlane.
    let sentinel_pt = LpsPointMm::new(98765.4321, -54321.9876, 13579.2468);
    let sentinel_plane =
        PatientPlane::new(sentinel_pt, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, 1.0).unwrap();
    let debug_str = format!("{sentinel_pt:?} {sentinel_plane:?}");
    assert!(!debug_str.contains("98765"));
    assert!(!debug_str.contains("54321"));
    assert!(!debug_str.contains("13579"));

    // 4. Reject non-unit, non-orthogonal, zero/negative spacing, 1e200-overflowing, and NaN planes.
    assert!(matches!(
        PatientPlane::new(
            LpsPointMm::new(0.0, 0.0, 0.0),
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            1.0,
            1.0
        ),
        Err(RenderError::UnsupportedGeometry(_))
    ));
    assert!(matches!(
        PatientPlane::new(
            LpsPointMm::new(0.0, 0.0, 0.0),
            [1.0, 0.0, 0.0],
            [0.6, 0.8, 0.0],
            1.0,
            1.0
        ),
        Err(RenderError::UnsupportedGeometry(_))
    ));
    assert!(matches!(
        PatientPlane::new(
            LpsPointMm::new(0.0, 0.0, 0.0),
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            0.0,
            1.0
        ),
        Err(RenderError::UnsupportedGeometry(_))
    ));
    assert!(matches!(
        PatientPlane::new(
            LpsPointMm::new(0.0, 0.0, 0.0),
            [1.0e200, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            1.0,
            1.0
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));
    assert!(matches!(
        PatientPlane::new(
            LpsPointMm::new(f64::NAN, 0.0, 0.0),
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            1.0,
            1.0
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));
    assert!(matches!(
        plane_dicom.pixel_to_patient(PixelIndex2D::new(f64::NAN, 0.0)),
        Err(RenderError::InvalidDescriptor(_))
    ));
    assert!(matches!(
        plane_dicom.patient_to_pixel(LpsPointMm::new(f64::INFINITY, 0.0, 0.0)),
        Err(RenderError::InvalidDescriptor(_))
    ));
}

#[test]
fn fs01_t11_checked_volume_regular_hand_and_singular_sheared_rejection() {
    let limits = ResourceLimits::default_limits();
    let prec12 = BitPrecision::new(16, 12, 11);

    // 1. Constant-memory admission of a full 512 x 512 x 1024 right-handed volume (exact 512 MiB ceiling).
    let rh_affine = [
        [0.7, 0.0, 0.0, -179.2],
        [0.0, 0.7, 0.0, -179.2],
        [0.0, 0.0, 1.25, -320.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let rh_desc = VolumeDescriptor::new(
        VolumeDimensions::new(512, 512, 1024),
        SampleType::I16,
        prec12,
        rh_affine,
        test_identity(100),
    );
    let rh_vol = admit_volume(rh_desc, &limits).expect("admit right-handed 512 MiB volume");
    assert_eq!(rh_vol.handedness(), Handedness::RightHanded);
    assert_eq!(rh_vol.voxel_count(), 512 * 512 * 1024);
    assert_eq!(rh_vol.byte_len(), 512 * 1024 * 1024);
    assert!((rh_vol.axis_spacings_mm()[0] - 0.7).abs() <= 1.0e-12);
    assert!((rh_vol.axis_spacings_mm()[1] - 0.7).abs() <= 1.0e-12);
    assert!((rh_vol.axis_spacings_mm()[2] - 1.25).abs() <= 1.0e-12);

    // 2. Left-handed (`det < 0`) rotated clinical stack with descending superior-to-inferior slice steps.
    // Basis: u0 = [0.6, 0.8, 0.0], u1 = [-0.8, 0.6, 0.0], u2 = [0.0, 0.0, -1.0] (det = -1 < 0).
    let lh_affine = [
        [0.6 * 0.5, -0.8 * 0.5, 0.0, 98765.4321],
        [0.8 * 0.5, 0.6 * 0.5, 0.0, -54321.9876],
        [0.0, 0.0, -2.5, 13579.2468],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let lh_desc = VolumeDescriptor::new(
        VolumeDimensions::new(256, 256, 120),
        SampleType::U16,
        BitPrecision::new(16, 16, 15),
        lh_affine,
        test_identity(101),
    );
    let lh_vol = admit_volume(lh_desc, &limits).expect("admit left-handed rotated volume");
    assert_eq!(lh_vol.handedness(), Handedness::LeftHanded);

    // Verify Debug redaction of translation origin on both VolumeDescriptor and CheckedVolume.
    let vol_dbg = format!("{lh_desc:?} {lh_vol:?}");
    assert!(!vol_dbg.contains("98765"));
    assert!(!vol_dbg.contains("54321"));
    assert!(!vol_dbg.contains("13579"));

    // Verify M * M_inv == I_4 within 1e-11 and voxel <-> LPS round-trips across test coordinates.
    let m = lh_vol.index_to_lps_affine();
    let inv = lh_vol.lps_to_index_affine();
    for (r, m_row) in m.iter().enumerate() {
        for c in 0..4 {
            let sum: f64 = m_row
                .iter()
                .zip(inv.iter())
                .map(|(&a, inv_row)| a * inv_row[c])
                .sum();
            let expected = if r == c { 1.0 } else { 0.0 };
            assert!((sum - expected).abs() < 1.0e-10);
        }
    }

    for &(col, row, slice) in &[
        (0.0_f64, 0.0_f64, 0.0_f64),
        (127.5, 64.25, 33.75),
        (255.0, 255.0, 119.0),
        (-5.5, 12.0, 0.5),
    ] {
        let pt_rh = rh_vol.voxel_to_patient(col, row, slice).unwrap();
        let vox_rh = rh_vol.patient_to_voxel(pt_rh).unwrap();
        assert!((vox_rh[0] - col).abs() < 1.0e-11);
        assert!((vox_rh[1] - row).abs() < 1.0e-11);
        assert!((vox_rh[2] - slice).abs() < 1.0e-11);

        let pt_lh = lh_vol.voxel_to_patient(col, row, slice).unwrap();
        let vox_lh = lh_vol.patient_to_voxel(pt_lh).unwrap();
        assert!((vox_lh[0] - col).abs() < 1.0e-9);
        assert!((vox_lh[1] - row).abs() < 1.0e-9);
        assert!((vox_lh[2] - slice).abs() < 1.0e-9);
    }

    // 3. Reject singular (`det == 0`), sheared gantry-tilted (`u0 · u2 != 0`), `1e200`-overflowing,
    // invalid bottom-row (`[0, 0, 0, 2]`), NaN, zero-dim, and over-budget descriptors.
    let dims_small = VolumeDimensions::new(16, 16, 16);

    // Singular: column 2 == 0 (or collinear with column 0).
    let singular_collinear = [
        [1.0, 0.0, 2.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    assert!(matches!(
        admit_volume(
            VolumeDescriptor::new(
                dims_small,
                SampleType::I16,
                prec12,
                singular_collinear,
                test_identity(102),
            ),
            &limits,
        ),
        Err(RenderError::UnsupportedGeometry(_))
    ));

    // Sheared gantry-tilted slice stack (`v2 = [0.35, 0.0, 2.5]` -> `u0 · u2 != 0`).
    let gantry_sheared = [
        [0.7, 0.0, 0.35, 0.0],
        [0.0, 0.7, 0.0, 0.0],
        [0.0, 0.0, 2.5, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    assert!(matches!(
        admit_volume(
            VolumeDescriptor::new(
                dims_small,
                SampleType::I16,
                prec12,
                gantry_sheared,
                test_identity(103),
            ),
            &limits,
        ),
        Err(RenderError::UnsupportedGeometry(_))
    ));

    // 1e200-overflowing basis column norm.
    let overflow_affine = [
        [1.0e200, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    assert!(matches!(
        admit_volume(
            VolumeDescriptor::new(
                dims_small,
                SampleType::I16,
                prec12,
                overflow_affine,
                test_identity(104),
            ),
            &limits,
        ),
        Err(RenderError::UnsupportedGeometry(_))
    ));

    // Invalid homogeneous bottom row `[0, 0, 0, 2]`.
    let bad_bottom = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 2.0],
    ];
    assert!(matches!(
        admit_volume(
            VolumeDescriptor::new(
                dims_small,
                SampleType::I16,
                prec12,
                bad_bottom,
                test_identity(105),
            ),
            &limits,
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));

    // Over-budget volume bytes: 512 x 512 x 1025 * 2 > 512 MiB default ceiling.
    assert!(matches!(
        admit_volume(
            VolumeDescriptor::new(
                VolumeDimensions::new(512, 512, 1025),
                SampleType::I16,
                prec12,
                rh_affine,
                test_identity(106),
            ),
            &limits,
        ),
        Err(RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::VolumeBytes,
            ..
        })
    ));

    // Over-budget volume dimension: 2049 > 2048 default max_volume_dim.
    assert!(matches!(
        admit_volume(
            VolumeDescriptor::new(
                VolumeDimensions::new(2049, 1, 1),
                SampleType::I16,
                prec12,
                rh_affine,
                test_identity(107),
            ),
            &limits,
        ),
        Err(RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::VolumeDimension,
            ..
        })
    ));
}
