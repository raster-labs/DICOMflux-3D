# Flux Imaging Suite — Rust Coding-Agent Implementation Instructions

**Document version:** 0.1.0  
**Issued:** 10 November 2026  
**Programme:** DICOM Toolkit & Image Algorithms, Raster Images  
**Status:** Proposed coding-agent handoff; **not a claim that a Rust codec, benchmark or hardware test was executed**  
**Intended executor:** Separately authorised development/coding agent  
**Governing shared rules:** [DICOMflux and Flux Imaging Suite Development Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view)  
**Destination:** [Flux Imaging Suite documentation folder](https://drive.google.com/drive/folders/1RQ7qcjhZntu3_9QMjv8_MKdA2PJJOFlP)

> **Commission boundary.** Implement the **Rust image algorithms and their reusable API contracts**, not DICOMflux itself, not DICOMflux-3D and not an Android, desktop or web application. The owner must specify authorised repository locations, issue scopes, reviewers, access credentials, toolchains and execution authority. The material below is suitable for an agent work order; it does not authorise this assistant to alter source repositories or claim code completion.

## 1. Mission and portfolio

Build six **independently usable Rust library components** constituting the Flux Imaging Suite, conceptually aligned with the project's current **Swift Imaging Suite (SIS)**. Implement each codec and resampler within the new Rust programme, keeping generic image processing **DICOM aware but not DICOM dependent**. DICOM-specific dataset parsing, Transfer Syntax selection, patient metadata and output-object updates belong to DICOMflux. Native Rust algorithms are the deliverable, not wrappers around CharLS, OpenJPEG, libjxl or other complete external codec engines.

| Component | Main remit | First executable target | Beyond first slice |
|---|---|---|---|
| **FluxRLE** | PackBits + DICOM RLE frame profile | Bounded PackBits and 8/16-bit greyscale DICOM RLE encode/decode | Colour layouts, palette, additional interpretations |
| **FluxJLS** | JPEG-LS | Lossless greyscale encode/decode with explicit precision and bounds | Near-lossless, colour and interleave profiles |
| **FluxJLI** | JPEG family incl. SOF3 lossless | Declared SOF3 lossless greyscale | Sequential/progressive DCT, extended precisions, colour conversions |
| **FluxJ2K** | JPEG 2000 and HTJ2K | 2D reversible Part 1 profile, then separately HTJ2K | Part 2 MCT, irreversible/lossy, advanced progression and containers |
| **FluxJXL** | JPEG XL | Bounded Modular lossless greyscale | Colour/alpha, VarDCT, and **exact original JPEG bitstream** reconstruction as its own profile |
| **FluxResample** | General-purpose resizing | Nearest and bilinear with exact geometry/border rules | Bicubic, Mitchell–Netravali, Catmull–Rom, Lanczos, area/box; 7-kernel portfolio |

HTJ2K is part of FluxJ2K, not another mandatory codec repo. The five compression components and one resampler need not release simultaneously. The first cross-repository consumer can be a standalone CLI or a tiny Rust integration harness, never a DICOM-aware dependency in a generic codec.

## 2. Latest accessible Swift source snapshot

**Source repositories directly inspected on 10 November 2026; default-branch commits are immutable input candidates, not proof of accepted clinical or Rust capability.**

| Reference successor | HEAD SHA | Latest release/version described by its repository | Specific limits to preserve |
|---|---|---|---|
| [SwiftJ2K](https://github.com/raster-labs/SwiftJ2K) | `3059a3c87d471028f3524241ec4f881f97cac264` (6 Nov 2026) | `12.1.0` | 2D JPEG 2000/HTJ2K qualified scope; JP3D and JPIP experimental/deferred to 12.2.0 |
| [SwiftJLS](https://github.com/raster-labs/SwiftJLS) | `48b65b291cd17d23d23f182db0bb48fb396c8319` (6 Nov 2026) | `2.0.0` | Distinguish software/simulator evidence from physical-device qualification |
| [SwiftJXL](https://github.com/raster-labs/SwiftJXL) | `2c7638ca9b9c617709bfc9c1f8958eec1e351b47` (6 Nov 2026) | `2.0.0` | Modular/VarDCT/JPEG recompression each have different profiles and guarantees |
| [SwiftJLI](https://github.com/raster-labs/SwiftJLI) | `b6b9f99c3ae79ca3fa5539d14de45543fb88ea36` (6 Nov 2026) | `2.0.2` | SOF3 native lossless distinct from lossy JPEG; qualifications vary by target |
| [SwiftRLE](https://github.com/raster-labs/SwiftRLE) | `4eaaf9139d2f7a836af142b4fdb32f3b9582e005` (30 Oct 2026) | `1.0.1` | PackBits, DICOM RLE descriptor and ownership; physical-device waivers do not imply execution |
| **SwiftResample** | **No current repository/SHA verified** | Accepted seven fixed kernels in programme instructions | Obtain actual latest authorised foundation specification/source before freezing numerical defaults |

The repositories commonly retain **historical “Swift Image Compression Suite” wording**. The owner's current portfolio name is **Swift Imaging Suite (SIS)**, with Compression and Resampling branches. Respect the current name without falsifying historical source filenames or version strings.

**Contract provenance:** [`SwiftRLE/Documentation/SUITE_POLICY.md`](https://github.com/raster-labs/SwiftRLE/blob/4eaaf9139d2f7a836af142b4fdb32f3b9582e005/Documentation/SUITE_POLICY.md) identifies shared API/policy contract **0.10.0**, including independent packages and no mandatory sibling dependencies. [`MEMORY_CONTRACT.md`](https://github.com/raster-labs/SwiftRLE/blob/4eaaf9139d2f7a836af142b4fdb32f3b9582e005/Documentation/MEMORY_CONTRACT.md) labels its contract **0.7.0** and spells out ownership and layout behaviour. These are different document-version labels, not permission to infer two compatible Rust ABIs. Agents must compare the actual seven common contract files and their digests across all five repositories. [SwiftJXL's 2.0.0 release record](https://github.com/raster-labs/SwiftJXL/blob/2c7638ca9b9c617709bfc9c1f8958eec1e351b47/Documentation/RELEASE-v2.0.0.md) states a 7/7 byte-identity check for its qualification snapshot; this is **not our independently executed check**.

The actual Swift repositories declare **Apache-2.0** for their successor code, notwithstanding the new Rust programme's selected **MIT** licence. An owner-held copyright may permit specifically authorised migration/relicensing of owned material, but third-party code, fonts, tables, samples, patents and notices cannot be presumed owned. Prepare a file-level provenance map and legal-rights disposition before adapting source; use standards and independent authored explanations for restricted code.

### First files for each agent to inspect

For each corresponding `raster-labs/Swift*` repo at its pinned SHA: `Package.swift`, `README.md`, `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md` where present, `Sources/<Module>/CodecAPI.swift`, `ImageDescriptor.swift`, `ImageStorage.swift`, `Options.swift`, `Documentation/COMMON_API.md`, `Documentation/MEMORY_CONTRACT.md`, `Documentation/SUITE_POLICY.md`, `Documentation/RELEASE-*.md`, actual codec internals and test fixtures. If a file does not exist in a given repo, record the absence rather than manufacturing it. Pin source, contract, test, fixture and release-gate identities individually.

## 3. Repo structure, independence and shared contracts

Provide independent Rust crates, with a consistent `flux_` crate naming convention; use separate repositories or a segmented workspace according to the owner-authorised bootstrap instruction. Example logical structure:

```text
flux-imaging-suite/        # illustrative workspace, not a mandated umbrella repo
  contracts/               # specs, examples, compatibility tests (not runtime)
  flux-rle/                # crate flux_rle
  flux-jls/                # crate flux_jls
  flux-jli/                # crate flux_jli
  flux-j2k/                # crate flux_j2k
  flux-jxl/                # crate flux_jxl
  flux-resample/           # crate flux_resample
  tools/                   # CLI and cross-codec test harness, optional consumers
```

**Nonnegotiable:** each codec builds, tests and operates without DICOMflux, sibling codecs, a GUI, GPU, Java, Swift, .NET or an umbrella runtime. A tiny neutral descriptor/storage crate MAY be introduced only after proving it reduces contract drift and does not become a feature magnet or mandatory release synchronisation point. Development-only oracles may use other codecs but the shipping implementation may not.

Preserve familiar API *meaning*, not Swift actor/ARC mechanics. First produce a short symbol/semantic equivalence table with verified source names and Rust counterparts. Use Rust `Result`, validated constructors, borrowing, owned leases, safe concurrency and explicit sealed outputs. No public cross-codec API may promise more than a profile can demonstrate.

### Proposed generic Rust API model (illustrative signatures only)

```rust
// Names and signatures are provisional until source inventory and contract review.
struct ImageDescriptor { /* width, height, precision, planes, colour, alpha */ }
struct PlaneDescriptor { /* offset, dimensions, strides, capacity */ }
struct ResourceLimits { /* encoded, decoded, scratch, workers, work limits */ }
enum Fidelity { ExactSamples, BoundedError(u32), Lossy, OriginalBitstream }
enum CopyPolicy { AllowCopies, RequireSharedStorage }
struct OperationReport { /* effective backend, fidelity, copies, allocations */ }
struct DecodedImage { /* sealed owned image + report */ }

trait ImageDecoder {
    fn inspect(&self, source: &[u8], limits: &ResourceLimits)
        -> Result<ImageInfo, CodecError>;
    fn decode_into(&self, source: &[u8], destination: &mut ImageDestination,
                   options: &DecodeOptions) -> Result<OperationReport, CodecError>;
}
```

These examples **do not freeze** a trait object, allocator, error enum or callback ABI. The agent must resolve actual symbols from pinned upstream declarations, propose Rust-idiomatic equivalents and verify at least one independent consumer before publishing a shared API.

### Core memory and value invariants

1. **Descriptors** distinguish width/height, sample type (`u8/u16` and signed variants, with `f32` only when qualified), storage bits, meaningful bits, byte order, pixel/row/plane strides, components, colour/alpha interpretation, offset and retained capacity. Dimensions and last addressed bytes use checked arithmetic. Planes cannot illegally overlap, and writable regions must be exclusive.
2. **Precision** preserves exact logical samples for named lossless paths; signed twelve-bit-in-sixteen-bit data must sign-extend correctly. Padding bytes, high-bit normalisation, per-plane endianness and conversions must be explicit. Do not silently clamp, normalise, invert MONOCHROME1 or apply DICOM rescale/windowing in a generic codec.
3. **Ownership** uses retained immutable source owners and an exclusive writable destination state machine. Borrowed pointers never outlive their access scope; an async worker retains owned backing through actual completion. A sealed result can be shared without another full-frame allocation. Release exactly once through the correct allocator.
4. **Zero-copy claims** are per handoff, not entire pipeline guarantees. A `RequireSharedStorage` option rejects undeclared full-image bridge copies; `AllowCopies` reports reason and byte count. A compressed-domain transcoder avoiding pixel buffers is a separate measurable case.
5. **Budgets** account encoded, decoded, scratch, metadata, queues, concurrent workers and retained allocations. Distinguish per-operation limits from host-wide admission. Reject decompression bombs, overflowing strides, excessive frames, malformed packet lengths and unsupported profile combinations before harmful allocation.
6. **Cancellation** is cooperative. Joining/retaining in-flight native/GPU work is mandatory before destination reuse or release; a failed partially written destination must not become a valid output.

### Common operations and reporting

Each codec SHOULD expose `inspect`, `encode`, `decode`, `decode_into`, `capabilities` and typed configuration/options for its qualified modes. The common CLI vocabulary is `inspect`, `encode`, `decode`, `validate` and `capabilities`, plus codec-specific commands where justified. Do not expose a success stub. Default lossless is valid only if the named codec/profile truly supports lossless; otherwise return explicit unsupported.

Report actual backend (`scalar`, qualified SIMD/device choice), fidelity category, fallback reason, resource peaks, elapsed measurement if collected, copies/conversions and implementation/profile version. Define machine-readable report schema and exit statuses. Do not claim pixel identity from equality of two lossy byte streams or use reconstruction of an existing JPEG file as evidence for encoding fresh pixels.

## 4. Codec-specific implementation orders

### 4.1 FluxRLE — first integration baseline

- Implement standard PackBits with bounded row/chunk processing, exact decoded-length checks, negative/repeat packet semantics, allowed run-length boundaries, truncation detection and no hidden whole-file materialisation.
- Add a **DICOM RLE frame profile** above PackBits: validate 64-byte header, maximum segments, monotonic/aligned offsets, byte-plane order **most-significant byte first** and externally supplied width/height/precision/colour descriptor. DICOM RLE frame APIs do not read a DICOM dataset.
- First profile: greyscale 8/16-bit, explicit signedness and 1..allocated meaningful bits. Require independent vectors for signed/unsigned samples, padded strides and both byte orders. Add colour layouts separately, with raw-cell versus logical-sample guarantees distinguished.
- Inspect [SwiftRLE R2 profile](https://github.com/raster-labs/SwiftRLE/blob/4eaaf9139d2f7a836af142b4fdb32f3b9582e005/Documentation/R2_FRAME_PROFILE.md) before freezing any frame/padding detail.

### 4.2 FluxJLS — second codec candidate

- Implement the normative JPEG-LS predictive/entropy path independently, not through CharLS. First qualified operation pair: 2–16 meaningful bit greyscale lossless with validated codestream/header and caller destination. Expand to near-lossless error bounds and multi-component interleave modes only after the base passes independent known values and two-way interoperability.
- Record whether external DICOM-specific signedness mapping is needed rather than inventing private JPEG-LS metadata. Bound line buffers, restart state, component counts, image sizes and malformed entropy codes. Use independent CharLS **tests/oracles**, not a linked runtime.
- Reference [SwiftJLS v2.0.0](https://github.com/raster-labs/SwiftJLS/tree/48b65b291cd17d23d23f182db0bb48fb396c8319) for API, actual behaviour and limitations. Physical Apple device checks in Swift docs are not portable Rust evidence.

### 4.3 FluxJLI — lossless JPEG first

- Preserve the name's **actual JPEG scope**: SOF3 lossless first, then supported sequential/progressive DCT profiles as separately measured capabilities; the library is not restricted to lossless JPEG. Implement frame/scan/entropy state with bounded DHT/DQT/marker handling, predictors, point transforms and correct precision.
- Do not silently turn a lossless request into high-quality lossy encoding. When JPEG colour semantics or signedness requires an external descriptor, report it explicitly; keep DICOM metadata interpretation in the caller. Add RGB, YCbCr and optional accelerated transforms only with profile tests.
- Reference [SwiftJLI v2.0.2](https://github.com/raster-labs/SwiftJLI/tree/b6b9f99c3ae79ca3fa5539d14de45543fb88ea36); source describes SOF3 lossless and additional 8/12-bit lossy paths. Do not infer unsupported scan/precision combinations from the title alone.

### 4.4 FluxJ2K — JPEG 2000 + HTJ2K

- Begin with checked 2D Part 1 SIZ/COD/QCD/TLM-style header inspection and a reversible 5/3 lossless encode/decode subset. Later qualify HTJ2K's Part 15 block-coding path and original-sample preservation. More elaborate progression orders, precincts, resolution/layer selection, Part 2 MCT and 9/7 lossy profiles are separately admitted capabilities.
- **Reversible Part 1 ↔ HTJ2K compressed-domain transcode** is a separate milestone with exact sample/bitstream invariant definitions. Measure whether pixel allocations truly stay zero; never infer zero-copy just because the operation is named transcode.
- Use [SwiftJ2K v12.1.0](https://github.com/raster-labs/SwiftJ2K/tree/3059a3c87d471028f3524241ec4f881f97cac264) as 2D reference, not as authority to claim JP3D/JPIP production support. Those are held experimental in the cited release record. DICOMflux-3D rendering is not a JP3D codec.

### 4.5 FluxJXL — JPEG XL with distinct preservation classes

- Start with a bounded Modular lossless profile, explicit integer sample precision, validated boxes/codestream headers and exact logical sample tests. Add colour/ICC, animation, VarDCT lossy and accelerated paths only through separately enumerated support gates.
- Treat **existing JPEG → JPEG XL → original JPEG byte reconstruction** as a distinct `OriginalBitstream` profile requiring byte-for-byte restoration from the JXL alone; the original JPEG must not be cached as an undeclared side channel. Separately test normal pixel decode and encoded sample fidelity.
- Reference [SwiftJXL v2.0.0](https://github.com/raster-labs/SwiftJXL/tree/2c7638ca9b9c617709bfc9c1f8958eec1e351b47), including its [`Transcoding.swift`](https://github.com/raster-labs/SwiftJXL/blob/2c7638ca9b9c617709bfc9c1f8958eec1e351b47/Sources/SwiftJXL/Transcoding.swift) and release limitations. Do not link libjxl as shipping implementation.

### 4.6 FluxResample — seven fixed kernels

- Required portfolio: **nearest, bilinear, bicubic, Mitchell–Netravali, Catmull–Rom, Lanczos, area/box**. Begin with nearest and bilinear and explicit pixel-centre / coordinate transforms. Define boundary extension, downsampling antialias, support radius, interpolation precision, round/clamp rules, minimum dimensions, overflow/stride limits and determinism.
- Retain distinction between continuous quantitative images and label masks: nearest/declared label-preserving behaviour for discrete classes; no invented interpolated labels. Keep gamma and colour-space conversions opt-in and separate from numeric image resampling.
- **Source identity caveat:** the current SwiftResample repository/accepted symbol catalogue was **not retrieved** in this review. The agent MUST obtain the owner's latest foundation/specification and revision before freezing bicubic family parameters, Lanczos radii, edge conventions and any Swift-equivalence claim. Continue unrelated codec work; do not invent an API or claim source-parity. Once the source is available, record a parameterised cross-language golden matrix.
- Evaluate SIMD/Accelerate knowledge as methodology, not as a mandatory Apple dependency. Implement the portable scalar path first, then measured x64/ARM SIMD, GPU or platform primitives.

## 5. Staged milestones and verifiable outputs

| Gate | Executable output | Acceptance evidence |
|---|---|---|
| **F0 — Snapshot and contracts** | Six module work orders, pinned Swift-source/source-rights ledger, common image descriptor and Rust consumer example | Exact API differences captured; representative signed/unsigned/padded input bounds test; independent library build proof |
| **F1 — FluxRLE** | PackBits + DICOM RLE first profile; CLI | Known hand-verified vectors, two-way independent oracle, negative/resource tests and exact sample restoration |
| **F2 — FluxResample** | Nearest + bilinear with recorded numerical semantics; extend after source recovery | Analytical interpolation vectors; border/subpixel/stride/determinism cases; label safeguards |
| **F3 — FluxJLS / FluxJLI** | One admitted lossless profile per library; independent CLI consumers | Cross-decoding with independent software; signed/precision/length and malformed entropy tests |
| **F4 — FluxJ2K / FluxJXL** | One independently useful bounded lossless profile each | Independent standards-derived expectations, reproducible bitstream/sample tests, resource contracts |
| **F5 — Cross-codec and DICOMflux integration** | Explicit in-memory handoff and one qualified DICOM-compatible frame profile | Copy-accounted resource report, correct descriptor/metadata boundary, independent decoding via DICOMflux adapter |
| **F6 — Advanced profiles and acceleration** | Individually scoped HTJ2K, VarDCT, lossy/near-lossless, JPEG reconstruction, seven-kernel completion and vendor backends | Per-profile correctness, documented fallbacks, real-device benchmarks and no regression to scalar path |

F0 enables multiple agents to work independently on F1–F4 after contracts are agreed; no codec must wait for the whole portfolio to complete. F5 follows the necessary actual capabilities, not a date. Do not confuse the order of qualification with the order of all exploratory implementation. Each agent returns small executable tests, not a documentation-only scaffold.

### Platform and test acceptance

Primary target matrix: Windows x64 and ARM64, Linux x64 and ARM64 (named distribution/libc), Android ARM64 (`aarch64-linux-android`), plus separately reported Android x86_64 emulator; macOS compatibility secondary. Every released **advertised profile** must have an explicit platform outcome (executed, compiled, not run, unsupported). Real Android ARM64 physical-device validation is required for Android device qualification; build-only/emulator results are not equivalent. Record NDK, toolchain, 4 KB / 16 KB page-size compatibility, package-shared-object identity, JNI ownership tests when a host binding is in scope.

Minimum security and fidelity test classes:

- **Format correctness:** standards valid/minimal/edge inputs, cross-decoding both directions with an independent oracle, invalid marker/segment/runs, incorrect dimensions and inconsistent metadata.
- **Fidelity:** exact logical values for lossless; separate allocated-cell and original-bitstream guarantees where promised. Lossy output is judged against declared quantitative criteria; a lossy-to-lossless re-encode never claims original source recovery.
- **Memory and safety:** fuzzing, checked size arithmetic, decompression amplification caps, row/plane aliasing, partially written output invalidation, cancellation and owner lifecycle.
- **Resources/performance:** true peak allocations, workspace, staging, concurrency and full-operation latency (including copies, GPU transfers and FFI), scalar and accelerated comparison, lower-memory device profile.
- **Interoperability:** standalone library consumers, different compiler/architecture builds, optional C ABI where assigned, no hidden sibling or external codec runtime dependency.
- **Corpus:** offline local versioned fixtures and expected results, hashes and use rights; cloud CI uses a pinned verified copy only.

## 6. Source/rights and release rules

- New Rust code uses the programme's selected **MIT** outbound licence and accurate Raster Images attribution. Every externally submitted copyrightable contribution requires the executed copyright assignment before incorporation. Do not state that signing the DCO is an assignment.
- Swift successor repos inspected here are **Apache-2.0**; their licence and third-party notices are separate from the proposed MIT release. A change of language does not eliminate provenance obligations. Legal/rights review is required for any source adapted instead of independently authored. Do not incorporate copyleft/proprietary implementation code by a superficial rewrite.
- Preserve a per-module source register, C/C++/GPU platform-primitives inventory, exact dependency snapshots, SPDX licence list, notices and vulnerability review. Do not write custom cryptography. Check standards/codec patent declarations before making royalty-free claims.
- Every versioned release carries source commit IDs, compiler/target triples, capability/unsupported matrix, independent validation, performance result conditions, exact CLI commands, API migration notes, licences and SHA-256 integrity records. Checksums prove artifact integrity relative to the accompanying manifest, not publisher identity.
- Do not create workstation, camera, browser or mobile-viewer documentation in this library folder. Rust GPU volume rendering belongs to DICOMflux-3D's own future work package.

## 7. Coding-agent completion report template

```text
MODULE: Flux[...]
WORK ORDER / PERMITTED BRANCH:
UPSTREAM SNAPSHOT: repository + SHA + rights/contract digests
RUST SOURCE ID / TOOLCHAIN / TARGET:
SUPPORTED PROFILES: operation × precision × colour × loss mode × platform
ACTUAL COMMANDS: fmt, clippy, build, test, fuzz/oracle, CLI, benchmarks
RESULT CLASSIFICATION: implemented | compiled | executed | verified | not-run | blocked
INDEPENDENT ORACLE: name, version, test-only path, fixtures, results
BUFFER/ERROR/OWNERSHIP: lease, copies, cancellation, partial output
RESOURCE METRICS: peak retained, scratch, threads, full-operation time
LICENCE/PROVENANCE: author, adaptation disposition, notices, dependencies
GAPS/REGRESSIONS: precise blocker and evidence
NEXT BOUNDED STEP:
ARTIFACT INTEGRITY: filenames + SHA-256
```

## 8. Copy-ready coding-agent commission

```text
You are implementing the assigned Flux Imaging Suite module(s) in Rust.
Read Flux Imaging Suite Rust Coding-Agent Implementation Instructions v0.1.0
and DICOMflux and Flux Imaging Suite Development Manifesto v1.3.1.

Confirm repository authority, current working tree and target work order.
Inspect the exact pinned corresponding Swift successor repository and shared
contracts; verify its latest source identity and release/qualification limits.
The named starting references are SwiftJ2K 12.1.0, SwiftJLS 2.0.0,
SwiftJXL 2.0.0, SwiftJLI 2.0.2 and SwiftRLE 1.0.1 at the recorded SHAs.
SwiftResample source/API identity is NOT yet verified: obtain the owner's
accepted specification before freezing kernel parameters or claiming parity.

Implement a small real Rust codec/resampler capability through its own
library and CLI tests. No wrapper around another production codec engine.
Keep generic imaging entirely independent of DICOMflux and applications.
Preserve meaningful precision, signedness, colour/alpha interpretation,
resource admission, correct ownership and cancellation. Create safe owned
and advanced borrowed/destination APIs with truthful copy/fidelity reports.

Start by qualifying the shared 16-bit greyscale layout, FluxRLE PackBits
and DICOM frame profile, and nearest/bilinear resampling; other modules
can proceed independently after the common contract gate. Make every
unsupported feature fail explicitly, not silently approximate it. Test
independent known values and cross-decoding both directions; a self-round
trip alone is insufficient.

Qualify applicable Windows/Linux x64 and ARM64 and Android ARM64 profiles,
with separately reported Android emulator and secondary macOS builds.
Report unexecuted hardware tests as UNEXECUTED. Maintain Apache-2.0 Swift
reference provenance while distributing authorised new Rust code as MIT.
Never infer copyright ownership or relicense unrelated third-party code.

Return actual reviewed source, tests, measurements, coverage, unresolved
limitations, independent consumer evidence and SHA-256 deliverables. Stop
at the assigned executable acceptance gate, not a new documentation cycle.
```

## 9. Reference links

- [Common DICOMflux/FIS manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view).
- [SwiftJ2K source](https://github.com/raster-labs/SwiftJ2K/tree/3059a3c87d471028f3524241ec4f881f97cac264); [release gate](https://github.com/raster-labs/SwiftJ2K/blob/3059a3c87d471028f3524241ec4f881f97cac264/Documentation/RELEASE-v12.1.0.md).
- [SwiftJLS source](https://github.com/raster-labs/SwiftJLS/tree/48b65b291cd17d23d23f182db0bb48fb396c8319); [SwiftJXL source](https://github.com/raster-labs/SwiftJXL/tree/2c7638ca9b9c617709bfc9c1f8958eec1e351b47); [SwiftJLI source](https://github.com/raster-labs/SwiftJLI/tree/b6b9f99c3ae79ca3fa5539d14de45543fb88ea36); [SwiftRLE source](https://github.com/raster-labs/SwiftRLE/tree/4eaaf9139d2f7a836af142b4fdb32f3b9582e005).
- [SwiftRLE shared API](https://github.com/raster-labs/SwiftRLE/blob/4eaaf9139d2f7a836af142b4fdb32f3b9582e005/Documentation/COMMON_API.md), [memory contract](https://github.com/raster-labs/SwiftRLE/blob/4eaaf9139d2f7a836af142b4fdb32f3b9582e005/Documentation/MEMORY_CONTRACT.md), [suite policy](https://github.com/raster-labs/SwiftRLE/blob/4eaaf9139d2f7a836af142b4fdb32f3b9582e005/Documentation/SUITE_POLICY.md).
- [DICOMkit v3.0.0-described integration](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/CHANGELOG.md) is the consumer reference, not a codec implementation dependency.
- Codec normative standards: ISO/IEC 15444 (J2K/HTJ2K), ISO/IEC 14495 (JPEG-LS), ISO/IEC 18181 (JPEG XL), applicable JPEG/ITU-T T.81/ISO 10918, and [DICOM PS3.5](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/) for the external DICOM RLE frame profile.

*Source-status note:* The source review was bounded to named repository heads, manifests, relevant source and release/contract files. No local Rust build, Swift runtime replay, hardware measurements, ABI conformance, code execution or cross-oracle qualification was carried out for this document.
