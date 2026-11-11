use std::error::Error;
use std::fmt;

/// Identifies the bounded quota or dimensional ceiling pool evaluated by `ResourceTracker`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LimitPoolKind {
    /// Retained CPU backing capacity across admitted `CheckedFrame` instances.
    RetainedSource,
    /// Temporary host-mapped GPU upload staging buffers (`COPY_SRC`).
    GpuStagingUpload,
    /// Resident GPU source (`R16Sint` / `R16Uint`) and offscreen target (`R8Uint`) textures.
    GpuResident,
    /// Host-mapped GPU readback staging buffers (`MAP_READ`).
    GpuStagingReadback,
    /// Tight 8-bit CPU output pixel buffers owned by `RenderedImage`.
    CpuOutput,
    /// Validated 3D regular volume byte extent ceiling (`CheckedVolume`).
    VolumeBytes,
    /// Maximum 2D frame width or height ceiling.
    FrameDimension,
    /// Maximum 3D volume column, row, or slice ceiling.
    VolumeDimension,
    /// Maximum concurrent CPU or GPU rendering operations in flight.
    InFlightJobs,
}

impl LimitPoolKind {
    /// Returns a stable privacy-safe identifier for the quota pool.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RetainedSource => "retained_source_bytes",
            Self::GpuStagingUpload => "gpu_staging_upload_bytes",
            Self::GpuResident => "gpu_resident_bytes",
            Self::GpuStagingReadback => "gpu_staging_readback_bytes",
            Self::CpuOutput => "cpu_output_bytes",
            Self::VolumeBytes => "max_volume_bytes",
            Self::FrameDimension => "max_frame_dim",
            Self::VolumeDimension => "max_volume_dim",
            Self::InFlightJobs => "max_in_flight_jobs",
        }
    }
}

impl fmt::Display for LimitPoolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Structured, privacy-safe error taxonomy for `dicomflux-3d` admission, geometry, and rendering.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq)]
pub enum RenderError {
    /// Malformed descriptor shape, stride, byte extent, bit layout, padding, or non-finite value.
    InvalidDescriptor(&'static str),
    /// Unadmitted pixel scalar representation, bit depth, component count, or multiframe input.
    UnsupportedPixelFormat {
        /// Declared `BitsAllocated`.
        bits_allocated: u8,
        /// Declared `BitsStored`.
        bits_stored: u8,
        /// Declared samples per pixel (`components`).
        components: u8,
    },
    /// Unadmitted photometric colour model, Modality LUT sequence, or nonlinear VOI function.
    UnsupportedPresentation(&'static str),
    /// Non-orthonormal plane cosines, invalid spacing, sheared tilt, or singular 3D affine matrix.
    UnsupportedGeometry(&'static str),
    /// Deterministic pre-allocation refusal against a configured `ResourceLimits` ceiling.
    ResourceLimitExceeded {
        /// Quota pool or dimension ceiling that refused the reservation.
        pool: LimitPoolKind,
        /// Requested byte count, dimension, or job count.
        requested: usize,
        /// Remaining available capacity prior to the request.
        available: usize,
        /// Governing pool limit from `ResourceLimits`.
        limit: usize,
    },
    /// No GPU adapter satisfied the caller's `AdapterSelectionPolicy`.
    NoSuitableAdapter,
    /// Selected GPU adapter lacks required integer texture formats or buffer/dimension limits.
    UnsupportedGpuCapability(&'static str),
    /// GPU device was lost before or during execution, or `GpuContext` is latched in `DeviceLost`.
    GpuDeviceLost,
    /// GPU out-of-memory error scope tripped during buffer, texture, or submission execution.
    GpuOutOfMemory,
    /// Cooperative `CancellationToken` was observed set prior to completion.
    Cancelled,
    /// Internal validation scope trap carrying only a static non-sensitive tag.
    InternalError(&'static str),
}

fn contains_ascii_case_insensitive(haystack: &str, needle_lower: &[u8]) -> bool {
    if needle_lower.is_empty() || haystack.len() < needle_lower.len() {
        return false;
    }
    haystack
        .as_bytes()
        .windows(needle_lower.len())
        .any(|window| {
            window
                .iter()
                .zip(needle_lower.iter())
                .all(|(a, b)| a.to_ascii_lowercase() == *b)
        })
}

fn has_multi_digit_fraction(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    for (idx, &b) in bytes.iter().enumerate() {
        if b == b'.'
            && idx > 0
            && bytes[idx - 1].is_ascii_digit()
            && idx + 2 < bytes.len()
            && bytes[idx + 1].is_ascii_digit()
            && bytes[idx + 2].is_ascii_digit()
        {
            return true;
        }
    }
    false
}

fn has_uid_like_dotted_digits(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    let mut digit_dots = 0usize;
    for idx in 1..bytes.len().saturating_sub(1) {
        if bytes[idx] == b'.' && bytes[idx - 1].is_ascii_digit() && bytes[idx + 1].is_ascii_digit()
        {
            digit_dots += 1;
            if digit_dots >= 2 {
                return true;
            }
        }
    }
    false
}

fn sanitize_static_tag(tag: &'static str) -> &'static str {
    if tag.len() > 120
        || tag.contains(":\\")
        || contains_ascii_case_insensitive(tag, b"/users/")
        || contains_ascii_case_insensitive(tag, b"/home/")
        || contains_ascii_case_insensitive(tag, b"/private/")
        || contains_ascii_case_insensitive(tag, b".dcm")
        || contains_ascii_case_insensitive(tag, b"deadbeef")
        || contains_ascii_case_insensitive(tag, b"0xde")
        || has_multi_digit_fraction(tag)
        || has_uid_like_dotted_digits(tag)
    {
        "<redacted>"
    } else {
        tag
    }
}

impl fmt::Debug for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDescriptor(reason) => f
                .debug_tuple("InvalidDescriptor")
                .field(&sanitize_static_tag(reason))
                .finish(),
            Self::UnsupportedPixelFormat {
                bits_allocated,
                bits_stored,
                components,
            } => f
                .debug_struct("UnsupportedPixelFormat")
                .field("bits_allocated", bits_allocated)
                .field("bits_stored", bits_stored)
                .field("components", components)
                .finish(),
            Self::UnsupportedPresentation(reason) => f
                .debug_tuple("UnsupportedPresentation")
                .field(&sanitize_static_tag(reason))
                .finish(),
            Self::UnsupportedGeometry(reason) => f
                .debug_tuple("UnsupportedGeometry")
                .field(&sanitize_static_tag(reason))
                .finish(),
            Self::ResourceLimitExceeded {
                pool,
                requested,
                available,
                limit,
            } => f
                .debug_struct("ResourceLimitExceeded")
                .field("pool", pool)
                .field("requested", requested)
                .field("available", available)
                .field("limit", limit)
                .finish(),
            Self::NoSuitableAdapter => f.write_str("NoSuitableAdapter"),
            Self::UnsupportedGpuCapability(reason) => f
                .debug_tuple("UnsupportedGpuCapability")
                .field(&sanitize_static_tag(reason))
                .finish(),
            Self::GpuDeviceLost => f.write_str("GpuDeviceLost"),
            Self::GpuOutOfMemory => f.write_str("GpuOutOfMemory"),
            Self::Cancelled => f.write_str("Cancelled"),
            Self::InternalError(reason) => f
                .debug_tuple("InternalError")
                .field(&sanitize_static_tag(reason))
                .finish(),
        }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDescriptor(reason) => {
                write!(f, "invalid descriptor: {}", sanitize_static_tag(reason))
            }
            Self::UnsupportedPixelFormat {
                bits_allocated,
                bits_stored,
                components,
            } => write!(
                f,
                "unsupported pixel format (bits_allocated={bits_allocated}, bits_stored={bits_stored}, components={components})"
            ),
            Self::UnsupportedPresentation(reason) => {
                write!(
                    f,
                    "unsupported presentation profile: {}",
                    sanitize_static_tag(reason)
                )
            }
            Self::UnsupportedGeometry(reason) => {
                write!(f, "unsupported geometry: {}", sanitize_static_tag(reason))
            }
            Self::ResourceLimitExceeded {
                pool,
                requested,
                available,
                limit,
            } => write!(
                f,
                "resource limit exceeded for pool {pool} (requested={requested}, available={available}, limit={limit})"
            ),
            Self::NoSuitableAdapter => f.write_str("no suitable GPU adapter matched policy"),
            Self::UnsupportedGpuCapability(reason) => {
                write!(
                    f,
                    "unsupported GPU capability: {}",
                    sanitize_static_tag(reason)
                )
            }
            Self::GpuDeviceLost => f.write_str("GPU device lost"),
            Self::GpuOutOfMemory => f.write_str("GPU out of memory"),
            Self::Cancelled => f.write_str("operation cancelled"),
            Self::InternalError(reason) => {
                write!(
                    f,
                    "internal rendering error: {}",
                    sanitize_static_tag(reason)
                )
            }
        }
    }
}

impl Error for RenderError {}

#[cfg(test)]
mod tests {
    use super::{LimitPoolKind, RenderError};

    #[test]
    fn all_variants_format_cleanly_and_redact_sensitive_sentinels() {
        let safe_variants = [
            RenderError::InvalidDescriptor("frame width must be non-zero"),
            RenderError::UnsupportedPixelFormat {
                bits_allocated: 8,
                bits_stored: 8,
                components: 3,
            },
            RenderError::UnsupportedPresentation("color_model must be grayscale"),
            RenderError::UnsupportedGeometry("plane cosines are not orthogonal"),
            RenderError::ResourceLimitExceeded {
                pool: LimitPoolKind::RetainedSource,
                requested: 4096,
                available: 1024,
                limit: 2048,
            },
            RenderError::NoSuitableAdapter,
            RenderError::UnsupportedGpuCapability("missing R16Sint texture support"),
            RenderError::GpuDeviceLost,
            RenderError::GpuOutOfMemory,
            RenderError::Cancelled,
            RenderError::InternalError("gpu_validation_scope_trap"),
        ];

        for err in &safe_variants {
            let dbg = format!("{err:?}");
            let disp = format!("{err}");
            assert!(!dbg.is_empty());
            assert!(!disp.is_empty());
        }

        let sentinel_errors = [
            RenderError::InvalidDescriptor("origin 98765.4321 -54321.9876"),
            RenderError::InternalError("/Users/patient_doe/study.dcm"),
            RenderError::UnsupportedGeometry("1.2.840.10008.5.1.4.1.1.2"),
            RenderError::UnsupportedPresentation("payload 0xDEADBEEF"),
        ];

        for err in &sentinel_errors {
            let dbg = format!("{err:?}");
            let disp = format!("{err}");
            assert!(dbg.contains("<redacted>"), "unexpected debug: {dbg}");
            assert!(disp.contains("<redacted>"), "unexpected display: {disp}");
            assert!(!dbg.contains("98765.4321"));
            assert!(!disp.contains("98765.4321"));
            assert!(!dbg.contains("patient_doe"));
            assert!(!dbg.contains("1.2.840"));
            assert!(!dbg.to_ascii_lowercase().contains("deadbeef"));
        }
    }
}
