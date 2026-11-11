# 03 — Data, API and Execution Contracts

**Version/date:** 0.2.0 / 10 November 2026  
**Status:** Design contract for FS-01 implementation; type names and Rust signatures are **illustrative**, not a frozen API.

## 1. Seam design rules

Keep the Rust API idiomatic and narrow. It must accept checked decoded scalar samples and DICOM-specific metadata **from the caller**; it must not own datasets, decompression, studies or GUI widgets. It may know DICOM patient LPS and presentation semantics because that is its distinct remit. A future DICOMflux adapter constructs input from parsed DICOM; the renderer should accept equivalent test data without linked DICOMflux parser. Where DICOMflux later supplies the same shared types, reuse them or provide a zero/low-copy adapter with written compatibility tests instead of inventing a second competing public contract.

## 2. Proposed conceptual types

| Type / concept | Fields and invariants | Owned by |
|---|---|---|
| `DecodedFrameLease` | Stable owner, slice access, total byte extent; no use after release/in-flight completion | Producer/consumer lease |
| `FrameDescriptor` | width, height, row stride, sample kind `i16/u16`, precision/bit layout, one component, source identity | Checked DICOMflux adapter or test harness |
| `IntensityTransform` | identity or finite linear slope/intercept, modality unit, value stage | DICOM presentation metadata adapter |
| `PatientPlane` | origin, row/column directions, spacing, LPS mm, `f64` conversions | Validated shared contract |
| `CheckedFrame` | Immutable validated descriptor + lease + intensity + orientation/padding/presentation metadata | DICOMflux-3D admitted input |
| `CheckedVolume` | Positive 3D shape/strides, finite `f64` affine, regularity provenance, source lease | DICOMflux-3D admitted input |
| `VoiRequest` | function, finite centre, width, padding/background and polarity policy | Caller request |
| `OutputDescriptor` | width, height, format/bit depth, stride, colour/polarity policy | DICOMflux-3D |
| `GpuCapabilities` | backend, adapter, vendor/driver when available, features/limits, supported texture/render paths | DICOMflux-3D |
| `ResourceLimits` | source/staging/resident/output/in-flight budgets and max shape; checked arithmetic | Caller + DICOMflux-3D |
| `RenderReceipt` | operation id, input provenance, VOI parameters, path, rounding/tolerance policy, result status and timings | DICOMflux-3D |
| `RenderError` | invalid input, unsupported profile, budget exceeded, no adapter, device lost, cancellation, OOM, internal failure | DICOMflux-3D |

## 3. Illustrative safe Rust API sketch (not shipping code)

```rust,ignore
// Design sketch only. Agent must reconcile naming and source versions.
pub struct FrameDescriptor { /* validated dimension/precision/stride */ }
pub struct CheckedFrame<'a> { /* immutable owner/borrow + metadata */ }
pub struct CheckedVolume<'a> { /* regular affine geometry; FS-01 validation only */ }
pub struct VoiRequest { /* function, centre, width, polarity */ }
pub struct ResourceLimits { /* bounded allocations and in-flight work */ }
pub enum PresentationPath { CpuReference, GpuOffscreen }
pub struct RenderReceipt { /* metadata and provenance */ }
pub struct RenderedImage { /* bytes, width, height, stride, receipt */ }

pub fn admit_frame<'a>(
    input: FrameDescriptor,
    source: &'a [u8],
    limits: &ResourceLimits,
) -> Result<CheckedFrame<'a>, RenderError>;

pub fn render_reference(
    frame: &CheckedFrame<'_>,
    request: &VoiRequest,
    limits: &ResourceLimits,
) -> Result<RenderedImage, RenderError>;

// GPU context is created explicitly; this does not imply a winit/egui window.
pub async fn render_offscreen(
    context: &mut GpuContext,
    frame: &CheckedFrame<'_>,
    request: &VoiRequest,
    limits: &ResourceLimits,
) -> Result<RenderedImage, RenderError>;
```

The final design must choose **one** clear owner policy (`&[u8]` with lifetime, `Arc<[T]>`, or an explicit source lease) and document how GPU submissions retain valid source/storage until completion. Never expose a dangling borrowed slice through an async task; copied staging, command submission and `map_async` lifetime must be explicit. The sample signature above may be revised to enforce that guarantee. Do not transmute a foreign pointer to a `'static` borrow.

## 4. Data stages (separate authority and preview)

```text
Decoded input samples (unchanged, with ownership/provenance)
          |
          v
Modality values (rescale/Modality LUT, as supported)
          |
          v
VOI presentation transform (LINEAR required in FS-01)
          |
          v
Photometric/polarity and declared presentation transform
          |
          v
8-bit offscreen preview + receipt (NOT a new quantitative image)
```

The renderer may cache GPU representations keyed to source and intensity metadata. Changing only W/L changes a uniform/lookup, not the source. Cache identity includes source content/generation, format, mapping, transform, device and relevant quality flags. No source hash of patient pixel data should be transmitted or logged externally; store only privacy-safe identifiers/evidence locally and redact patient information.

## 5. Image geometry conversion

Let `P0` be LPS position of the centre of pixel (column=0,row=0), `R` the direction along increasing columns, `C` the direction along increasing rows, `dr` row spacing and `dc` column spacing. Then:

```text
P(column,row) = P0 + column * dc * R + row * dr * C
```

Follow DICOM's row/column and spacing conventions rather than assuming first spacing is the X axis. For a regular 3D affine, the third basis comes from validated differences in slice positions with consistent orientation and spacing; do not substitute nominal thickness. Use precise/f64 transforms for reference and measured results, convert deliberately for GPU camera-relative coordinates.

## 6. Ownership, cancellation and completion

A synchronous CPU operation returns an owned complete output or error. For GPU paths, the input owner/lease, staging allocation, texture, bind groups, command buffers and output mapping must remain valid across submission and completion. Cancellation can stop admission/queueing, but cannot prematurely free or declare finished an already submitted GPU operation. `RenderReceipt` is committed only when output is valid and complete. A context/device loss invalidates dependent resources, returns an explicit outcome and permits recovery only through a new documented context lifecycle.

Set bounded outstanding task count and memory budgets; distinguish current resident memory from total allocations in flight. Support explicit resource teardown. No global implicit GPU instance in library import; no hidden network or telemetry. Host applications own event-loop and scheduling policy.

## 7. Error and capability schema (candidate)

```text
InvalidDescriptor        (bad shape, strides, precision, affine or data length)
UnsupportedPixelFormat   (unadmitted scalar/component representation)
UnsupportedPresentation  (unadmitted LUT, mode, photometric or calibration profile)
UnsupportedGeometry      (irregular/unvalidated grid or coordinate convention)
ResourceLimitExceeded    (bounded admission refusal)
NoSuitableAdapter        (no matching GPU device under policy)
UnsupportedGpuCapability (format/feature/surface limitation)
GpuDeviceLost            (device no longer usable)
GpuOutOfMemory           (allocation/submission failure)
Cancelled                (request intentionally stopped; no complete output)
InternalError            (non-sensitive diagnostic; provenance local)
```

Do not report unsupported GPU access as DICOM parse failure. Error logs must not include patient identifiers, image content, sensitive paths, study UIDs by default, or unbounded driver messages. Driver details may be included in a privacy-reviewed local technical report.

## 8. Output and host interop

First slice output is a complete explicitly sized 8-bit offscreen image plus receipt. Expert GPU-texture handoff is deferred until multi-device, surface lifetime, queue/fence and host graphics API compatibility are demonstrated. A later versioned C ABI may expose opaque context/request/lease handles, fixed-width integers and explicit release functions to Java FFM/host adapters; no Rust `Vec`, references, panics or `wgpu::Texture` may cross the ABI. The UI layer supplies input commands and layout. DICOMflux-3D remains independent of Java/Swing, egui and winit.

## 9. Interface acceptance checks

- Compile a standalone consumer outside the Cargo workspace without importing a DICOM parser or GUI.
- Admit a small `i16` synthetic frame with signed 12-bit semantics and finite rescale, then compare CPU/GPU 8-bit output to a pinned oracle.
- Prove output provenance includes source revision/identity, modality, W/L, mode, polarity, device capability and error policy.
- Show invalid dimensions, overflow, illegal width/centre, unadmitted VOI function, and excessive budgets fail without a partial success.
- Prove swapping `VoiRequest` leaves source bytes unchanged; verify no redundant DICOM decode is invoked (there is none in this library).
- Reject irregular `CheckedVolume` construction before any GPU allocation.
