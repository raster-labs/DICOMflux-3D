# DICOMflux-3D (`dicomflux-3d` `0.1.0` — First Slice `FS-01`)

`dicomflux-3d` is a standalone, MIT-licensed Rust library owned by Raster Images for checked 2D 16-bit medical greyscale presentation and `f64` patient Left-Posterior-Superior (`LPS`) spatial geometry validation.

## First Slice (`FS-01`) Capability Scope

`dicomflux-3d` `0.1.0` (`FS-01`) implements **2D-only** single-frame 16-bit greyscale presentation and spatial geometry contracts:

- **Immutable 16-Bit Frame Admission (`admit_frame`, `CheckedFrame`, `DecodedFrameLease`)**: Fail-closed validation of canonical 16-bit signed (`i16`) and unsigned (`u16`) single-plane greyscale descriptors (`BitsAllocated == 16`, `1 <= BitsStored <= 16`, `HighBit == BitsStored - 1`), stored-container `PixelPadding`, even byte offsets/row strides, and pre-allocation `ResourceLimits` quotas without mutating caller pixel bytes.
- **Four-Stage DICOM Greyscale Presentation (`render_reference` & `render_offscreen`)**: Stored-container pixel padding mask evaluation, finite linear modality rescale (`slope` and `intercept`), DICOM PS3.3 `LINEAR` VOI window centre/width (`width >= 1.0`), `MONOCHROME1` / `MONOCHROME2` photometric polarity, and round-ties-to-even quantization into tight 8-bit uncalibrated sRGB greyscale buffers (`DisplayEncoding::UncalibratedSrgbU8`) on both an analytic `f64` CPU reference oracle and a quarantined synchronous offscreen `wgpu` (`WGSL`) pipeline with single-slot texture residency and `RenderReceipt` provenance.
- **Authoritative `f64` Spatial Geometry (`PatientPlane` & `CheckedVolume`)**: Constant-memory (`O(1)`) validation and analytical forward/inverse coordinate projection for 2D DICOM Image Plane (`LPS` mm) orientations and regular orthogonal 3D volume affine descriptors without inferring slice spacing from `SliceThickness` or filenames.
- **Bounded Budgets, Capability Probing & Privacy-Safe Diagnostics**: Pre-work `probe_capabilities` inspection, atomic `ResourceTracker` quota enforcement across CPU and GPU pools, cooperative `CancellationToken` handling, `#![forbid(unsafe_code)]`, and PHI-redacted `Debug`/`Display` formatting.

## Non-Qualification, Scope Exclusions & Clinical Safety Notice

1. **Explicit `FS-01` Scope Exclusions**: `FS-01` is strictly a **2D-only** greyscale presentation and geometry validation slice. It does **not** implement Multi-Planar Reconstruction (`MPR`), Maximum or Minimum Intensity Projection (`MIP` / `MinIP`), Direct Volume Rendering (`DVR`), 3D volume rendering, 3D GPU texture uploads, threshold or surface segmentation, DICOM Part 10 file parsing, compressed transfer-syntax codecs, image resampling, `LINEAR_EXACT` / `SIGMOID` / LUT sequences, DICOM PS3.14 GSDF display calibration, or GUI windowing (`egui` / `winit`).
2. **Non-Clinical / Non-Medical-Device Status**: `dicomflux-3d` is an engineering software component under active development. It is **not a medical device**, is **not** cleared, certified, or qualified for clinical diagnostic use or primary clinical interpretation, and makes no clinical diagnostic claims. Downstream integrators remain solely responsible for system-level verification, display calibration, and regulatory qualification.
3. **Platform Qualification Honesty (`BoundedDevelopmentHost`)**: Primary target qualification is defined for physical Windows `x86_64` (`Dx12` / `Vulkan`, `FS01-T19`) and Linux `x86_64` (`Vulkan`, `FS01-T20`) workstations (with `ARM64` tracked separately under `FS01-T21`). Executions on macOS Metal stamp `QualificationTier::BoundedDevelopmentHost` in `CapabilityReport` and `RenderReceipt` and constitute bounded development-host verification rather than primary workstation qualification.

## Verification & Build Commands

Run formatting, lint, test, standalone outside-workspace consumer, and reproducible evidence checks from the repository root using standard Cargo commands:

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo test --locked --manifest-path examples/independent-consumer/Cargo.toml
cargo run --locked --manifest-path examples/independent-consumer/Cargo.toml
cargo run --release --locked --example generate_evidence
shasum -a 256 -c evidence/SHA256SUMS.txt
```

## Licence

Licensed under the [MIT License](LICENSE). Copyright (c) 2026 Raster Images.
