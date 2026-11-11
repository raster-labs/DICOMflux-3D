//! Checked 2D 16-bit DICOM greyscale presentation and `f64` patient LPS spatial geometry (`FS-01`).

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

mod error;
mod image;
mod presentation;
mod rendering;
mod resource;
mod spatial;
mod volume;

pub use error::{LimitPoolKind, RenderError};
pub use image::{
    BitPrecision, CheckedFrame, ColorModel, DecodedFrameLease, FrameDescriptor, PixelPadding,
    SampleType, SourceIdentity, admit_frame,
};
pub use presentation::{
    DisplayEncoding, GpuBackendKind, IntensityTransform, ModalityUnit, OutputDescriptor,
    PhotometricPolarity, PresentationPath, QualificationTier, RenderReceipt, RenderedImage,
    StageTimings, UploadAccounting, VoiFunction, VoiParameters, VoiRequest,
};
pub use rendering::{
    AdapterDeviceType, AdapterSelectionPolicy, AllowedBackends, CapabilityReport, GpuCapabilities,
    GpuContext, probe_capabilities, render_offscreen, render_reference,
};
pub use resource::{
    CancellationToken, OperationId, PoolUsageSnapshot, ResourceLimits, ResourceTracker,
};
pub use spatial::{LpsPointMm, PatientPlane, PixelIndex2D};
pub use volume::{CheckedVolume, Handedness, VolumeDescriptor, VolumeDimensions, admit_volume};
