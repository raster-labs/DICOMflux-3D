# Flux Imaging Suite — F1 Initial Algorithms: FluxRLE and FluxResample

**Work order:** F1 · **Version:** 0.1.0 · **Issued:** 10 November 2026 · **Status:** PROPOSED; IMPLEMENTATION REQUIRES SEPARATE AUTHORITY  
**Subtracks:** **F1-R** (FluxRLE PackBits/DICOM frame) and **F1-S** (nearest-neighbour/bilinear resampling), **separately reviewable and separately accepted**  
**Owner:** Raster Images Private Limited · **Governing:** [Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view)  
**Folder:** [Flux Imaging Suite](https://drive.google.com/drive/folders/1RQ7qcjhZntu3_9QMjv8_MKdA2PJJOFlP) · **Parent handoff:** [Rust Implementation Instructions v0.1.0](https://drive.google.com/file/d/1bFhqX1t4Qh0tezeyXXt4ofbWqTTShRdx/view)

> **Scope safeguard:** This document commissions no implementation in the present conversation. The coding agent may execute only the subtrack and repository authorised later by the developer. F1-R and F1-S depend on accepted F0 image, buffer, error and resource contracts; neither depends on DICOMflux. Passing one track does not approve the other.

## 1. Deliverable and dependencies

F1 provides the first actual generic image algorithms on the F0 contract. F1-R must encode/decode PackBits byte/row streams and a strict DICOM RLE **frame** profile, without parsing DICOM datasets. F1-S must implement nearest-neighbour and bilinear resizing on explicit typed image descriptors without assuming display-oriented semantics. Every advertised profile requires independently justified correctness evidence, malformed inputs, cancellation and retained buffer ownership.

```text
F0 common contract accepted
  ├── F1-R FluxRLE: standalone PackBits + selected DICOM RLE frame
  └── F1-S FluxResample: nearest + bilinear (independent gate)
Later: additional resampling kernels and compression families
Separate DICOMflux image adapter later consumes F1-R; no reverse dependency.
```

## 2. Source selection and unresolved SwiftResample identity

- **FluxRLE reference:** `raster-labs/SwiftRLE` **main** `f81855537674ac5c3d64d6a984386bc54988b176` (10 November 2026), published release `v1.0.1`. Review `Sources/SwiftRLE/PackBitsEncoder.swift`, `PackBitsDecoder.swift`, `DICOMRLEFrame.swift`, `RLEDescriptor.swift`, `CodecAPI.swift`, `ImageStorage.swift`, `ImageDescriptor.swift`, `Documentation/R2_FRAME_PROFILE.md`, `R5_QUALIFICATION.md` and current licence/notice records. HEAD now declares MIT but old tags/documents may be Apache-2.0; track copied/adapted paths and third-party notices individually.
- **Common SIS:** five successor HEADs and their differing common contracts are pinned in F0's source register. Do not let SwiftJ2K 12.2.0 and its unreleased/held gates establish a generic F1 completion claim.
- **Normative DICOM RLE:** DICOM PS3.5 **2026d** §8.2.2 and Annex G. Acquire and hash the official 2026d text and verify each byte-plane/header/packet rule; do not substitute the Swift implementation for the standard. The selected subset is not a claim about every colour/pixel format.
- **SwiftResample:** `raster-labs/SwiftResample` was not found on GitHub and the accessible `SwiftResample` Google Drive folder `1G1rubxhzRlD72giX7it_xPDVcX_U-5n8` contained no files. The **accepted seven-kernel programme scope** is confirmed, but no current exact Swift public API or numerical defaults were verified. F1-S must explicitly document chosen mathematical conventions and *cannot claim numerical Swift parity* until that source is recovered or an accepted replacement contract is approved. This is **F1-S-G00**, not a block on F1-R.

## 3. Common F1 requirements (both tracks)

| ID | Common obligation |
|---|---|
| **F1-REQ-001** | Use only the admitted F0 descriptor/ownership/resource/error/copy contracts; no independently invented competing image type. |
| **F1-REQ-002** | Implement portable, safe Rust scalar algorithms. SIMD, Metal, CUDA, Vulkan, NPU and other acceleration are later independently qualified paths, not F1 prerequisites. |
| **F1-REQ-003** | Validate every input/output byte region, precision, capacity and stride with checked arithmetic before allocating, reading or writing. |
| **F1-REQ-004** | Offer an allocating result and a caller-provided destination where the selected operation supports it; preserve one-writer reserve/write/seal/invalidate semantics. |
| **F1-REQ-005** | Honour finite memory/work/worker limits and cooperative cancellation without leaking leases or publishing a partial image as complete. |
| **F1-REQ-006** | Emit accurate `OperationReport`: actual backend, fidelity class, copy events, source/destination layouts, allocations/peaks, normalisations and any loss of precision. |
| **F1-REQ-007** | Provide a useful standalone CLI with separate stdout payload/machine report and stderr diagnostics; atomic file output and deliberate overwrite handling. |
| **F1-REQ-008** | Use rights-cleared, internally versioned fixtures and standards/hand-mathematics-derived expected results; self-round-trips are supplementary. |
| **F1-REQ-009** | Build and execute on named Windows/Linux x64/ARM64 and Android ARM64 profiles; Android x86_64 is a distinct emulator profile; macOS secondary. |
| **F1-REQ-010** | Deliver per-subtrack requirement evidence, target/build logs, malformed/fuzz cases, performance measurements, notices and SHA-256. |

Neither track may depend on DICOMflux, the applications, a GPU UI toolkit or a complete external compression engine. Third-party codecs may be invoked only by isolated **development-time** oracles; the shipped Rust algorithms remain programme-owned.

## 4. F1-R — FluxRLE required capability

### Scope and algorithm contracts

| ID | Implementing-agent deliverable | Verification basis |
|---|---|---|
| **F1-R-REQ-001** | Raw PackBits byte-stream encode/decode with declared final decoded size and bounded packet work. A row-constrained mode must reset or enforce row output boundaries explicitly. | Independent literal/run control-byte examples; malformed short packet; zero/one/exact-size outputs. |
| **F1-R-REQ-002** | PackBits semantics: controls `0..127` copy `n+1` literal bytes; `-127..-1` repeat the next byte `1-n` times; `-128` is a no-op, subject to bounded work. | Reference hand-coded packet vectors and exact output. |
| **F1-R-REQ-003** | DICOM RLE **frame** profile with 64-byte little-endian header: segment count, 15 offsets, first offset 64, increasing offsets in frame bounds, unused offsets zero, expected segment count from external descriptor. | Raw header tests and independent Annex G comparison; no fabricated frame geometry. |
| **F1-R-REQ-004** | Explicit `RLEFrameDescriptor` supplied by caller: width/height, sample type, `BitsAllocated`, `BitsStored`, signedness, component roles/colour, pixel/plane strides and RAM byte order. Frame header alone has no such information. | Missing/conflicting descriptor rejected; correct metadata-free frame interpretation. |
| **F1-R-REQ-005** | First admitted image profile: 8-bit and 16-bit **monochrome**, signed or unsigned, meaningful bits 1..allocated bits subject to F0's canonical representation. Other sample/components are explicitly unsupported unless later admitted. | Extreme 8/16-bit and signed 12-in-16 fixtures; no silent RGB/alpha/signed-colour claims. |
| **F1-R-REQ-006** | Encode DICOM segments as component/byte planes in **most-significant-byte first** order, with PackBits for each segment and frame-safe padding/boundaries under Annex G. | Independent byte-lane fixture and cross-decoder; not only self-round-trip. |
| **F1-R-REQ-007** | Decode directly to a checked final destination when possible; do not require a hidden whole-frame pixel workspace. Enforce exact expected bytes per segment/row. | Allocation instrumentation and malformed packet tests. |
| **F1-R-REQ-008** | Distinguish `decode_allocated_cells` preserving every allocated bit from `decode_image` producing canonical logical samples, including zeroing/sign-extending unused bits. Do not claim both guarantees simultaneously. | Two distinct fidelity tests and truthful report. |
| **F1-R-REQ-009** | Strict trailing input policy: accept only padding/NOP permitted by the chosen normative and source-evidence profile, charge NOP work and reject unexplained trailing bytes. Document any source-specific stricter rule. | Padding/extra-data positive and negative fixtures. |
| **F1-R-REQ-010** | CLI `encode`, `decode`, `inspect`, `validate`, `capabilities` for the exact supported PackBits and DICOM RLE profiles, with clear descriptors/limits. | Process tests, independent input/output comparison, unsupported format failures. |

**Frame structural requirements:** Count `1..15` segments, all offset values stored as little-endian `u32`; validate conversion to target `usize`, first offset `64`, strictly increasing offsets for populated segments and end no later than frame length. Check unused header slots. For 16-bit monochrome, byte-plane decomposition is MSB then LSB regardless of RAM endianness; the destination layout is reconstructed through explicit `byteOrder`. `segments = samples_per_pixel * bytes_per_sample` for supported allocated widths. This frame is **not a DICOM Part 10 object**; no patient metadata or transfer-syntax negotiation enters this crate.

**One important status distinction:** A strict RLE packet/header decoder may be correct without full colour, multi-frame offset-table or dataset support. Do not claim those profiles. Cross-codec, DICOMflux integration and conversion of metadata/lossy history require separate later work.

### F1-R tests and independent oracles

| Test | Fixture/result |
|---|---|
| **R-T01** | Literal packet `[2, A, B, C]` produces exactly `ABC` (byte values, not text encoding). |
| **R-T02** | Run control `0xFE, 0x7A` represents three copies of `0x7A`. |
| **R-T03** | `0x80` NOP does not emit output but counts against input/work limits. |
| **R-T04** | 16-bit pixels `0x1234`, `0xABCD` require MSB segment `[0x12,0xAB]` and LSB `[0x34,0xCD]` before PackBits, independent of host endian. |
| **R-T05** | Signed 12-in-16 `-1`, `-2048`, `2047`, padded-row and nonzero-offset buffers preserve declared logical values. |
| **R-T06** | Header too short, count 0/16, first offset not 64, overlapping/decreasing offset, offset beyond frame, nonzero unused offset -> structural error. |
| **R-T07** | Packet crosses row output boundary, truncated literal or run, decoded count mismatch, unexpected tail/NOP amplification -> precise failure without output overrun. |
| **R-T08** | Aliased writable destination, insufficient capacity, cancellation mid-frame and resource refusal -> invalidated output, no stale published frame. |
| **R-T09** | Independent encoder/decoder cross-check using permitted DCMTK or other isolated oracle on at least the admitted mono profiles, with exact source and command receipts. |

A successful `encode` -> `decode` of our own bytes does **not** establish Annex G conformance. Match header and reconstructed meaningful/allocated-cell bytes against independent expectations; DICOM integration is checked later by its own adapter.

## 5. F1-S — initial FluxResample capability

### Mandatory numerical convention gate: F1-S-G00

Before claiming a complete F1-S implementation, recover the authoritative SwiftResample source/specification **or** have the owner-appointed technical reviewer admit a standalone **independent numerical contract**. The following are concrete **candidate conventions for review**, not verified Swift behaviour:

- Pixel centres at integer sample indices; map destination centre `(i+0.5)*src_size/dst_size - 0.5` to continuous source coordinates.
- Nearest-neighbour selects the nearest source index using a *specified* tie rule; clamp out-of-range indices to the nearest valid source sample.
- Bilinear samples the four neighbouring source centres with separable weights and an explicitly stated clamp-to-edge rule.
- Integer conversion uses a named accumulator precision and an explicit rounding/tie/saturation rule. Do not let language casts silently decide this.

The agent **MUST NOT** assert these conventions are inherited from SwiftResample until reconciled with its actual accepted contract. If unresolved, F1-S can produce a well-labelled experimental implementation but **F1-S-G00 remains open**; its gate does not block F1-R.

### Required functions, profiles and tests

| ID | F1-S obligation | Required proof |
|---|---|---|
| **F1-S-REQ-001** | Validate 2D source/destination descriptors, dimensions, extents and coordinate convention; no overflow or out-of-bounds reads. | Small exact matrices, zero/huge dimensions and stride cases. |
| **F1-S-REQ-002** | Implement nearest-neighbour for declared integer greyscale profiles; labels may use nearest only. | Exact source sample equality and label-preservation vectors. |
| **F1-S-REQ-003** | Implement bilinear for admitted quantitative scalar samples, with explicit weights, precision, boundary and rounding. Reject categorical/segmentation label data. | Independently calculated interpolation values and tie/boundary tests. |
| **F1-S-REQ-004** | Support source/destination nonzero offsets, padded rows, both byte orders and caller-owned output through F0 ownership. | Sentinel padding intact, no overlap and lifetime tests. |
| **F1-S-REQ-005** | Define aspect-ratio policy: explicit output width/height; no implicit crop, stretch or physical-pixel-spacing metadata rewrite. | Geometry request tests and explicit no-op/identity cases. |
| **F1-S-REQ-006** | Preserve declared sample type/precision and signedness or report explicit conversion; do not apply windowing, gamma, colour-space conversion or normalisation. | Signed/unsigned high-precision and no-silent-conversion tests. |
| **F1-S-REQ-007** | Enforce resource/work limits, cancellation and one-writer lifecycle, with scalar CPU reference and truthful full-operation costs. | Cancellation/low-memory, work-budget, allocation and stress runs. |
| **F1-S-REQ-008** | Publish CLI `resize`/`inspect`/`capabilities` for supported profiles, schema-validated machine results and independent consumer. | Pixel oracle comparisons and process exit tests. |

**Independent 2D vectors to include:** a 2×2 source greyscale matrix `[0, 10; 20, 30]`, evaluated at a documented centre position to yield a bilinear 15 before output rounding; a 1×N and N×1 edge case; nonuniform resize 3×2 -> 5×3; downsize/upsample with padded rows and source offsets; source signed extrema; labels `{0,1,2}` never producing a third class by interpolation. Verify bilinear weights and integer tie decisions against the independently accepted F1-S-G00 numerical contract. Do not claim that a simple bilinear resize is an antialiased downsampling filter; the accepted seven-kernel portfolio remains later work.

The full accepted kernels (nearest, bilinear, bicubic, Mitchell-Netravali, Catmull-Rom, Lanczos, area/box) are future separately scoped capabilities. GPU resampling, spatial registration, DICOM pixel spacing/rescale updates and medical measurements are not part of F1-S.

## 6. Execution sequence and code ownership

1. **F1-Preflight:** freeze admitted F0 revision and selected SwiftRLE source/rights, recover normative Annex G and F1-S source if available. Specify working repository(s), branch and assigned agent authority. No initial creation of all future codecs unless authorised.
2. **F1-R1:** independently implement PackBits bounded byte/row encode/decode; pass R-T01..03/07 before DICOM frame work.
3. **F1-R2:** implement strict 64-byte header, byte-plane lanes and mono frame encode/decode; pass R-T04..09 and report raw-cell/canonical logical fidelity separately.
4. **F1-S0:** resolve or explicitly hold F1-S-G00 numerical contract and label-type policy; develop hand-checkable 2D vectors.
5. **F1-S1:** implement nearest/bilinear scalar operations, caller destination, cancellation and exact per-request report; pass all S requirements and independent sample oracles.
6. **F1-Integration:** two independent CLI/consumer tests and per-profile cross-platform executions; prepare per-subtrack completion reports and sources/hashes. No F1 success blanket statement.

Separate branches or modules can progress independently once F0 is stable. `FluxRLE` remains DICOM-aware at its frame-profile boundary, but does not parse DICOM datasets or negotiate Transfer Syntax UIDs. `FluxResample` remains entirely DICOM-independent.

## 7. Verification commands, hardware and resource budgets

Future implementing agents should record actual commands (these examples are not executed):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p fluxrle --locked
cargo test -p fluxresample --locked
cargo run --release -p fluxrle-cli -- capabilities --format json
cargo run --release -p fluxresample-cli -- capabilities --format json
# Run R-T01..09 and all F1-S exact matrices against an independent fixture manifest.
# Build and execute each *assigned* module/consumer on all declared target profiles.
```

The final package names, CLI flags and example fixture paths follow the admitted F0 implementation. Use actual scripts/paths rather than reporting these illustrative commands as executed. Include fuzz campaigns for PackBits packet controls, RLE header offsets and resampling geometry/strides with fixed seed and duration; negative tests for cancelled work and a failed final seal. Report the actual tested input size, workers, scalar backend, resident memory, library-charged bytes, output bytes and copy events. Define performance budgets before measurement based on a selected low-resource ARM64 device and an x64 workstation; no unmeasured speed-up target.

**Primary execution matrix:** Windows x64 and ARM64; Linux x64 and ARM64; Android ARM64 physical device, with separate x86_64 Android emulator/CI profile. Each F1 track needs **actual executable operation tests**, not merely a static library build. Cross-compilation, emulator execution and real-device execution have different ledger statuses. Android 4 KB/16 KB native binary alignment and AAR handling are later binding/release gates but must not be contradicted by the F1 ABI or allocator assumptions. macOS optional/secondary.

## 8. Independent acceptance gates: F1-R and F1-S

| Gate | Pass condition | Evidence / disposition |
|---|---|---|
| **F1-R-G01** | F0 contract and normative 2026d Annex G/frame descriptor source pinned; rights clear. | Source/rights receipt; BLOCKED if missing. |
| **F1-R-G02** | PackBits matches independent packet oracles, rows, NOP policy and malformed/truncation cases. | R-T01..03/07 logs; FAIL otherwise. |
| **F1-R-G03** | DICOM RLE frame header/lane/mono tests pass, including raw allocated-cell vs canonical image fidelity. | R-T04..06, independent cross-codec oracles; FAIL otherwise. |
| **F1-R-G04** | Resource limits, cancellation, own destination, CLI, external consumer and independent interoperability evidence pass. | R-T08/09, process receipts; FAIL otherwise. |
| **F1-R-G05** | Required targets execute the advertised RLE profiles; notices/metrics/SHA-256 complete. | Per-target ledger; missing device = NOT RUN/BLOCKED. |
| **F1-S-G00** | SwiftResample contract recovered or independent coordinate/edge/tie/rounding rules *explicitly accepted*. | Source identity or signed technical decision; unresolved blocks parity claim. |
| **F1-S-G01** | Nearest matches exact integer and label-preservation oracles, padded rows, bounds/stride tests. | Matrices/negative cases; FAIL. |
| **F1-S-G02** | Bilinear matches approved independent numerical expectations, rejects labels and preserves declared signedness/output policy. | Exact/tolerance oracle and tie tests; FAIL. |
| **F1-S-G03** | Caller-destination, cancellation, resource refusal, CLI and independent consumer verified. | Source/build/test/memory receipts; FAIL. |
| **F1-S-G04** | Required targets execute the advertised operations; notice/metrics/SHA-256 complete. | Per-target ledger; missing = NOT RUN/BLOCKED. |

**Acceptance rule:** F1-R may be admitted with F1-S-G00 pending. F1-S may not be represented as SwiftResample-parity-qualified with its source missing. An owner-approved subset may progress with clear unresolved hardware gates, but no full multi-platform claim follows from host-only tests.

## 9. Open owner/reviewer decisions and exact blockers

- **F1-DEC-01:** confirm F0 selected source/ABI/buffer contract and the first mono sample/precision profiles; any expansion to colour, alpha, or extended precision is a subsequent explicit capability.
- **F1-DEC-02:** freeze whether `RawAllocatedCells` is a separate public API and its relationship to canonical `Image`; do not conflate byte-for-byte and logical sample guarantees.
- **F1-DEC-03:** agree precise strict frame/PackBits padding and NOP limits from normative and verified source, including handling of trailing zero pad and extra control bytes.
- **F1-DEC-04:** close F1-S-G00 (source recovery or accepted independent numerical rule set), especially centre transform, nearest tie, edge extension, floating accumulation and integer rounding.
- **F1-DEC-05:** select resource/worker limits and real Windows/Linux/Android execution environments and name separate independent reviewers for R and S.

A missing SwiftResample repository is an *evidenced source-identity blocker for exact compatibility*, not permission to fabricate an API and not an excuse to delay FluxRLE.

## 10. Completion records and review

Each subtrack supplies a separate `F1R_COMPLETION.md` or `F1S_COMPLETION.md` with requirement and negative-case evidence, source/fixture SHA-256, compiler/target and tests, validated fidelity class, memory/copy/worker reports, rights inventory, CLI invocation, independent-consumer path, and **IMPLEMENTED / BUILT / EXECUTED / VERIFIED / NOT RUN / BLOCKED** statuses. A combined summary must not convert either track's unexecuted tests into a passing result. The reviewer signs **F1-R ADMIT / HOLD / RETURN** and **F1-S ADMIT / HOLD / RETURN** separately. F1 acceptance does not qualify DICOMflux integration or medical application usage.

## 11. Copy-ready F1-R coding-agent prompt

```text
You are the separately authorised FluxRLE F1-R coding agent. Pin accepted
F0 contracts, SwiftRLE source f81855537674ac5c3d64d6a984386bc54988b176,
DICOM PS3.5 2026d Annex G and actual source licensing. Implement
standalone safe Rust PackBits and DICOM RLE frame encode/decode for admitted
8/16-bit monochrome samples: strict 64-byte header, MSB-to-LSB byte planes,
explicit descriptor, caller-owned destination, raw allocated-cell versus
canonical sample guarantees, work/memory limits, cancellation, reports and
CLI. Do not parse datasets or import a third-party codec in production.

Execute R-T01..09 with independent oracles, consumer, fuzz and target runs.
Report actual source/build/fixture SHA-256 and F1-R-G01..05 individually.
Stop at F1-R; do not claim the resampler track was completed.
```

## 12. Copy-ready F1-S coding-agent prompt

```text
You are the separately authorised FluxResample F1-S coding agent. Pin F0
contracts and inspect the SwiftResample source identity. If unavailable,
record F1-S-G00 as OPEN and present an independent numerical rule set for
technical approval. Do not claim SwiftResample parity without evidence.

After that rule set is admitted, implement portable safe Rust nearest and
bilinear 2D resampling with defined centre mapping, edge extension, tie and
rounding, signedness/precision, validated strides, caller-owned output,
resource/cancellation limits and CLI. Reject bilinear for segmentation
labels. Verify independent small matrices, malformed geometry, memory
lifetimes, exact target executions and numerical reports. Stop at F1-S;
do not claim other kernels or full portfolio completion.
```

## 13. Documentation and naming integrity

Use `FluxRLE` and `FluxResample` as the Rust library names in this work order, with exact Cargo crate naming fixed by authorised repository/bootstrap conventions. Preserve the distinct names DICOMflux, DICOMflux-3D and Flux Imaging Suite; do not duplicate module implementation in an application or place workstation documentation in this library folder. Both tracks remain MIT for newly authored project-owned work with historical Swift/third-party licences and notices preserved as applicable.

## 14. Normative/reference pointers and status

- [SwiftRLE current code](https://github.com/raster-labs/SwiftRLE/tree/f81855537674ac5c3d64d6a984386bc54988b176), [R2 strict frame profile](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Documentation/R2_FRAME_PROFILE.md).
- [DICOM official current index](https://dicom.nema.org/medical/DICOM/current/) and [PS3.5 Annex G](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/chapter_G.html); acquire exact 2026d archive/digests rather than relying solely on moving links.
- [SIS shared API](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Documentation/COMMON_API.md), [memory contract](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Documentation/MEMORY_CONTRACT.md); inspect current local contract divergences, see accompanying source register.

**Document status:** Proposed work order; F1-R and F1-S source code, tests and target executions have **not** been performed by this documentation commission.
