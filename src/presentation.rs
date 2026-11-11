// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::error::RenderError;
use crate::image::SourceIdentity;
use crate::resource::{BudgetGuard, OperationId};

pub(crate) const ROUNDING_POLICY_TIES_EVEN_U8: &str = "round_ties_even_u8";
pub(crate) const MAX_GPU_VS_CPU_INTERIOR_LSB_TOLERANCE: u8 = 1;

/// Physical unit produced by the modality rescale stage.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModalityUnit {
    Unspecified,
    HounsfieldUnit,
    OpticalDensity,
    Other,
}

/// Stored-sample-to-modality transformation stage.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntensityTransform {
    Identity,
    LinearRescale {
        slope: f64,
        intercept: f64,
        unit: ModalityUnit,
    },
    UnsupportedModalityLutSequence,
}

/// DICOM Value of Interest (`VOI`) transfer function identifier.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiFunction {
    Linear,
    LinearExact,
    Sigmoid,
}

/// VOI window/LUT configuration supplied in a [`VoiRequest`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiParameters {
    Linear { center: f64, width: f64 },
    LinearExact { center: f64, width: f64 },
    Sigmoid { center: f64, width: f64 },
    UnsupportedLutSequence,
}

/// DICOM greyscale photometric polarity applied after VOI scaling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhotometricPolarity {
    Monochrome2,
    Monochrome1,
}

/// Complete presentation request separating modality rescale, VOI windowing, polarity, and pad code.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct VoiRequest {
    pub voi: VoiParameters,
    pub rescale: IntensityTransform,
    pub polarity: PhotometricPolarity,
    pub padding_display_code: u8,
}

impl VoiRequest {
    #[must_use]
    pub const fn linear(center: f64, width: f64) -> Self {
        Self {
            voi: VoiParameters::Linear { center, width },
            rescale: IntensityTransform::Identity,
            polarity: PhotometricPolarity::Monochrome2,
            padding_display_code: 0,
        }
    }

    #[must_use]
    pub const fn new(
        voi: VoiParameters,
        rescale: IntensityTransform,
        polarity: PhotometricPolarity,
        padding_display_code: u8,
    ) -> Self {
        Self {
            voi,
            rescale,
            polarity,
            padding_display_code,
        }
    }

    pub(crate) fn validate(&self) -> Result<(f64, f64, f64, f64, ModalityUnit), RenderError> {
        let (center, width) = match self.voi {
            VoiParameters::Linear { center, width } => (center, width),
            VoiParameters::LinearExact { .. }
            | VoiParameters::Sigmoid { .. }
            | VoiParameters::UnsupportedLutSequence => {
                return Err(RenderError::UnsupportedPresentation(
                    "only DICOM LINEAR VOI windowing is admitted in FS-01",
                ));
            }
        };

        let (slope, intercept, unit) = match self.rescale {
            IntensityTransform::Identity => (1.0, 0.0, ModalityUnit::Unspecified),
            IntensityTransform::LinearRescale {
                slope,
                intercept,
                unit,
            } => (slope, intercept, unit),
            IntensityTransform::UnsupportedModalityLutSequence => {
                return Err(RenderError::UnsupportedPresentation(
                    "nonlinear Modality LUT sequence is not admitted in FS-01",
                ));
            }
        };

        if !center.is_finite() || !width.is_finite() || !slope.is_finite() || !intercept.is_finite()
        {
            return Err(RenderError::InvalidDescriptor(
                "VOI center, width, rescale slope, and intercept must all be finite",
            ));
        }

        if width < 1.0 {
            return Err(RenderError::InvalidDescriptor(
                "DICOM LINEAR window width must be >= 1.0",
            ));
        }

        let c_minus_half = center - 0.5;
        let half_span = (width - 1.0) * 0.5;
        let low = c_minus_half - half_span;
        let high = c_minus_half + half_span;
        if !c_minus_half.is_finite()
            || !half_span.is_finite()
            || !low.is_finite()
            || !high.is_finite()
        {
            return Err(RenderError::InvalidDescriptor(
                "VOI window thresholds overflowed f64",
            ));
        }

        let min_prod = -32768.0 * slope;
        let max_prod = 65535.0 * slope;
        let min_rescaled = min_prod + intercept;
        let max_rescaled = max_prod + intercept;
        if !min_prod.is_finite()
            || !max_prod.is_finite()
            || !min_rescaled.is_finite()
            || !max_rescaled.is_finite()
        {
            return Err(RenderError::InvalidDescriptor(
                "rescaled 16-bit container extrema overflowed f64",
            ));
        }

        Ok((center, width, slope, intercept, unit))
    }
}

/// Output pixel transfer encoding produced by `FS-01` presentation paths.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayEncoding {
    UncalibratedSrgbU8,
}

/// Geometry, bit depth, stride, and encoding metadata for a [`RenderedImage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputDescriptor {
    pub width: u32,
    pub height: u32,
    pub row_stride_bytes: usize,
    pub bits_per_pixel: u8,
    pub polarity: PhotometricPolarity,
    pub display_encoding: DisplayEncoding,
}

/// Execution backend path used to produce a [`RenderedImage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresentationPath {
    CpuReference,
    GpuOffscreen,
}

/// Native GPU API family used for an offscreen GPU render or capability probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuBackendKind {
    Metal,
    Vulkan,
    Dx12,
    Other,
}

/// Target hardware qualification classification recorded in [`RenderReceipt`] and capability reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualificationTier {
    PrimaryTargetCandidate,
    BoundedDevelopmentHost,
    SoftwareAdapter,
}

#[must_use]
pub(crate) const fn host_cpu_qualification_tier() -> QualificationTier {
    if (cfg!(target_os = "windows") || cfg!(target_os = "linux")) && cfg!(target_arch = "x86_64") {
        QualificationTier::PrimaryTargetCandidate
    } else {
        QualificationTier::BoundedDevelopmentHost
    }
}

/// GPU upload and staging byte accounting recorded in a [`RenderReceipt`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UploadAccounting {
    pub source_cache_hit: bool,
    pub source_bytes_uploaded: usize,
    pub staging_upload_bytes_reserved: usize,
    pub staging_readback_bytes_reserved: usize,
}

/// Host-observable stage durations recorded for a single render operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StageTimings {
    pub validation: Duration,
    pub host_encode_and_submit: Duration,
    pub gpu_wait_and_unpack: Duration,
    pub total: Duration,
}

/// Structured, privacy-safe provenance receipt accompanying every completed [`RenderedImage`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct RenderReceipt {
    pub operation_id: OperationId,
    pub source_identity: SourceIdentity,
    pub path: PresentationPath,
    pub backend: Option<GpuBackendKind>,
    pub qualification_tier: QualificationTier,
    pub shader_sha256: Option<Arc<str>>,
    pub output: OutputDescriptor,
    pub voi_function: VoiFunction,
    pub applied_center: f64,
    pub applied_width: f64,
    pub applied_slope: f64,
    pub applied_intercept: f64,
    pub modality_unit: ModalityUnit,
    pub polarity: PhotometricPolarity,
    pub padding_applied: bool,
    pub padding_display_code: u8,
    pub rounding_policy: &'static str,
    pub max_gpu_vs_cpu_interior_lsb_tolerance: u8,
    pub upload: UploadAccounting,
    pub timings: StageTimings,
}

/// Tight 8-bit greyscale output buffer paired with its [`OutputDescriptor`], [`RenderReceipt`],
/// and RAII CPU output budget guard.
pub struct RenderedImage {
    pixels: Vec<u8>,
    _output_budget: BudgetGuard,
    descriptor: OutputDescriptor,
    receipt: RenderReceipt,
}

impl fmt::Debug for RenderedImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderedImage")
            .field("descriptor", &self.descriptor)
            .field("receipt", &self.receipt)
            .field("pixel_byte_len", &self.pixels.len())
            .field("pixels", &"<redacted>")
            .finish()
    }
}

impl RenderedImage {
    #[must_use]
    pub(crate) fn new(
        pixels: Vec<u8>,
        output_budget: BudgetGuard,
        descriptor: OutputDescriptor,
        receipt: RenderReceipt,
    ) -> Self {
        Self {
            pixels,
            _output_budget: output_budget,
            descriptor,
            receipt,
        }
    }

    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    #[must_use]
    pub const fn descriptor(&self) -> &OutputDescriptor {
        &self.descriptor
    }

    #[must_use]
    pub const fn receipt(&self) -> &RenderReceipt {
        &self.receipt
    }
}
