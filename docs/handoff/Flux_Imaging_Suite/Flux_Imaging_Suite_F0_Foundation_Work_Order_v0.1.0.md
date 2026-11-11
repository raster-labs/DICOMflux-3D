# Flux Imaging Suite — F0 Common Image API, Memory and Execution Contracts

**Work order:** F0 · **Version:** 0.1.0 · **Date:** 10 November 2026 · **Status:** PROPOSED FOR OWNER / TECHNICAL ADMISSION; DOCUMENTATION ONLY  
**Owner:** Raster Images Private Limited · **Executor:** separately commissioned Rust coding agent · **Licensing target:** project-owned MIT  
**Folder:** [Flux Imaging Suite](https://drive.google.com/drive/folders/1RQ7qcjhZntu3_9QMjv8_MKdA2PJJOFlP)  
**Governing:** [DICOMflux and Flux Imaging Suite Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view)  
**Parent instructions:** [Flux Imaging Suite Rust Coding-Agent Implementation Instructions v0.1.0](https://drive.google.com/file/d/1bFhqX1t4Qh0tezeyXXt4ofbWqTTShRdx/view)

> **Authority:** Creating this work order neither writes Rust code nor authorises a GitHub change. A developer must identify approved repositories, target branch, permissions and reviewers before commissioning a separate coding agent. F0 may proceed in parallel with DICOMflux D0; F0 **MUST NOT** depend on DICOMflux or any desktop, Android or browser application.

## 1. Target outcome and portfolio boundary

Deliver an independently testable generic image and memory contract that all future Flux codecs/resamplers can implement consistently in Rust: validated descriptors, owned and borrowed images, writable destinations with one-writer lifecycle, resource/copy/execution/fidelity contracts, deterministic errors and a small independent consumer. **F0 does not implement any codec or resampler**; that belongs to F1 and later work orders. Avoid broad framework scaffolding with no executable consumer.

```text
F0: image descriptors / storage owners / leases / limits / reports / ABI
  ├── F1-R: FluxRLE (PackBits + DICOM RLE frame profile)
  ├── F1-S: FluxResample nearest-neighbour + bilinear
  └── later: FluxJLS, FluxJ2K/HTJ2K, FluxJXL, FluxJLI
DICOMflux: consumer of admitted Flux capabilities (never a Flux dependency)
```

A tiny neutral `flux-image-core` crate is permitted if it demonstrably removes incompatible duplication. Each codec must remain independently buildable without sibling codecs or an umbrella runtime; the shared crate must contain no DICOM, UI, imaging-workflow or network dependency. Alternatively, mirror a versioned contract through implementation-local types if that better preserves independence. Select *one* approach in F0 and test consumers; do not introduce two competing common APIs.

## 2. Source identities, current differences and provenance

The inspected Swift Imaging Suite successors are:

| Swift reference | Current `main` (10 Nov 2026) | Published release | Current HEAD licence |
|---|---|---|---|
| SwiftJ2K | `8b94527fb3057ea58cb49c754219f97961df3557` | `v12.1.0`; `12.2.0` development line | MIT |
| SwiftJLS | `a014bac5611ca654079c655e8fd88eb25caeab15` | `v2.0.0` | MIT |
| SwiftJXL | `e3adb2d0f5d16e38d36fd13e7f4a2de385e66381` | `v2.0.0` | Apache-2.0 |
| SwiftJLI | `97a50eee8af72e2a0b32f8a2444b8bc7b96a4c2c` | `v2.0.2` | MIT |
| SwiftRLE | `f81855537674ac5c3d64d6a984386bc54988b176` | `v1.0.1` | MIT |

The package's principal F0 references are `Documentation/COMMON_API.md`, `MEMORY_CONTRACT.md`, `SUITE_POLICY.md`, `COMMON_CONTRACT_SHA256.txt`, `Sources/SwiftRLE/ImageDescriptor.swift`, `ImageStorage.swift`, `CodecAPI.swift`, `Options.swift`, plus the same public declarations in each successor. **The common documents are not automatically reconciled:** at inspection, `SUITE_POLICY.md` reports contract `0.11.0` in SwiftJ2K/SwiftJLS but `0.10.0` in SwiftJXL/SwiftJLI/SwiftRLE; some `COMMON_API.md`/`MEMORY_CONTRACT.md` headers retain historical `0.7.0`. Inspect exact bytes and precedence, do not pick a shared version solely from its filename.

The SwiftJ2K HEAD 12.2.0 line is not the latest published tag and has distinct qualification holds. SwiftJXL's current code remains Apache-2.0. Older tags and third-party assets retain their actual original licence/notice terms even where a repository HEAD is now MIT. F0's new Rust source follows the owner's MIT direction, subject to exact-source ownership and counsel-reviewed contribution assignment. Do not wrap or vendor an external complete codec.

**SwiftResample:** No verified current GitHub repository. An accessible Drive `SwiftResample` folder exists but contained zero files when checked on 10 November 2026. This is a named source-identity gap; F0 defines only generic image contracts, not resampling mathematics.

## 3. Included requirements — F0

| ID | Required contract | Observable evidence |
|---|---|---|
| **F0-REQ-001** | Create the minimal authorised Rust workspace/crate graph, licensing/provenance/security/release documentation and stable Rust toolchain/MSRV/feature policy. | Clean external consumer with no DICOM/sibling-codec/UI dependency. |
| **F0-REQ-002** | Define validated `ImageDescriptor`/`PlaneDescriptor`: width, height, sample type, storage bits, meaningful bits, byte order, components, colour, alpha, plane dimensions, component mapping, offsets, pixel/sample strides, row stride and exact retained capacity. | Valid and invalid geometry/layout test matrix. |
| **F0-REQ-003** | Define canonical numeric sample semantics: integers low-bit aligned, unused high bits zero/sign-extended, signed precision includes sign, no silent clipping/down-conversion; lossless transforms compare meaningful samples. | 12-in-16 signed/unsigned and 16-bit extreme oracle cases. |
| **F0-REQ-004** | Define immutable owning `Image` and read-only storage, with explicit allocation identity/generation and scoped borrow. Borrowed pointers must never outlive their valid borrow. | Concurrent readers, alias identity, owner drop/release tests. |
| **F0-REQ-005** | Define `ImageDestination` with exclusive writer reservation, writable scoped access and seal/abort/invalidate outcome; no partially written image may be published as complete. | One-writer, double-lease rejection, cancellation while writing, failed-seal tests. |
| **F0-REQ-006** | Bound checked dimensions, products, plane extents, overlap, alignment and conversion; calculate actual final addressed byte before indexing. | Overflow, nonzero offset, padded row, multi-plane and aliasing tests. |
| **F0-REQ-007** | Implement finite `ResourceLimits` and per-operation accounting for live backing, workspace, output, metadata and concurrent tasks; distinguish charged bytes from process RSS and external memory. | Rejection/rollback/retained-owner accounting and concurrency records. |
| **F0-REQ-008** | Define `EncodeOptions`, `DecodeOptions` and common `Encoder`/`Decoder` vocabulary as capability contracts only; codec-specific controls may not be smuggled into shared semantics. | Documentation/compile check and capability-only implementation. |
| **F0-REQ-009** | Define execution (`scalar`, automatic, preferred/required backend), worker budgets, cooperative cancellation/deadlines, progress/event callback reentrancy and deterministic operation completion. | Cancellation/worker-limit/reentrancy tests. |
| **F0-REQ-010** | Define `CopyPolicy` (`allow` vs `require_shared_storage`), explicit copy events, and no-zero-copy-without-proof reporting. | Strict-copy rejection and honest copy accounting tests. |
| **F0-REQ-011** | Define `OperationReport`, error taxonomy and **distinct fidelity categories** for exact logical samples, bounded error, lossy, original compressed bitstream and allocated-cell preservation. | Reports and stable error/metric schema tests. |
| **F0-REQ-012** | Define a thin versioned C ABI for descriptors/opaque owners/status/capabilities with no exposed Rust layout; avoid parallel incompatible JVM/JNI facades in F0. | External C consumer, allocation/release and malformed-call checks. |
| **F0-REQ-013** | Provide common CLI vocabulary/schema and external-consumer examples without fake codec success. | A real `capabilities` command; unsupported `encode`/`decode` fail truthfully. |
| **F0-REQ-014** | Provide requirement-to-test matrix, SBOM/notice list, fixtures, precise source hashes, signed-off review fields and SHA-256 handoff. | F0 completion evidence; no invented execution. |

## 4. Descriptor, numeric and ownership details

**Minimum required interoperable image profile:** unsigned greyscale, 16-bit storage, little endian, one plane, two bytes per sample, positive even row stride at least `width * 2`, packed or padded rows, declared capacity and two-byte alignment or safe unaligned reads. The general type system may describe 8-bit/16-bit signed/unsigned integers, optional floating point, planar/interleaved components, alpha, ICC or other metadata, **but describing a type does not claim a codec supports it**.

For each plane calculate in checked arithmetic its final addressed byte: `offset + (height-1)*row_stride + (width-1)*pixel_stride + last_component_offset + sample_bytes`. Validate against the **retained allocation's** capacity, not the visible slice length alone. Never infer the next plane's origin from the previous plane's height; row padding and explicit plane offsets matter. Negative/overlapping writable strides must be rejected unless a distinct profile later admits them. A scalar fallback may handle unaligned data; do not blindly use aligned loads.

- **Representations:** Stored sample values, modality-transformed values and display colours must remain distinct. F0 is generic imaging and includes no DICOM modality transform, window/level, Transfer Syntax or patient metadata logic.
- **Read-only owner:** Multiple immutable readers may share sealed backing with one explicit allocation identity and retained lifetime; no single-writer lease imposed on read-only sources.
- **Writable owner:** State transitions proposed as `Vacant -> Reserved -> Writing -> Sealed` or `Invalid`. The exact Rust type-level mechanism is to be fixed once, with no double-writer lease, no sealing until all workers finish, and no use of uninitialised bytes as a safe slice.
- **Exception/error behaviour:** A failed operation after writing begins invalidates output unless a specifically supported partial type is returned; cancellation does not release shared or device storage still in use. Resource ledger charges actual capacity until final owner retirement.
- **Copies:** Moving an owner handle is not a pixel copy. If a backend requires conversion, staging or a whole-image handoff copy, `require_shared_storage` must reject it; `allow_copy` must report reason, bytes, input/output layouts and real allocators.

## 5. Common operations and FFI boundary

The following is an API *inventory*, not pre-approved Rust syntax:

```rust
ImageDescriptor::validate(limits) -> Result<ValidatedDescriptor, ImageError>;
Image::from_owned(storage, descriptor) -> Result<Image, ImageError>;
Image::with_bytes(|bytes| ...);           // synchronous, non-escaping borrow
ImageDestination::reserve() -> Result<WriteLease, ImageError>;
WriteLease::with_bytes_mut(|bytes| ...);
WriteLease::seal() -> Result<Image, ImageError>;
WriteLease::invalidate(reason);
CodecCapabilities::query() -> Capabilities;
OperationReport { backend, fidelity, copy_events, resource_peaks, normalisations };
```

The implementation must document the safe Rust form (borrow, lifetime, non-Send lease if relevant) and separately an opaque C owner/handle layout. `repr(C)` is a C interoperation technique, not a universal storage serialisation. Strings and arrays in FFI use explicit length, encoding and owner; C must call matching Rust release. Do not claim a Kotlin/JNI binding exists merely because an Android Rust `.so` builds. The later Android consumer may use JNI/NDK without requiring F0 to use the desktop Java 25/FFM route.

## 6. Execution sequence

1. Preflight source/rights, exact `COMMON_CONTRACT_SHA256` inputs, MSRV/toolchain and repository permissions. Resolve any conflicting shared document revisions in a single F0 contract decision record.
2. Implement `ImageDescriptor`/`PlaneDescriptor` and strict checked geometry, signedness, byte order and colour semantics; write hand-calculated tests.
3. Implement read-only owner and one-writer `ImageDestination` lifecycle with exclusive mutable borrow; exercise alias, panic/cancel and drop semantics without permitting unsafe storage reuse.
4. Implement resource/copy/fidelity reports, cancellation and capability-only API; test finite limits and no-hidden-output on failure.
5. Publish minimal C ABI and independent consumer, common CLI/command schema, CI/testing and source register; stop before F1 algorithm work.

Do not add a universal executor, GPU runtime, container format, intermediate TIFF/PNG/NRRD representation, DICOM parser, pre-emptive codec registry or umbrella package.

## 7. Testing, commands and target qualification

The following future-agent commands are indicative; adapt package names and flags to the real Cargo graph and record actual invocations.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo test --manifest-path examples/independent-consumer/Cargo.toml --locked
cargo build --release --locked --target x86_64-pc-windows-msvc
cargo build --release --locked --target aarch64-pc-windows-msvc
cargo build --release --locked --target x86_64-unknown-linux-gnu
cargo build --release --locked --target aarch64-unknown-linux-gnu
cargo build --release --locked --target aarch64-linux-android
cargo build --release --locked --target x86_64-linux-android
# Execute actual appropriate external consumers on each target/device.
```

**Independent test oracles:** a 3×2 unsigned-16 greyscale plane with `row_stride=8` has 12 payload bytes but last addressed byte at `offset + (2-1)*8 + (3-1)*2 + 2` (exclusive endpoint), not `3*2*2`. For signed 12-in-16, `-1` has meaningful two's-complement 12-bit form `0xFFF` and, in canonical 16-bit RAM, sign-extension `0xFFFF`. `0xF000` cannot silently be interpreted as canonical unsigned 12-bit data. A final-byte-endpoint equal to retained capacity is permitted; exceeding it fails. Two destinations targeting overlapping writable ranges fail unless one is ended and re-admitted.

Other required cases: zero dimensions, malformed component mapping, odd row stride if incompatible, integer multiplication overflow, noncanonical high bits, overlapping planes, double writer, immutable read while mutably writing, user callback reentrancy, dropping a future before worker completion, background owner lease after a host wrapper closes, resource-limit refusal and allocator failure. Test C ABI version mismatch and null/invalid scalar arguments without unwinding. Use sanitiser/Miri or equivalent supported tools where feasible, and report unexecuted analyses honestly.

**Platform acceptance:** Windows/Linux x64 and ARM64, Android ARM64 physical device plus separately qualified Android x86_64 emulator. The required foundation consumer must execute on the actual target. Cross-compilation is not a physical run, and the Android toolchain must use the Android target/sysroot, not a desktop Linux binary. Library ABI/Android packaging tests must be repeated when a Kotlin AAR is later developed. macOS remains secondary.

## 8. F0 acceptance gates

| Gate | Pass criteria / evidence | Disposition if missing |
|---|---|---|
| **F0-G01** | Source/rights, selected shared contract and target toolchain are pinned, with audited deviations. | BLOCKED on unlicensed paths/unspecified contract. |
| **F0-G02** | Descriptor/numeric contracts pass valid and hostile boundary vectors, without overflow/overlap. | FAIL. |
| **F0-G03** | Read owners, destination leases, cancellation, invalidation and retirement tested under concurrent scenarios. | FAIL. |
| **F0-G04** | Resource caps, copy policy and fidelity categories produce accurate reports and fail closed. | FAIL. |
| **F0-G05** | Independent Rust consumer and external C ABI consumer both execute without sibling/GUI/DICOM dependencies. | FAIL. |
| **F0-G06** | CLI/capabilities and negative unsupported operations produce stable process results; no fake codec success. | FAIL. |
| **F0-G07** | Required native target/device executions logged distinctly with exact toolchains. | NOT RUN/BLOCKED per missing target. |
| **F0-G08** | Release/notice/SBOM, requirement-evidence matrix and SHA-256 verified; independent reviewer records disposition. | INCOMPLETE. |

An internal host slice may allow F1 exploratory coding after F0-G02..06 pass, but no claim of full F0 platform acceptance until G07 and source/rights controls close. Never describe an unresolved Swift contract mismatch as a passed equivalence gate.

## 9. Resource/performance budgets and security

Before implementation, the agent selects finite limits for pixel count, components, planes, retained bytes, buffer alignment, worker count, scratch capacity and queued operations, with a named device/workload. Report actual peak charged bytes, process RSS and any device memory separately. Benchmark creation, validation, borrow, lease/seal, failed allocation and C-ABI handoff; measure full operation cost, not a synthetic no-op. Correctness and security take precedence over any throughput target. No patient payload is included in logs or prompts. Provide security contact, dependency inventory and a process for malformed-data bugs.

## 10. Open decisions and boundary controls

- **F0-DEC-01:** elect common-crate versus mirrored local implementation with a testable independence guarantee, avoiding publication/release coupling.
- **F0-DEC-02:** freeze the Rust normalised descriptors, error/fidelity and owner/lease signatures after examining *actual* Swift declarations from all five source snapshots.
- **F0-DEC-03:** reconcile the `SUITE_POLICY` 0.10.0/0.11.0 drift and historical `COMMON_API`/`MEMORY_CONTRACT` headers; no unverified version-wide parity claim.
- **F0-DEC-04:** approve the minimal C ABI and host-aggregate admission shape, or explicitly defer optional parts not used by F1.
- **F0-DEC-05:** select toolchain/primary platform environments, fixtures, limits and named independent reviewer.

Each open decision must be resolved within the implementing work order or marked a blocking precise contract issue; it does not authorise re-opening the owner-approved library independence/MIT/first-platform direction.

## 11. Completion report and acceptance owner

Deliver `F0_COMPLETION.md`, `F0_CONTRACT_INVENTORY.md`, `F0_SOURCE_PROVENANCE.md`, `F0_TEST_MATRIX.md`, `F0_TARGET_MATRIX.md` and `SHA256SUMS.txt` (or an equivalent concise indexed set). Preserve **IMPLEMENTED / BUILT / EXECUTED / VERIFIED / NOT RUN / BLOCKED** distinctions, exact commits, commands, logs, source/dataset hashes, measured memory and any `unsafe` justification. Independent reviewer records **ADMIT / ADMIT WITH EXPLICIT HOLDS / RETURN FOR CORRECTION**. F0 admission does not automatically authorise implementing F1-R or F1-S; assign them separately.

## 12. Definition of done and dependencies

F0 is complete only for the explicitly listed profiles passing gates G01–G08. It must build as a standalone component with a neutral dependency graph and a test consumer outside its workspace. F1-R and F1-S may advance after the applicable accepted F0 image/memory subset is stable, and may finish at separate times. DICOMflux D0/D1 never gate F0 or F1.

## 13. Copy-ready F0 coding-agent instruction

```text
You are the separately authorised F0 Rust coding agent for Flux Imaging
Suite. Read Manifesto v1.3.1, Flux Imaging Suite Rust Implementation
Instructions v0.1.0, the F0 Work Order v0.1.0 and its source register.
Pin the latest specified SwiftJ2K/JLS/JXL/JLI/RLE revisions and check
actual licence/contract bytes. Do not assume suite contracts agree.

Implement only the common, DICOM-independent Rust descriptor, owned image,
exclusive destination lease, resource/copy/fidelity/error/cancellation,
minimum C ABI and independent-consumer contracts in F0-REQ-001..014.
Prove last-byte arithmetic, signed meaningful bits, stable ownership through
cancellation, no overlapping writers, correct refusal/rollback and truthful
capabilities. Do not implement FluxRLE or resampling algorithms in F0.

No DICOMflux, GPU, UI, foreign complete codec or umbrella dependency.
Run the actual tests and named target consumers; report unexecuted device
checks, sanitiser exclusions and licensing gaps explicitly. Submit G01..08
evidence, source/build hashes, memory reports and the next bounded F1
commission. Stop after F0.
```

## 14. Source and reference links

- [SwiftRLE reference API](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Documentation/COMMON_API.md), [memory contract](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Documentation/MEMORY_CONTRACT.md), and [suite policy](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Documentation/SUITE_POLICY.md).
- [SwiftJ2K HEAD policy](https://github.com/raster-labs/SwiftJ2K/blob/8b94527fb3057ea58cb49c754219f97961df3557/Documentation/SUITE_POLICY.md); [SwiftJLS HEAD policy](https://github.com/raster-labs/SwiftJLS/blob/a014bac5611ca654079c655e8fd88eb25caeab15/Documentation/SUITE_POLICY.md).
- [Rust Reference / lifetimes](https://doc.rust-lang.org/reference/), [Cargo rust-version](https://doc.rust-lang.org/cargo/reference/rust-version.html); full sources and qualification limits in `Flux_Imaging_Suite_Work_Order_Source_Register_v0.1.0.md`.

**Documentation-only status:** No F0 implementation, codec, benchmark, target build or clinical result has been produced here.
