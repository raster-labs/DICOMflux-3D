---
trigger: always_on
description: "Cached project architecture map, key entry points, commands, and conventions for DICOMflux-3D."
---

# DICOMflux-3D Project Context (`FS-01`)

## 1. Tech Stack & Runtime
- **Crate**: `dicomflux-3d` (`0.1.0`, Rust Edition `2024`, `rust-version = "1.97"`, pinned `rust-toolchain.toml` = `1.97.1`, MIT licence, `Copyright (c) 2026 Raster Images`).
- **Safety & Lints**: `#![forbid(unsafe_code)]`, `#![deny(missing_debug_implementations)]`, `clippy::all = "deny"`.
- **Dependencies**:
  - `[dependencies]`: `wgpu = { version = "30.0.1", default-features = false, features = ["std", "parking_lot", "wgsl", "metal", "vulkan", "dx12"] }`, `sha2 = "0.11"`.
  - `[dev-dependencies]`: `serde_json = "1"`.
  - Prohibited across transitive runtime closure: DICOM parsers, codecs, `egui`/`winit` UI toolkits, `glam`/`ndarray`, `bytemuck`, `tokio`/`pollster`, C++/Swift bridges, VTK.
- **Target Platforms**: Primary workstation targets are Windows `x86_64` (`Dx12`/`Vulkan`, `FS01-T19`) and Linux `x86_64` (`Vulkan`, `FS01-T20`), plus `ARM64` (`FS01-T21`). Local macOS Apple Silicon (`Metal`) runs stamp `QualificationTier::BoundedDevelopmentHost`.

## 2. Architecture & Key Directories
- `docs/designs/fs01-foundation.md`: Approved EGM technical specification and 40-target `[Create]` implementation plan (`FS01-R01`–`R18`, `FS01-T01`–`T24`).
- `docs/handoff/`: Sealed authority specifications (`DICOMflux-3D` `v0.2.0`, `DICOMflux` Manifesto `v1.3.1` & `D0`, `Flux Imaging Suite` `F0`). Never modify baseline `docs/handoff/DICOMflux-3D/FS01_Test_Case_Manifest_v0.2.0.json`.
- Implemented `src/` modules (7 private modules re-exported selectively via `src/lib.rs`):
  - `error.rs`: 11-variant `RenderError` + 9-variant `LimitPoolKind` with redacted `Debug`/`Display`.
  - `resource.rs`: `ResourceLimits`, `ResourceTracker` (owns `ResourceLimits`), `pub(crate) BudgetGuard`, `PoolUsageSnapshot`, `OperationId`, `CancellationToken`.
  - `image.rs`: `SourceIdentity`, 3-variant safe `DecodedFrameLease` (`Bytes`/`WordsI16`/`WordsU16`), `FrameDescriptor` (with `ColorModel` & container-domain `PixelPadding`), `CheckedFrame`, `admit_frame`.
  - `spatial.rs`: `f64` LPS `LpsPointMm`, `PixelIndex2D`, `PatientPlane` (`2 x 2` Gram-matrix inverse).
  - `volume.rs`: Constant-memory `f64` regular 3D `VolumeDescriptor`, `CheckedVolume`, `admit_volume`.
  - `presentation.rs`: `IntensityTransform`, `VoiParameters`, `VoiRequest`, `DisplayEncoding::UncalibratedSrgbU8`, `OutputDescriptor`, `RenderReceipt`, `RenderedImage`.
  - `rendering/`: `cpu.rs` (`render_reference` `f64` oracle) and `gpu/` (`capabilities.rs`, `pitch.rs`, `uniforms.rs`, `voi_linear.wgsl` [SHA-256 `b4b0ad1b6fd0a1b24555d83544d1bfe4390bc11e296a674c7ec7a66818383701`], `pipeline.rs`, `mod.rs` — quarantined `wgpu`).
- `tests/`: `37` workspace tests (`18` lib unit + `8` `frame_and_voi_oracle` + `4` `fuzz_and_governance` + `5` `gpu_parity_and_faults` + `2` `spatial_and_volume`).
- `examples/independent-consumer/`: Outside-workspace consumer crate (`[workspace]` excluded, `1` end-to-end test) for `FS01-T01`.
- `examples/generate_evidence.rs` & `evidence/`: Reproducible benchmark (`FS01-T22`) and SHA-256-manifested evidence bundle (`21 / 24` `PASS` on `BoundedDevelopmentHost`; `FS01-T19`–`T21` `NOT_RUN` `OPEN_PENDING_TARGET_HARDWARE`).

## 3. Build, Test, Lint & Evidence Commands
Prepend Homebrew `rustup` path on macOS when needed:
```zsh
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo test --locked --manifest-path examples/independent-consumer/Cargo.toml
cargo run --locked --manifest-path examples/independent-consumer/Cargo.toml
cargo run --release --locked --example generate_evidence
shasum -a 256 -c evidence/SHA256SUMS.txt
```

## 4. Core Conventions, Invariants & Gotchas
- **Zero PHI in Diagnostics**: Never include patient names, UIDs, file paths, raw pixel bytes, LPS origins, or raw `wgpu` driver strings in `Debug`/`Display` or logs.
- **Canonical 16-Bit & Container Padding**: Enforce `bits_allocated == 16`, `1 <= bits_stored <= 16`, `high_bit == bits_stored - 1`; exempt declared `FrameDescriptor::pixel_padding` container sentinels (e.g. `-3024` in 12-bit CT) during upper-bit checks.
- **WGSL / `wgpu 30.0.1` Specifics**: Bind `src_sint` at `@binding(1)` (`wgpu::TextureSampleType::Sint` unit variant) and `src_uint` at `@binding(2)` (`Uint`); set `constant_mode = 1` strictly when `slope == 0.0`; clamp `!scale_f32.is_finite()` to `0.0`; use out-of-range clamp sentinels `-32769` / `65536`; pop `ErrorScopeGuard` in reverse LIFO order; treat `Err(wgpu::PollError::Timeout)` as wait-loop continuation; populate `COPY_SRC` and `64`-byte `UNIFORM` buffers via `mapped_at_creation: true`.
- **Single-Slot Residency & Cancellation**: Key `ResidentGpuFrame` on `(SourceIdentity, FrameDescriptor)`, evict before reserving new `GpuResident` quotas, commit after clean error-scope checks, and retain cancelled in-flight resources via `Arc<BudgetGuard>` + `Queue::on_submitted_work_done` (`poll_reap()`).
- **Governance & Git**: Work directly on `main` (annotated tags for milestones); authorship is independent implementation with documented provenance (never claim formal clean-room; external contributions require executed copyright assignment to Raster Images).
