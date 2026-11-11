# 02 — FS-01 Software Requirements Specification

**DICOMflux-3D first slice:** Checked 2D greyscale presentation + reusable checked geometry contracts  
**Version/date:** 0.2.0 / 10 November 2026  
**Status:** Requirements for separately authorised coding-agent implementation; **all test execution pending**.

## 1. Entry, objective and acceptance scope

FS-01 proves the actual Rust imaging/presentation boundary before 3D complexity. An independent Rust executable SHALL construct a synthetic or rights-cleared checked 2D frame without opening a DICOM file; invoke a CPU reference windowing path and a wgpu/WGSL offscreen path; produce comparable, explicitly described 8-bit greyscale presentation; and record adapter/resource failures without crashing or corrupting output. A separately testable 3D geometry descriptor SHALL verify dimensional integrity and a caller-supplied patient LPS affine, **without** attempting DICOM series assembly or GPU volume rendering.

FS-01 depends conceptually on the DICOMflux shared output contract but **does not wait for the DICOMflux Rust parser repository to be complete**. It MUST not introduce an independent DICOM parser, codec or DICOM media reader. Any future DICOMflux adapter is a separately qualified addition and must not contradict the generic toolkit's interface requirements.

## 2. Normative terms and ID convention

`MUST` = hard requirement; `SHOULD` = recommended default with recorded reason for variation; `MAY` = optional, separately labelled. `FS01-Rnn` identifies an implementation requirement; `FS01-Tnn` a **planned** verification test. No test in this SRS is claimed to have run. `Admitted` means validated under the first-slice profile, **not** clinically suitable.

## 3. Required input profile

- Single-plane greyscale only; one numeric component per pixel, decoded from an external producer. Mandatory `u16` and `i16`; an optional explicit `u8` profile MAY be added if it does not compromise tests or memory contracts. Unsupported RGB, palette, float, signed 32-bit or multicomponent formats reject until separately admitted.
- Input dimensions strictly positive and bounded by `ResourceLimits`; checked `width * height`, row stride and byte extent; no arithmetic wrap, unvalidated cast or trust in externally supplied pointer length.
- `BitsAllocated`, `BitsStored`, `HighBit`, `PixelRepresentation` information MUST be validated as appropriate for the supplied canonical decoded samples, with explicit sign handling and non-significant-bit policy. A producer MAY supply already-normalised decoded values only under a documented contract.
- An immutable owner keeps source pixels alive for every in-flight CPU/GPU read. Borrowed views cannot outlive the allocation; `Send`/`Sync` follow actual Rust safety guarantees rather than declarations by fiat.
- A finite linear rescale `slope` and `intercept` MAY be applied to form modality values. Reject non-finite parameters and unsupported nonlinear Modality LUTs; never mutate source samples. Declare the value domain and unit (e.g. HU when actually supported by metadata).
- Required VOI function is DICOM **LINEAR**, not a generic contrast slider. Additional functions `LINEAR_EXACT` and `SIGMOID` are optional *capability-gated* profiles. Explicitly reject a request requiring a non-admitted function, colour palette or table VOI LUT. Document centre/width units and full DICOM edge behaviour (`width=1` threshold for LINEAR).
- Required output is a tightly described 8-bit greyscale buffer, with declared row stride, polarity and sRGB/non-calibrated display assumption. Never claim PS3.14 GSDF, HDR or diagnostic display calibration. A GPU texture-only result may be added but must be independently verifiable through offscreen readback.

## 4. Requirements and traceability

| ID | Requirement | Required verification |
|---|---|---|
| FS01-R01 | Provide a real, MIT-licensed Rust crate with documentation, pinned toolchain and lockfile; no Swift/C++/DICOM parser dependency in the minimal feature graph | Build and dependency inspection; source licences |
| FS01-R02 | Expose checked decoded-frame descriptors, ownership, precision, dimensions and strides; reject invalid states without panics | Constructive/negative unit tests, fuzz entry |
| FS01-R03 | Keep source pixels immutable and separate canonical/modality/presentation representations | Source hashes before/after, API review |
| FS01-R04 | Validate caller-supplied `f64` LPS geometry for a 2D plane and a regular synthetic `CheckedVolume`; do not infer slice spacing | Geometry phantom/invalid-affine tests |
| FS01-R05 | Implement deterministic CPU DICOM LINEAR W/L over admitted samples with boundary/rounding conventions documented | Analytic VOI oracle |
| FS01-R06 | Apply a finite rescale slope/intercept in the correct stage and preserve signedness, stored precision and padding semantics | Signed 12-bit, intercept, padding cases |
| FS01-R07 | Implement wgpu/WGSL offscreen LINEAR W/L for admitted GPU path with explicit adapter and format checks | CPU/GPU result comparison |
| FS01-R08 | Keep UI, egui, winit and DICOM file I/O out of the public library's minimal dependency closure | `cargo tree`, independent consumer, headless smoke |
| FS01-R09 | Repeated window changes MUST NOT trigger DICOM re-decoding or irreversible mutation; report material GPU uploads separately | Source hash and instrumentation |
| FS01-R10 | Return capability report before work: available backend, format, max input/output, excluded profiles and reasons | Simulated unsupported-capability tests |
| FS01-R11 | Admit all CPU/GPU/staging allocations under checked per-job/host budgets; reject oversize input before allocating | Budget limits/overflow and failure injection |
| FS01-R12 | Define cancellation, GPU loss, resource release, result completion and error categories; no success after partial failure | Device-loss/cancel harness and unit tests |
| FS01-R13 | Produce offscreen render/readback with explicit output shape, stride, pixel order and presentation provenance | Round-trip, image hash, metadata inspection |
| FS01-R14 | Record exact toolchain, target triple, adapter/driver, WGSL/wgpu revisions, test case versions and actual executed result evidence | Reproducible evidence bundle |
| FS01-R15 | No clinical-use, universal DICOM compatibility, full 3D capability or pixel-parity claims beyond executed tests | Documentation/README audit |
| FS01-R16 | Build an independent sample consumer outside workspace exposing minimal CPU and GPU paths | Consumer receipt and integration log |
| FS01-R17 | Enforce separation from DICOMflux/Flux core using dependency graph assertions or CI checks | Automated layering verification |
| FS01-R18 | Detect unsupported Modality LUT, VOI LUT, multiframe and malformed sample description explicitly; never silently treat as supported | Negative profile cases |

## 5. Presentation mathematical contract

For DICOM LINEAR on modality value `x`, window centre `c`, width `w >= 1`, `ymin=0` and `ymax=255` for FS-01, the reference implementation SHALL use DICOM PS3.3's half-sample convention:

```text
if x <= c - 0.5 - (w - 1) / 2: y = 0
else if x > c - 0.5 + (w - 1) / 2: y = 255
else: y = ((x - (c - 0.5)) / (w - 1) + 0.5) * 255
```

For `w=1`, the middle branch is unreachable; code must avoid division by zero. The conversion from theoretical `y` to an integer display code MUST fix a rounding policy and test both endpoints and representative midpoints. `c`/`w` and `slope`/`intercept` are finite; `w<1` is rejected for LINEAR. For optional `LINEAR_EXACT` and `SIGMOID`, source their exact equations and validation from the pinned DICOM edition and add independent oracles before advertising those modes.

Pixel Padding Value and Range Limit are input metadata concerns handled by declared display policy; pad pixels MUST NOT be inadvertently transformed into meaningful anatomy or counted in later statistics. FS-01's default proposal is a configurable background code for identified padding, with masking tested independently from VOI. DICOM specific Presentation LUT, GSPS and PS3.14 display calibration are not in FS-01.

**Precision contract:** authoritative CPU reference may compute in `f64`; GPU shader uses supported WGSL numeric formats, commonly `f32`. Each test declares exact expected output or an explicitly justified 8-bit maximum deviation (recommended <=1 code in non-boundary cases); boundary and threshold cases must have their own strict expected outcomes. Do not hide failures behind an unbounded visual tolerance.

## 6. Minimal spatial contract

A synthetic 2D frame admits row/column orientation unit vectors in LPS, pixel spacing in mm and position of the centre of index (0,0). Validate finite positive spacing, finite position, nonzero orientation vectors, normalisation/orthogonality within a declared tolerance, handedness and a non-singular mapping. If a validated DICOMflux affine is received, preserve its provenance and coordinate convention. The public API must make index-to-patient conversion and the inverse availability explicit; use `f64` for authoritative geometry and no patient-coordinate truncation to `f32` as storage.

For a regular `CheckedVolume` descriptor, validate positive dimensions, byte extent, a full regular `f64` 4×4 affine, nonzero determinant, finite values, named axis-to-patient semantics and frame/source references. The descriptor's validation does not construct or upload a 3D texture in FS-01.

## 7. Explicit FS-01 exclusions

- Parsing Part 10/DICOM datasets, encapsulated pixel fragments, DICOM transfer syntax selection or codecs.
- Series assembly, irregular slice resampling, enhanced multiframe selection, temporal 4D, mixed modalities or colour images.
- Viewport UI/controls, DICOM Capacitor integration, Android Kotlin/JNI or browser/WASM adapter.
- MPR, MIP, MinIP, DVR, volumetric transfer functions, surface extraction, Marching Cubes, parry3d, segmentation/SEG export or registration.
- PS3.14 diagnostic calibration, presentation-state authoring, DICOM SR evidence or any clinical qualification.

## 8. Future interfaces preserved without premature implementation

Validated `CheckedVolume`, patient-coordinate transforms, quality/request flags and resource budgets leave room for FS-02 MPR, FS-03 MIP/DVR and FS-04 threshold masks. **Do not publish placeholder traits, comprehensive feature enumerations or empty modules** unless they are directly needed by FS-01; documenting a forward-compatibility seam is sufficient.

## 9. Test, delivery and stop conditions

FS-01 admission requires all mandatory R01–R18 tests and checks to be executed and recorded, including one actual Windows x64 and one actual Linux x64 build/test path where qualified hardware is available. ARM64 targets and supported graphics paths require separate evidence before any ARM64 qualification claim. Cross-compilation is not execution. A developer may scope the initial execution to fewer machines, but must mark remaining primary-platform evidence **OPEN**, not silently declare release readiness.

Stop and escalate if authoritative DICOM semantics conflict with these proposed requirements, a required dependency has incompatible provenance, the owner-selected new repository cannot be identified, or the output requires a change to DF3D-DEC01–09 or the common manifest. Otherwise continue unaffected work, documenting deviations, rather than restarting the architecture exercise.
