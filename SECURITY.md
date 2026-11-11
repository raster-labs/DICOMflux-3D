# Security and Privacy Policy

## Supported Scope

`dicomflux-3d` (`0.1.0`, First Slice `FS-01`) processes caller-supplied in-memory pixel buffers and spatial/presentation descriptors. It performs no network I/O, spawns no external services, and parses no DICOM files or compressed codecs.

## Memory and Bounds Safety Guarantees

- **Zero Unsafe Rust**: `#![forbid(unsafe_code)]` is enforced at the crate root and verified by automated governance tests (`FS01-T18`).
- **Fail-Closed Checked Arithmetic**: Every frame plane extent, byte offset, row stride, 256-byte GPU copy pitch, and 3D voxel/byte count is validated using checked integer arithmetic (`checked_add` / `checked_mul`) prior to memory reservation or allocation (`FS01-R02`, `FS01-R11`).
- **Pre-Allocation Resource Quotas**: `ResourceTracker` enforces hard limits across retained source buffers, GPU upload staging, GPU resident textures, GPU readback staging, CPU output buffers, volume byte ceilings, frame/volume dimensions, and concurrent in-flight jobs before allocating host or GPU resources.
- **Fault Isolation**: GPU validation and out-of-memory errors are captured inside scoped error guards and converted into typed `RenderError` variants without panicking or returning partial images.

## Protected Health Information (PHI) Redaction Invariants

`dicomflux-3d` enforces privacy-by-construction across all public types, receipts, and error paths (`FS01-R15`, `FS01-T23`):

- Public contracts accept numeric `SourceIdentity` tokens `(NonZeroU64, u64)` and never store patient names, patient IDs, birth dates, study/series/SOP Instance UIDs, or filesystem paths.
- Manual `fmt::Debug` and `fmt::Display` implementations on `DecodedFrameLease`, `CheckedFrame`, `LpsPointMm`, `PatientPlane`, `CheckedVolume`, `RenderedImage`, `GpuCapabilities`, `GpuContext`, and `RenderError` redact raw pixel payloads, patient LPS origin coordinates, and raw GPU driver strings.
- Generated verification logs scrub local user home-directory path prefixes before writing evidence files.

## Reporting a Vulnerability

Please do **not** open public issues containing exploit payloads or any Protected Health Information (`PHI`). Report suspected security or privacy vulnerabilities privately to:

- **Security Contact**: `ranjithkumar.k@raster.in`
- **Advisory Channel**: Private GitHub Security Advisory under the `raster-labs` organization repository once published.

Include the crate version, commit SHA, target OS/architecture, minimal synthetic reproduction steps, and expected vs. observed behaviour.
