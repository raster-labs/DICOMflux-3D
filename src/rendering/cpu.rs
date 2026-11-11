// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::time::{Duration, Instant};

use crate::error::{LimitPoolKind, RenderError};
use crate::image::{CheckedFrame, PixelPadding};
use crate::presentation::{
    DisplayEncoding, MAX_GPU_VS_CPU_INTERIOR_LSB_TOLERANCE, OutputDescriptor, PhotometricPolarity,
    PresentationPath, QualificationTier, ROUNDING_POLICY_TIES_EVEN_U8, RenderReceipt,
    RenderedImage, StageTimings, UploadAccounting, VoiFunction, VoiRequest,
};
use crate::resource::{CancellationToken, ResourceTracker};

#[must_use]
pub(crate) const fn host_cpu_qualification_tier() -> QualificationTier {
    crate::presentation::host_cpu_qualification_tier()
}

#[must_use]
pub(crate) fn eval_linear_voi_code(
    x: f64,
    center: f64,
    width: f64,
    polarity: PhotometricPolarity,
) -> u8 {
    let c_minus_half = center - 0.5;
    let half_span = (width - 1.0) * 0.5;
    let low = c_minus_half - half_span;
    let high = c_minus_half + half_span;

    let y_voi = if x <= low {
        0.0
    } else if x > high {
        255.0
    } else {
        ((x - c_minus_half) / (width - 1.0) + 0.5) * 255.0
    };

    let y_pol = match polarity {
        PhotometricPolarity::Monochrome2 => y_voi,
        PhotometricPolarity::Monochrome1 => 255.0 - y_voi,
    };

    y_pol.clamp(0.0, 255.0).round_ties_even() as u8
}

/// Deterministic `f64` CPU reference renderer implementing stored-domain pixel padding, linear
/// modality rescale, DICOM PS3.3 `C.11.2.1.2.1` `LINEAR` VOI windowing, photometric polarity,
/// and round-ties-to-even `u8` quantisation with zero GPU dependency.
pub fn render_reference(
    frame: &CheckedFrame,
    request: &VoiRequest,
    tracker: &ResourceTracker,
    cancel: &CancellationToken,
) -> Result<RenderedImage, RenderError> {
    let total_start = Instant::now();
    let validation_start = Instant::now();

    if cancel.is_cancelled() {
        return Err(RenderError::Cancelled);
    }

    let (center, width, slope, intercept, modality_unit) = request.validate()?;

    let out_w = usize::try_from(frame.width())
        .map_err(|_| RenderError::InvalidDescriptor("frame width exceeds host usize"))?;
    let out_h = usize::try_from(frame.height())
        .map_err(|_| RenderError::InvalidDescriptor("frame height exceeds host usize"))?;
    let output_bytes = out_w
        .checked_mul(out_h)
        .ok_or(RenderError::InvalidDescriptor(
            "CPU output pixel count overflow",
        ))?;

    let _in_flight_guard = tracker.reserve_job()?;
    let output_guard = tracker.reserve(LimitPoolKind::CpuOutput, output_bytes)?;
    let validation_dur = validation_start.elapsed();

    let exec_start = Instant::now();
    let mut pixels = Vec::with_capacity(output_bytes);
    let padding = frame.pixel_padding();
    let pad_code = request.padding_display_code;

    for row in 0..frame.height() {
        if cancel.is_cancelled() {
            return Err(RenderError::Cancelled);
        }
        for col in 0..frame.width() {
            let s = frame.sample_i32(col, row);
            if padding.matches(s) {
                pixels.push(pad_code);
                continue;
            }
            let x = f64::from(s) * slope + intercept;
            pixels.push(eval_linear_voi_code(x, center, width, request.polarity));
        }
    }

    if cancel.is_cancelled() {
        return Err(RenderError::Cancelled);
    }

    let exec_dur = exec_start.elapsed();
    let total_dur = total_start.elapsed();

    let output = OutputDescriptor {
        width: frame.width(),
        height: frame.height(),
        row_stride_bytes: out_w,
        bits_per_pixel: 8,
        polarity: request.polarity,
        display_encoding: DisplayEncoding::UncalibratedSrgbU8,
    };

    let receipt = RenderReceipt {
        operation_id: tracker.next_operation_id(),
        source_identity: frame.identity(),
        path: PresentationPath::CpuReference,
        backend: None,
        qualification_tier: host_cpu_qualification_tier(),
        shader_sha256: None,
        output,
        voi_function: VoiFunction::Linear,
        applied_center: center,
        applied_width: width,
        applied_slope: slope,
        applied_intercept: intercept,
        modality_unit,
        polarity: request.polarity,
        padding_applied: !matches!(padding, PixelPadding::None),
        padding_display_code: pad_code,
        rounding_policy: ROUNDING_POLICY_TIES_EVEN_U8,
        max_gpu_vs_cpu_interior_lsb_tolerance: MAX_GPU_VS_CPU_INTERIOR_LSB_TOLERANCE,
        upload: UploadAccounting {
            source_cache_hit: false,
            source_bytes_uploaded: 0,
            staging_upload_bytes_reserved: 0,
            staging_readback_bytes_reserved: 0,
        },
        timings: StageTimings {
            validation: validation_dur,
            host_encode_and_submit: exec_dur,
            gpu_wait_and_unpack: Duration::ZERO,
            total: total_dur,
        },
    };

    Ok(RenderedImage::new(pixels, output_guard, output, receipt))
}
