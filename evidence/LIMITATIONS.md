# Known Limitations, Open Platform Gates & Non-Clinical Notice (`FS-01`)

## 1. Open Primary Target Hardware Gates (`NOT_RUN` / `OPEN_PENDING_TARGET_HARDWARE`)

All verification in this bundle executed on `aarch64-apple-darwin` (`QualificationTier::BoundedDevelopmentHost`). Cross-compilation is not device execution; the following primary target gates remain `NOT_RUN` in [`evidence/TEST_RESULTS.json`](TEST_RESULTS.json) until executed on physical target workstations:

| Case ID | Target Platform & Graphics Path | Current Status |
| :--- | :--- | :--- |
| `FS01-T19` | Physical Windows `x86_64` workstation (`Direct3D 12` / `Vulkan` + CPU) | `NOT_RUN` (`OPEN_PENDING_TARGET_HARDWARE`) |
| `FS01-T20` | Physical Linux `x86_64` workstation (`Vulkan` + CPU) | `NOT_RUN` (`OPEN_PENDING_TARGET_HARDWARE`) |
| `FS01-T21` | Physical Windows `ARM64` / Linux `ARM64` target device | `NOT_RUN` (`OPEN_PENDING_TARGET_HARDWARE`) |

## 2. Explicit `FS-01` Capability Exclusions

- **No DICOM File Parsing, Codecs, or Resampling**: `dicomflux-3d` does not parse DICOM Part 10 files, tags, or UIDs, decode compressed transfer syntaxes, export DICOM `SEG`/`SR`, or resample images.
- **No 3D Volumetric Rendering or Segmentation in `FS-01`**: Multi-Planar Reconstruction (`MPR`), Maximum/Minimum Intensity Projection (`MIP`/`MinIP`), Direct Volume Rendering (`DVR`), 3D GPU texture residency, bricked volume streaming, threshold masks, Marching Cubes surface extraction, and `parry3d` spatial queries are deferred to `FS-02`–`FS-04`.
- **Excluded Sample & Presentation Subprofiles**: Multiframe (`frame_count > 1`), non-16-bit scalar containers (`u8`, `i32`, `f32`), palette/colour photometrics, `LINEAR_EXACT` and `SIGMOID` VOI functions, nonlinear Modality or VOI LUT sequences, GSPS, and DICOM PS3.14 GSDF display calibration fail closed with `UnsupportedPixelFormat` or `UnsupportedPresentation`.
- **No Windowing GUI, Foreign-Language FFI, or Browser Builds**: `egui`/`winit` viewports, C-ABI / Java Panama FFM bindings, Android JNI, and WebAssembly/WebGPU targets are excluded from `FS-01`.

## 3. Architectural & Operational Constraints

- **Single-Slot GPU Texture Residency**: `GpuContext` caches one resident 2D source texture keyed by `(SourceIdentity, FrameDescriptor)`. Alternating between two distinct frames evicts and re-uploads on each switch until multi-entry residency is introduced in `FS-02`.
- **Synchronous Stepped Polling & Post-Submit Cancellation Retention**: `render_offscreen` blocks its calling OS thread in `10 ms` `device.poll` increments. If `CancellationToken` trips after `queue.submit`, `render_offscreen` returns `Err(RenderError::Cancelled)` immediately to the caller while retaining submitted GPU buffers, textures, and `Arc<BudgetGuard>` quotas in `InFlightLedger` until hardware queue completion (`on_submitted_work_done` / `poll_reap()`).
- **Sub-16-Bit Admission Scan**: When `bits_stored < 16`, `admit_frame` performs one `O(width * height)` scan over active words to verify zero-filled (`u16`) or sign-extended (`i16`) upper bits while exempting declared container-domain `PixelPadding` values.

## 4. Non-Clinical / Non-Medical-Device Safety Notice

`dicomflux-3d` (`0.1.0`, `FS-01`) is an engineering software library component and is **not a medical device**. It is **not** cleared, certified, or qualified for clinical diagnostic interpretation, patient-care decision making, or calibrated diagnostic display (`DisplayEncoding::UncalibratedSrgbU8`).
