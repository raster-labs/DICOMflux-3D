# DICOMflux — Rust Coding-Agent Implementation Instructions

**Document version:** 0.1.0  
**Issued:** 10 November 2026  
**Programme:** DICOM Toolkit & Image Algorithms, Raster Images  
**Status:** Proposed implementation handoff; **no Rust code, build, test or release is claimed by this document**  
**Intended executor:** Separately authorised coding agent supervised by a developer  
**Source of common rules:** [DICOMflux and Flux Imaging Suite Development Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view)  
**Destination:** [DICOMflux documentation folder](https://drive.google.com/drive/folders/1w5e4Td51soP-Agy1WiMzvs44ClSmKR4y)

> **Commission boundary.** This is an execution-ready instruction set, not permission for this assistant to create repositories or implement code. The development owner must identify the authorised repository, branch, agent credentials, source licences, named reviewers and first work order. Once commissioned, execute a real, bounded Rust implementation rather than generating another planning cycle. This work order concerns **DICOMflux the Rust toolkit**, not a workstation, web viewer, DICOM Camera, DICOM Capacitor or DICOMflux-3D application.

## 1. Objective and required separation

Produce a **new, independently implemented, standards-led Rust DICOM toolkit** with an idiomatic Rust API, deterministic resource boundaries, reproducible verification and a deliberately versioned C ABI for later host bindings. Match the **semantics and relevant public operation families** of current project-owned DICOMkit, not its Swift-specific object layout or implementation mistakes. The generic Flux Imaging Suite is a separately implemented dependency for codec and resampling operations; it MUST NOT depend on DICOMflux. Windows/Linux x64 and ARM64 and Android ARM64 are primary targets; Android x86_64 is a separate emulator/CI profile, and macOS is secondary compatibility. No historical C++ DICOMflux or DICOMsteel implementation is a required baseline.

**Do not** import a replacement DICOM engine (dicom-rs, DCMTK, GDCM, dcm4che), call an external decoder as a shipping fallback, create a GUI, or present a prototype as clinically qualified. External projects MAY be independent development-time comparison oracles, subject to rights. Rust-native contracts take precedence over surface spelling compatibility. Preserve provenance whenever authorised project-owned Swift implementation or fixtures are adapted.

### Dependency direction

```text
Desktop / Android / server consumers (out of scope here)
      │ Rust API or small stable C ABI
      ▼
DICOMflux (tags, dataset, Part 10, reader/writer, validation,
           frame-index/encapsulation, optional services)
      │ neutral pixel descriptors and bounded storage operations
      ▼
Flux Imaging Suite (independent codecs / resampling)
```

DICOMflux handles DICOM-specific interpretation and metadata updates. Flux codec crates must never own patient identifiers, transfer-syntax negotiation, study/series logic or DICOM file writing. DICOMflux-3D is a future independent consumer; keep volume/geometry descriptors suitable for it without introducing a 3D renderer here.

## 2. Verified reference snapshot and provenance

**Observed GitHub default-branch heads, 10 November 2026:**

| Source | Immutable source identity | What was directly inspected | Boundary |
|---|---|---|---|
| [`raster-labs/DICOMKit`](https://github.com/raster-labs/DICOMKit) (product wording here: **DICOMkit**) | `9b402fef534c4d6778c36bd7498539fa80b1ee7a` (commit dated 7 Nov 2026) | `README.md`, `CHANGELOG.md`, `Package.swift`, DICOM writer, bounded allocation contract/code, source module inventory | Source describes a v3.0.0 preparation; README/release claims are **not independent Rust evidence** |
| [`raster-labs/SwiftRLE`](https://github.com/raster-labs/SwiftRLE) | `4eaaf9139d2f7a836af142b4fdb32f3b9582e005` (30 Oct 2026) | `COMMON_API.md`, `MEMORY_CONTRACT.md`, `SUITE_POLICY.md`, `CodecAPI.swift`, profile inventory | Cross-repo storage/API reference; library remains independent |
| [DICOM 2026d official consolidated publication](https://dicom.nema.org/medical/dicom/current/) | 2026d at issue | Official publication index | At kickoff **also check later approved Final Text supplements and corrections** |

The DICOMkit repository's current Swift package declares `swift-tools-version: 6.4`, Apple OS 26 floors and modules including `DICOMCore`, `DICOMDictionary`, `DICOMNetwork`, `DICOMWeb`, `DICOMKit` and `DICOMToolbox`. Its v3.0.0 changelog describes integration with **SwiftJ2K 12.1.0, SwiftRLE 1.0.1, SwiftJLS 2.0.0, SwiftJLI 2.0.2, SwiftJXL 2.0.0** across a stated 17-transfer-syntax profile. Those facts establish a **reference inventory**, not an instruction to implement every profile in the first Rust milestone. The changelog explicitly defers JP3D and JPIP integration pending a later SwiftJ2K capability version. Record any discrepancy between default-branch source, tags, releases and qualification records; do not silently resolve it by choosing the largest claim.

**Before the first code change, the agent MUST** re-read the pinned source files and authoritative product contracts, verify target-branch identities, extract the actual DICOMkit public API/signatures, check who owns any source to be adapted and record every consulted upstream revision. DICOMkit's ownership/provenance and external dependency notices must be checked against the intended MIT output. The existing project's published capitalisation does not override the programme's **DICOMkit** wording.

### Priority source paths for direct inspection

- [`Sources/DICOMKit/DICOMParser.swift`](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMKit/DICOMParser.swift), [`DICOMFile.swift`](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMKit/DICOMFile.swift), [`DataSet.swift`](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMKit/DataSet.swift).
- [`Sources/DICOMCore/DICOMWriter.swift`](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMCore/DICOMWriter.swift), [`EncapsulatedPixelData.swift`](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMCore/EncapsulatedPixelData.swift), [`PixelDataDescriptor.swift`](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMCore/PixelDataDescriptor.swift).
- [`DICOMAllocationBudget.swift`](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMCore/DICOMAllocationBudget.swift), [`Documentation/BoundedInput/AllocationContract.md`](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Documentation/BoundedInput/AllocationContract.md), and the actual tests for failure/cancellation/ownership.
- [`Sources/DICOMNetwork/`](https://github.com/raster-labs/DICOMKit/tree/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMNetwork) and [`Sources/DICOMDictionary/`](https://github.com/raster-labs/DICOMKit/tree/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMDictionary) for **subsequent** individually commissioned capability families.

## 3. Bootstrap deliverables and repository layout

Use a new clean Git repository under the owner's nominated GitHub organisation; do not change an existing repository unless explicitly authorised. Start with an intentionally small workspace. Suggested logical layout (adjust only with a documented reason):

```text
dicomflux/
  Cargo.toml                 # pinned edition, toolchain/MSRV and feature policy
  rust-toolchain.toml
  LICENSE                    # MIT, Raster Images copyright as legally accurate
  CONTRIBUTING.md            # copyright assignment gate; no secrets
  SECURITY.md
  crates/
    dicomflux-core/          # Tag, VR, values, owned/borrowed datasets, dictionary
    dicomflux-io/            # Part 10, bounded parsing and writing
    dicomflux-pixel/         # DICOM frame descriptors and encapsulation adapters
    dicomflux-ffi/           # optional versioned C ABI; never in core graph
  tools/dicomflux-cli/       # inspect/validate/convert only when implemented
  tests/fixtures/            # small synthetic or rights-cleared fixtures
  evidence/                  # commands, source pins, test reports, limitations
```

Use `cargo fmt`, `clippy`, `cargo test`, `cargo metadata` and vulnerability/dependency checks. Record the exact stable Rust toolchain, target triples, compiler flags and enabled features at bootstrap. Do not assume a future Rust compiler release number; select a real available version and retain build receipts. Provide an independent consumer outside the workspace to catch accidental private dependencies. No nightly feature is required for the portable core.

No Swift runtime, JVM, C++ codec engine, GUI toolkit, GPU or network service may be required to parse a local DICOM Part 10 file through the minimal core feature profile.

## 4. Public Rust contract — implement in useful slices

### 4.1 Core data model

- `Tag { group: u16, element: u16 }`, VR, VM expectations, transfer-syntax descriptors, value types and dictionary lookups, with checked conversion and explicit unknown/private element handling.
- A **loss-preserving** dataset model for the assigned profile. Record original encoding where required to preserve unknown/private bytes. Separate parsed raw representation from typed convenience access. Do not eagerly copy all bulk pixel data into generic value strings.
- Support sequences and nested items with explicit depth, item/element/byte limits; named handling for defined and undefined lengths and item/sequence delimiters. Private creator resolution has **dataset-item-local** scope. Unknown nonstandard values are not silently reinterpreted.
- Character set decoding, person-name components, date/time and numeric VR checks with explicit strict/compatibility policies. Parsing a value does not establish all IOD/module requirements.
- Concurrency: sealed datasets may be shared immutably; mutation uses explicit transactions or uniquely owned builders. State thread-safety and reentrancy per handle, not as a blanket marketing claim.

### 4.2 Bounded read and Part 10 handling

Start with a single explicitly recorded profile: Part 10 with Explicit VR Little Endian and a declared single-frame greyscale SOP/representation subset. Do not assume every `DICOMkit` format is immediately implemented. Validate preamble and `DICM`, File Meta Information in its required encoding, Transfer Syntax UID, lengths, VR rules, sequence boundaries and bulk offsets. Provide a distinct dataset-only entry point if later commissioned; do not infer a Part 10 wrapper.

- Distinguish **need more bytes** during streaming from **truncated final input**, malformed encoding, unsupported Transfer Syntax, resource budget exceeded and cancellation.
- Require checked arithmetic before allocation and before pointer/slice creation; cap nesting, metadata, value/fragment counts, input bytes and work. Return offsets and safe diagnostic codes, not patient values or identifying paths.
- For bulk data, expose a retained source/owner plus a checked offset/length view. A view must not outlive the source. Map/file-backed reads and seek requirements are explicit capabilities; inaccessible or non-seekable sources return a documented outcome.
- Introduce later transfer-syntax profiles via explicit capability flags and independent test evidence. Never claim a decoder merely because a transfer syntax UID is in the dictionary.

### 4.3 Writer, transformations and validation

Implement strict counterpart writing for the admitted profile: correct element headers and extended-length VRs, even-length padding, endianness of numeric value units, nested sequences/items, consistent Part 10 meta headers and Transfer Syntax. Publication to a file should use safe temporary output and atomic replacement where the host supports it; no complete output may be reported after partial failure.

For transformed objects, introduce an explicit `TransformationEffect`/derivation record. Where Pixel Data, dimensions, signedness, photometric interpretation, Transfer Syntax, lossy history, source references or instance UIDs change, the DICOM-specific layer validates and updates all applicable attributes. Reject a transformation when that obligation cannot be fulfilled; never persist pixels under stale metadata. Preserve unknown/private content only to the extent promised by the operation.

Treat **syntactic parse**, **semantic dataset validation**, **IOD/SOP-class validation**, and **DICOM service conformance** as separate capability profiles. At first, return a bounded structured diagnostic report for the implemented subset; do not market the module as universally DICOM-conformant.

### 4.4 Pixel/codec adapter boundary

A frame is not simply an encapsulated fragment. For compressed transfer syntaxes, validate Basic and Extended Offset Tables, frame grouping, fragment boundaries, declared compression profile, expected sample precision and component mapping. DICOM RLE's 64-byte segment offset header is not a dataset or image descriptor. The Rust adapters may select FluxRLE or another independently qualified Flux codec **only when the codec is ready**.

Provide an explicit adapter contract with `ImageDescriptor` / `PlaneDescriptor`, source precision, signedness, geometry, ownership and `OperationReport`. Avoid nonessential full-frame copies and expose copies that do occur. No codec may apply DICOM windowing, rescale or MONOCHROME1 inversion implicitly. Distinguish raw logical samples, modality-transformed values and presentation pixels.

Start with uncompressed frames and only then add compressed profiles. The initial integration candidate is DICOM RLE lossless, followed by JPEG-LS lossless. JPEG 2000/HTJ2K, JPEG XL and JPEG lossless/lossy integration follow separately qualified profiles and the agreed cross-codec contract. Transcoding must not claim original compressed-byte preservation unless that exact guarantee is proved.

## 5. Resource, security and foreign-interface rules

- Define a `ResourceLimits` and budget ledger with separate allocation classes for encoded input, decoded output, staging, resident/retained storage, preview and metadata. DICOMkit's bounded budget contract is a **behavioural design reference**, not a requirement to reproduce its Darwin memory mapping or fixed default megabyte values on every Rust target. Host aggregate admission, retained capacity, leases, shared-owner retirement and in-flight workers require explicit accounting. Name whether a limit is per operation, instance or host.
- Any retained output buffer releases **exactly once** through its owner after all consumers and hardware have finished. For caller-provided destinations use reserve/write/seal or an equivalently safe Rust lifecycle; cancellation after writing begins must not expose a complete image. Bounded synchronous APIs should exist independently of a particular async executor.
- Make an idiomatic Rust API first. The C ABI uses opaque handles, fixed-width fields, lengths, version negotiation, explicit release functions and stable error categories. Do not cross the ABI with Rust `Vec`, `String`, traits, panics or references. Catch unwinding where meaningful; aborting panics cannot be caught.
- Primary target profile: Windows `x86_64-pc-windows-*`, Windows ARM64, Linux x86_64 and aarch64 (specific libc), Android `aarch64-linux-android`; Android x86_64 emulator independently. Confirm target/linker, dynamic-library packaging, Android 16 KB page-size compatibility and physical-device execution. Keep native/OS qualifications separate. macOS compatible builds are secondary and do not gate the initial primary-platform milestone.
- Fuzz untrusted parser and encapsulation inputs. Avoid `unwrap`/`expect` reachable through malformed input. No implicit network, telemetry, shell or remote retrieval. Use maintained TLS/crypto libraries if network services are assigned later; do not build TLS.
- Keep the core **MIT** with accurate owner attribution. External Swift successor repositories declare **Apache-2.0**; project-owned algorithm adaptation needs documented rights, while third-party notices and upstream patent/licence issues require separate review. External contributions must have the owner's executed copyright assignment before incorporation. Preserve an SBOM and exact source revision list.

## 6. Milestones and acceptance gates

| Gate | Work order output | Pass criteria (executed evidence, not plans) |
|---|---|---|
| **D0 — Source and executable skeleton** | Real repository, Rust toolchain, source register and target matrix, small `Tag`/VR example and CLI startup | Clean build/test on an actual primary host; independent consumer; format/lint/CI configured; source and notices identified |
| **D1 — Bounded Part 10 reader** | Explicit VR Little Endian, selected SOP/profile, dataset/sequence access and source leases | Independent goldens plus truncation/nesting/overflow/unknown-tag/charset/partial-stream negatives; no unexpected allocations or panics |
| **D2 — Writer and validation** | Correct Part 10 writer, safe output policy, bounded validation report | Independent decode of outputs; exact sample and intended metadata equivalence; negative and partial-output tests |
| **D3 — Pixel frame / Flux integration** | Uncompressed frame path then one qualified lossless compressed Transfer Syntax | Source signedness/precision; frame/fragment/offset tables; independent codec oracles; no silent precision or metadata change |
| **D4 — Host consumption** | Rust public API and minimal C ABI exercised externally, plus Android/desktop native loading | Version negotiation, buffer ownership, cancellation/reuse and cross-architecture checks, real ARM64 device result |
| **D5 — Additional services** | Later **individually authorised** DIMSE, DICOMweb, de-identification, media or advanced validation | Protocol conformance, adversarial input/network tests and service-specific evidence; not implied by D0–D4 |

**Do not wait for D5 to release a useful independently qualified foundation.** The DICOMkit 17-transfer-syntax source inventory is a target for a staged capability map, not a single release gate.

### Minimum test matrix for each advertised profile

1. Standards-derived valid and edge cases; DICOM 2026d + applicable approved Final Text at kickoff.
2. Exact logical samples for lossless paths, including signed 12-in-16, MONOCHROME1 handling at the adapter/presentation boundary, padding and unusual strides.
3. Nested item/private creator tests; invalid lengths, encodings, duplication, truncated/corrupted compressed frames, overflow and resource-limit refusal.
4. Dataset semantic preservation independent of Part 10 byte identity; separate exact-byte tests where promised.
5. Explicitly cancelled operations; no use-after-free, double release, stale output or release before workers finish. Run sanitizers/fuzzing where supported and report omissions.
6. Interoperability with at least one independent external *test oracle* per admitted encoding direction; include self-round-trips only as supplementary evidence.
7. Actual Windows/Linux x64 and ARM64 plus Android library/binding physical-device runs where required by the assigned release profile; no simulation substituted for device evidence.

Keep the locally stored rights-cleared regression corpus internally versioned with source IDs, hashes, generators, oracles and licences. Hosted CI may fetch a pinned verified copy; normal development should work offline.

## 7. Required completion report

At the end of *each* gate, return: repository URL/branch/commit; precise implemented operation/profile list; API equivalence/differences table against pinned DICOMkit source; dependency and licence register; actual commands and stdout/stderr outcomes; executable tests, skipped/blocked tests and platform identities; memory/time and worker budgets; malformed-data/ownership results; unresolved items; a small next work order; release files and `SHA256SUMS.txt` or equivalent checksums. Separate **implemented**, **compiled**, **executed**, **verified**, **not run** and **blocked**. Do not cite a Swift test result as evidence that Rust passed.

**Hard stop conditions:** missing rights for copied source, unverifiable normative requirements, an ownership or ABI unsafety that cannot be bounded, incorrect sample fidelity, or an attempted change to unrelated existing repositories. Isolate the affected work and continue independent safe slices where possible. No clinical-use, standard-wide conformance or performance claim is authorised by this handoff.

## 8. Copy-ready coding-agent commission

```text
You are the developer/coding agent for DICOMflux, a new Rust DICOM toolkit.
Read DICOMflux Rust Coding-Agent Implementation Instructions v0.1.0 and
DICOMflux and Flux Imaging Suite Development Manifesto v1.3.1 in full.
Confirm the authorised Git repository, target branch, and approved work order.

Inspect the exact current DICOMkit sources and source/qualification records,
starting with raster-labs/DICOMKit @
9b402fef534c4d6778c36bd7498539fa80b1ee7a, and inspect the source
register and actual latest standard. Report any newer revisions or disputes.
Do not infer API symbols from the handoff or assume documentation-only
claims are executable evidence. Verify third-party licences and preserve
source lineage before any adaptation of project-owned Swift code.

Bootstrap a small idiomatic, independent, MIT-licensed Rust implementation.
Complete D0, then D1: a bounded, validated Part 10 read of an explicitly
admitted Explicit VR Little Endian profile, with tests, CLI and exact
resource/ownership contracts. Use independently justified expectations.
Make all unsupported operations fail explicitly. Never ship a foreign
DICOM parser/codec fallback. Do not build a workstation or GUI here.

Prepare the D2/D3 handoff after reporting actual D1 evidence. Keep codecs
independent in Flux Imaging Suite. Windows and Linux x64/ARM64 and Android
ARM64 are primary targets; report compile-only and device-unexecuted cases
honestly. Do not change other repos without explicit authority.

Deliver reviewable source, API/evidence/provenance updates, exact commands,
real observed results, source/fixture hashes and SHA-256 manifests. Stop
when the assigned gate is genuinely satisfied or a precise blocker is found.
```

## 9. Reference register (snapshot, not an exhaustive inspection)

- [DICOMflux/Flux Imaging Suite Development Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view).
- [DICOMkit source snapshot](https://github.com/raster-labs/DICOMKit/tree/9b402fef534c4d6778c36bd7498539fa80b1ee7a); [changelog](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/CHANGELOG.md); [Part 10 writer](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Sources/DICOMCore/DICOMWriter.swift); [budget contract](https://github.com/raster-labs/DICOMKit/blob/9b402fef534c4d6778c36bd7498539fa80b1ee7a/Documentation/BoundedInput/AllocationContract.md).
- [SwiftRLE common API](https://github.com/raster-labs/SwiftRLE/blob/4eaaf9139d2f7a836af142b4fdb32f3b9582e005/Documentation/COMMON_API.md) and [memory contract](https://github.com/raster-labs/SwiftRLE/blob/4eaaf9139d2f7a836af142b4fdb32f3b9582e005/Documentation/MEMORY_CONTRACT.md).
- Official DICOM [current publication](https://dicom.nema.org/medical/dicom/current/), [PS3.5](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/), [PS3.6](https://dicom.nema.org/medical/dicom/current/output/chtml/part06/), [PS3.10](https://dicom.nema.org/medical/dicom/current/output/chtml/part10/), and [status](https://www.dclunie.com/dicom-status/status.html).
- [Rust API guidelines](https://rust-lang.github.io/api-guidelines/); [Rust target support](https://doc.rust-lang.org/rustc/platform-support.html); [Android ABI guidance](https://developer.android.com/ndk/guides/abis); [16 KB page-size guidance](https://developer.android.com/guide/practices/page-sizes).

*Release note:* this document is a **proposal** for the first executable development work order. Current source inspection was read-only and bounded, and no Rust implementation or platform test has been run in preparation of this handoff.
