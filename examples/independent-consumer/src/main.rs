// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::error::Error;
use std::num::NonZeroU64;

use dicomflux_3d::{
    AdapterSelectionPolicy, BitPrecision, CancellationToken, DecodedFrameLease, DisplayEncoding,
    FrameDescriptor, GpuContext, Handedness, IntensityTransform, LpsPointMm, ModalityUnit,
    PatientPlane, PhotometricPolarity, PixelIndex2D, PixelPadding, PresentationPath,
    ResourceTracker, SampleType, SourceIdentity, VoiParameters, VoiRequest, VolumeDescriptor,
    VolumeDimensions, admit_frame, admit_volume, probe_capabilities, render_offscreen,
    render_reference,
};

fn build_synthetic_ct_phantom_64x64() -> Vec<i16> {
    let width = 64usize;
    let height = 64usize;
    let mut words = Vec::with_capacity(width * height);
    for row in 0..height {
        for col in 0..width {
            if row == 0 || row == height - 1 || col == 0 || col == width - 1 {
                words.push(-2000_i16);
            } else {
                let idx = (row * width + col) as i32;
                let signed_12bit = -1024 + (idx % 2048);
                words.push(signed_12bit as i16);
            }
        }
    }
    words
}

pub fn run_consumer_smoke() -> Result<(), Box<dyn Error>> {
    let tracker = ResourceTracker::with_default_limits();
    let cancel = CancellationToken::new();
    let identity = SourceIdentity::new(NonZeroU64::new(1).ok_or("invalid NonZeroU64")?, 1);

    // 1. Admit a 64 x 64 signed 12-bit i16 CT-like phantom with stored-container padding (-2000).
    let phantom_words = build_synthetic_ct_phantom_64x64();
    let mut frame_desc = FrameDescriptor::new(
        64,
        64,
        64 * 2,
        SampleType::I16,
        BitPrecision::new(16, 12, 11),
    );
    frame_desc.pixel_padding = PixelPadding::Single(-2000);
    let lease = DecodedFrameLease::from_vec_i16(phantom_words, identity)?;
    let checked_frame = admit_frame(frame_desc, lease, &tracker)?;

    // 2. Validate f64 DICOM Image Plane (PatientPlane) projection & inverse round-trip.
    let plane = PatientPlane::from_dicom_image_plane(
        [-120.0, -120.0, 45.0],
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [0.75, 0.75],
    )?;
    let target_idx = PixelIndex2D::new(16.0, 24.0);
    let patient_pt: LpsPointMm = plane.pixel_to_patient(target_idx)?;
    let (roundtrip_idx, normal_dist_mm) = plane.patient_to_pixel(patient_pt)?;
    if (roundtrip_idx.col - target_idx.col).abs() > 1.0e-12
        || (roundtrip_idx.row - target_idx.row).abs() > 1.0e-12
        || normal_dist_mm.abs() > 1.0e-12
    {
        return Err("PatientPlane round-trip exceeded analytical tolerance".into());
    }

    // 3. Validate constant-memory regular 3D CheckedVolume (64 x 64 x 16).
    let vol_desc = VolumeDescriptor::new(
        VolumeDimensions::new(64, 64, 16),
        SampleType::I16,
        BitPrecision::new(16, 12, 11),
        [
            [0.75, 0.0, 0.0, -120.0],
            [0.0, 0.75, 0.0, -120.0],
            [0.0, 0.0, 1.5, 45.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        identity,
    );
    let checked_vol = admit_volume(vol_desc, tracker.limits())?;
    if checked_vol.handedness() != Handedness::RightHanded {
        return Err("expected RightHanded axial volume basis".into());
    }
    let vox_pt = checked_vol.voxel_to_patient(10.0, 20.0, 5.0)?;
    let vox_rt = checked_vol.patient_to_voxel(vox_pt)?;
    if (vox_rt[0] - 10.0).abs() > 1.0e-12
        || (vox_rt[1] - 20.0).abs() > 1.0e-12
        || (vox_rt[2] - 5.0).abs() > 1.0e-12
    {
        return Err("CheckedVolume round-trip exceeded analytical tolerance".into());
    }

    // 4. Run analytic f64 CPU reference presentation (CT soft-tissue window 40 / 400).
    let pad_code = 17_u8;
    let soft_tissue_req = VoiRequest::new(
        VoiParameters::Linear {
            center: 40.0,
            width: 400.0,
        },
        IntensityTransform::LinearRescale {
            slope: 1.0,
            intercept: -1024.0,
            unit: ModalityUnit::HounsfieldUnit,
        },
        PhotometricPolarity::Monochrome2,
        pad_code,
    );
    let cpu_img = render_reference(&checked_frame, &soft_tissue_req, &tracker, &cancel)?;
    if cpu_img.pixels().len() != 64 * 64
        || cpu_img.descriptor().row_stride_bytes != 64
        || cpu_img.descriptor().display_encoding != DisplayEncoding::UncalibratedSrgbU8
        || cpu_img.receipt().path != PresentationPath::CpuReference
        || cpu_img.pixels()[0] != pad_code
    {
        return Err("CPU reference output failed contract verification".into());
    }

    // 5. Probe GPU capabilities and execute offscreen wgpu parity + warm W/L residency check.
    let policy = AdapterSelectionPolicy::default();
    let capabilities = probe_capabilities(&policy);
    if !capabilities.gpu_adapters.is_empty() {
        let gpu_ctx = GpuContext::new(&policy, &tracker)?;
        let gpu_cold = render_offscreen(&gpu_ctx, &checked_frame, &soft_tissue_req, &cancel)?;
        if gpu_cold.receipt().path != PresentationPath::GpuOffscreen
            || gpu_cold.receipt().upload.source_cache_hit
            || gpu_cold.receipt().upload.source_bytes_uploaded != 64 * 64 * 2
        {
            return Err("cold GPU render receipt accounting mismatch".into());
        }

        let mut max_abs_diff = 0_u8;
        for (&c, &g) in cpu_img.pixels().iter().zip(gpu_cold.pixels().iter()) {
            max_abs_diff = max_abs_diff.max(c.abs_diff(g));
        }
        if max_abs_diff > 1 || gpu_cold.pixels()[0] != pad_code {
            return Err(format!("CPU vs GPU max_abs_diff={max_abs_diff} exceeded 1 LSB").into());
        }

        let lung_req = VoiRequest::new(
            VoiParameters::Linear {
                center: -600.0,
                width: 1500.0,
            },
            IntensityTransform::LinearRescale {
                slope: 1.0,
                intercept: -1024.0,
                unit: ModalityUnit::HounsfieldUnit,
            },
            PhotometricPolarity::Monochrome2,
            pad_code,
        );
        let cpu_lung = render_reference(&checked_frame, &lung_req, &tracker, &cancel)?;
        let gpu_warm = render_offscreen(&gpu_ctx, &checked_frame, &lung_req, &cancel)?;
        if !gpu_warm.receipt().upload.source_cache_hit
            || gpu_warm.receipt().upload.source_bytes_uploaded != 0
        {
            return Err("warm GPU W/L update failed single-slot residency contract".into());
        }
        let warm_max_diff = cpu_lung
            .pixels()
            .iter()
            .zip(gpu_warm.pixels().iter())
            .map(|(&c, &g)| c.abs_diff(g))
            .max()
            .unwrap_or(0);
        if warm_max_diff > 1 {
            return Err(
                format!("warm W/L CPU vs GPU max_abs_diff={warm_max_diff} exceeded 1 LSB").into(),
            );
        }

        println!(
            "dicomflux-3d independent consumer smoke: PASS (frame=64x64 i16, volume=64x64x16, gpu_backend={:?}, qualification={:?}, cold_max_diff={}, warm_max_diff={})",
            gpu_warm.receipt().backend,
            gpu_warm.receipt().qualification_tier,
            max_abs_diff,
            warm_max_diff
        );
    } else {
        println!(
            "dicomflux-3d independent consumer smoke: PASS (frame=64x64 i16, volume=64x64x16, gpu_adapters=0, cpu_qualification={:?})",
            capabilities.host_qualification_tier
        );
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run_consumer_smoke()
}

#[cfg(test)]
mod tests {
    use super::run_consumer_smoke;

    #[test]
    fn independent_consumer_end_to_end() {
        run_consumer_smoke().expect("independent consumer smoke check must succeed");
    }
}
