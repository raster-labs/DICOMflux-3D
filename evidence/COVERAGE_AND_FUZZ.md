# Adversarial Fuzzing, Fault Injection & Boundary Coverage (`FS-01`)

## 1. Deterministic `SplitMix64` Fuzzing (`FS01-T16`)

- **Test Entry**: `tests/fuzz_and_governance.rs::fs01_t16_seeded_splitmix64_descriptor_and_voi_fuzz`
- **Generator & Seed**: Stable-Rust `SplitMix64` seeded with `0xDF3D_F501_2026_1110` (zero nightly compiler or C++ `libfuzzer-sys` runtime required per `FS01-R01`)
- **Iterations Executed**: `100000`
- **Replay Command**: `cargo test --locked --test fuzz_and_governance fs01_t16_seeded_splitmix64_descriptor_and_voi_fuzz`

### Adversarial Mutation Domains & Asserted Invariants

1. **Frame Admission & Backing Slices (`admit_frame`)**: Randomised `0`, boundary, and `u32::MAX` dimensions; odd/short/`usize::MAX` `byte_offset` and `row_stride_bytes`; mismatched `retained_capacity_bytes < logical_byte_len`; `SampleType::{I16, U16, U8, I32, F32}`; `ColorModel::{Grayscale, PaletteIndexed, Color}`; `frame_count` in `{0, 1, >=2}`; non-canonical `BitPrecision`; dirty upper sign/zero bits on sub-16-bit words; and out-of-container or inverted `PixelPadding` under tight `ResourceLimits` (`retained_source_bytes = 256`, `cpu_output_bytes = 128`, `max_frame_dim = 16`).
2. **VOI & Modality Rescale (`render_reference`)**: `NaN`, `±Inf`, subnormal (`±1e-310`), `±1e200`, `f64::MAX`, `width < 1.0`, `width == 1.0`, zero/negative slopes, `LinearExact`, `Sigmoid`, `UnsupportedLutSequence`, `UnsupportedModalityLutSequence`, and pre-set `CancellationToken`.
3. **2D `PatientPlane` & 3D `VolumeDescriptor`**: `NaN`/`±Inf`/`±1e200` origins and cosines, zero/negative spacings, non-unit and non-orthogonal vectors, invalid homogeneous bottom rows, singular matrices, and gantry-sheared off-diagonal terms, asserting `< 1e-9` forward/inverse round-trips on well-conditioned admits.
4. **Zero-Panic & Zero-Leak Guarantee**: Across all `100,000` iterations, zero panics occurred and `PoolUsageSnapshot` returned to `0` active bytes across all pools.

## 2. White-Box & Public GPU Fault Injection Coverage (`FS01-T09`, `FS01-T13`, `FS01-T15`)

- **Exact Pool Ceilings (`limit` vs. `limit + 1`)**: Verified across `RetainedSource`, `GpuStagingUpload` (256-byte row-aligned), `GpuResident` (source + transient `R8Uint` target with retire-before-replace eviction), `GpuStagingReadback`, `CpuOutput`, `VolumeBytes`, `FrameDimension`, `VolumeDimension`, and `InFlightJobs`.
- **Simulated GPU Out-of-Memory (`GpuOutOfMemory`) & Device Loss (`GpuDeviceLost`)**: Verified in `src/rendering/gpu/mod.rs` unit tests that uncommitted candidate residency slots are discarded, terminal `FaultLatch` prevents post-loss usage, and zero partial `RenderedImage` outputs are emitted.
- **Post-Submit Cooperative Cancellation (`InFlightLedger`)**: Verified that cancelling after `queue.submit` returns `Err(RenderError::Cancelled)` immediately while retaining the submitted buffers, target texture, `Arc<ResidentGpuFrame>`, `DecodedFrameLease`, and `RetainedSource` `Arc<BudgetGuard>` until `on_submitted_work_done` + `poll_reap()` confirm hardware completion.

## 3. Fuzzing Scope Limits

- `FS01-T16` fuzzes Rust public API descriptors, buffer leases, geometry matrices, and presentation parameters; GPU driver shader compiler fuzzing across vendor Vulkan/D3D12 stacks requires target workstation execution (`FS01-T19`–`FS01-T21`).
