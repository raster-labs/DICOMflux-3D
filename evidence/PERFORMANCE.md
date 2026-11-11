# Performance Baseline & Peak Memory Telemetry (`FS01-T22`)

## 1. Reproducible Methodology

- **Command**: `cargo run --release --locked --example generate_evidence`
- **Source Revision**: `7ed0d8d5fa67414fc4a6b66c2d04322d4236de55`
- **Host & Adapter**: `aarch64-apple-darwin` (`Apple M5`), `Metal/IntegratedGpu/BoundedDevelopmentHost`
- **Protocol**: `5` unmeasured warmup iterations followed by `20` measured iterations per operation. GPU operations synchronize through full `device.poll` `MAP_READ` readback unpacking into tight `RenderedImage` output buffers. Summary statistics report sample `min`, `p50` (median of 20), nearest-rank `p95`, and `max` in microseconds (`us`), plus `PoolUsageSnapshot` high-water marks in bytes.
- **Scope & Honesty Notice**: Measurements below are baseline engineering observations from a `QualificationTier::BoundedDevelopmentHost` host. No frame-rate (`FPS`) guarantees, millisecond SLAs, zero-copy shared-memory claims, or primary Windows/Linux workstation claims are made.

## 2. Cold `GpuContext::new` Initialization Latency

| Operation | Warmup | Measured | `min` (`us`) | `p50` (`us`) | `p95` (`us`) | `max` (`us`) |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| `GpuContext::new` (adapter select + device + WGSL pipeline compile) | 5 | 20 | 542.0 | 593.7 | 747.9 | 1011.0 |

## 3. Per-Fixture Presentation Latency & Stage Medians

### 3.1 Small Greyscale Frame (`512 x 512` signed 12-bit `i16`, CT HU rescale)

- **Format**: `i16 (BitsAllocated=16, BitsStored=12, HighBit=11, PixelPadding=-2000)` (`524288` source bytes, source SHA-256 `c9f0c0ac9ca433afcb2685bc95d44ae94d8fba7da331bd2ec85957d959c27b2f`)
- **CPU vs. GPU Pixel Agreement**: `max_abs_diff = 0` LSB (`<= 1` LSB tolerance, `0` LSB on padded border)

| Path | Warmup | Measured | `min` (`us`) | `p50` (`us`) | `p95` (`us`) | `max` (`us`) |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| Cold GPU first-frame upload + render (`source_cache_hit = false`) | 5 | 20 | 404.3 | 434.9 | 627.2 | 940.4 |
| Warm GPU repeated W/L render (`source_cache_hit = true`, `0` upload bytes) | 5 | 20 | 326.6 | 396.0 | 704.4 | 765.0 |
| Deterministic `f64` CPU reference (`render_reference`) | 5 | 20 | 529.9 | 575.5 | 663.2 | 687.5 |

#### GPU `StageTimings` Median (`p50`) Breakdown (`us`)

| GPU Mode | `validation` (`us`) | `host_encode_and_submit` (`us`) | `gpu_wait_and_unpack` (`us`) | `receipt.total` (`us`) |
| :--- | ---: | ---: | ---: | ---: |
| Cold upload + render | 0.2 | 108.8 | 332.5 | 434.8 |
| Warm repeated W/L | 0.1 | 43.0 | 355.6 | 395.9 |

#### Tracked `PoolUsageSnapshot` Peak Memory Bytes

| Pool | Peak Reserved Bytes / Count | Active After Drop |
| :--- | ---: | ---: |
| `retained_source_bytes` | 524288 | 0 |
| `gpu_staging_upload_bytes` | 524288 | 0 |
| `gpu_resident_bytes` (source `R16` + transient `R8Uint` target) | 786432 | 0 |
| `gpu_staging_readback_bytes` | 262144 | 0 |
| `cpu_output_bytes` | 786432 | 0 |
| `in_flight_jobs` | 1 | 0 |

### 3.2 Medium Greyscale Frame (`2048 x 2048` unsigned 16-bit `u16`, Identity rescale)

- **Format**: `u16 (BitsAllocated=16, BitsStored=16, HighBit=15, PixelPadding=0)` (`8388608` source bytes, source SHA-256 `876e4593d0eb3decfe4d36b9ff86d738f6966997c17f1acffbf4a14c4e7b7ff5`)
- **CPU vs. GPU Pixel Agreement**: `max_abs_diff = 0` LSB (`<= 1` LSB tolerance, `0` LSB on padded border)

| Path | Warmup | Measured | `min` (`us`) | `p50` (`us`) | `p95` (`us`) | `max` (`us`) |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| Cold GPU first-frame upload + render (`source_cache_hit = false`) | 5 | 20 | 2019.0 | 2254.8 | 3031.8 | 3032.1 |
| Warm GPU repeated W/L render (`source_cache_hit = true`, `0` upload bytes) | 5 | 20 | 742.8 | 889.6 | 1020.0 | 1032.5 |
| Deterministic `f64` CPU reference (`render_reference`) | 5 | 20 | 9575.5 | 9789.6 | 10414.3 | 10563.1 |

#### GPU `StageTimings` Median (`p50`) Breakdown (`us`)

| GPU Mode | `validation` (`us`) | `host_encode_and_submit` (`us`) | `gpu_wait_and_unpack` (`us`) | `receipt.total` (`us`) |
| :--- | ---: | ---: | ---: | ---: |
| Cold upload + render | 0.2 | 861.9 | 1399.6 | 2254.8 |
| Warm repeated W/L | 0.2 | 41.2 | 845.4 | 889.6 |

#### Tracked `PoolUsageSnapshot` Peak Memory Bytes

| Pool | Peak Reserved Bytes / Count | Active After Drop |
| :--- | ---: | ---: |
| `retained_source_bytes` | 8388608 | 0 |
| `gpu_staging_upload_bytes` | 8388608 | 0 |
| `gpu_resident_bytes` (source `R16` + transient `R8Uint` target) | 12582912 | 0 |
| `gpu_staging_readback_bytes` | 4194304 | 0 |
| `cpu_output_bytes` | 12582912 | 0 |
| `in_flight_jobs` | 1 | 0 |

