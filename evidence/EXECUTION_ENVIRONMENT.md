# Execution Environment & Capability Receipt (`FS-01`)

- **Execution Date (UTC)**: `2026-11-11`
- **Source Revision (`HEAD`)**: `7ed0d8d5fa67414fc4a6b66c2d04322d4236de55`
- **Host Target Triple**: `aarch64-apple-darwin`
- **CPU Architecture / Model**: `Apple M5`
- **Host Qualification Tier**: `QualificationTier::BoundedDevelopmentHost`
- **`wgpu` Crate Version**: `30.0.1`
- **WGSL Shader (`src/rendering/gpu/voi_linear.wgsl`) SHA-256**: `b4b0ad1b6fd0a1b24555d83544d1bfe4390bc11e296a674c7ec7a66818383701`

## 1. Operating System Summary

```text
ProductName:		macOS
ProductVersion:		27.0.1
BuildVersion:		26A434
```

## 2. Rust Toolchain (`rustc -vV` & `cargo -V`)

```text
rustc 1.97.1 (8bab26f4f 2026-07-14)
binary: rustc
commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452
commit-date: 2026-07-14
host: aarch64-apple-darwin
release: 1.97.1
LLVM version: 22.1.6
cargo 1.97.1 (c980f4866 2026-06-30)
```

## 3. Structured `probe_capabilities` Report (Privacy-Redacted)

- **`cpu_reference_available`**: `true`
- **`gpu_adapters` count**: `1`
  - Adapter `0`: `backend = Metal`, `device_type = IntegratedGpu`, `qualification_tier = BoundedDevelopmentHost`, `max_texture_dimension_2d = 32768`, `max_buffer_size = 4294967295`, `supports_16bit_grayscale_offscreen = true`
- **`excluded_profiles`**:
  - `multiframe_inputs (frame_count > 1): UnsupportedPixelFormat`
  - `non_16bit_sample_containers (u8, i32, f32): UnsupportedPixelFormat`
  - `color_and_palette_photometrics: UnsupportedPresentation`
  - `voi_linear_exact_and_sigmoid: UnsupportedPresentation`
  - `nonlinear_modality_and_voi_lut_sequences: UnsupportedPresentation`
  - `gsps_and_gsdf_calibrated_display: UnsupportedPresentation`
  - `mpr_mip_dvr_3d_volume_rendering: deferred_beyond_fs01`
  - `segmentation_overlays_and_surface_meshes: deferred_beyond_fs01`
  - `window_surfaces_and_browser_webgpu: deferred_beyond_fs01`

> **Qualification Notice**: Execution on `aarch64-apple-darwin` stamps `QualificationTier::BoundedDevelopmentHost`. It verifies mathematical parity, quota enforcement, fault isolation, and standalone integration on Apple Silicon Metal, and does **not** substitute for physical Windows `x86_64` (`FS01-T19`), Linux `x86_64` (`FS01-T20`), or `ARM64` target (`FS01-T21`) workstation runs.
