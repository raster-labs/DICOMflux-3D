# DICOMflux — D0 Foundation, Repository Bootstrap and Core Contracts

**Work order:** D0 · **Version:** 0.1.0 · **Issued:** 10 November 2026 · **Status:** PROPOSED FOR OWNER / ENGINEERING ADMISSION; NOT AN EXECUTION RESULT  
**Owner:** Raster Images Private Limited · **Intended executor:** separately authorised Rust coding agent · **Licensing target:** MIT, subject to the rights review below  
**Publication:** [DICOMflux library folder](https://drive.google.com/drive/folders/1w5e4Td51soP-Agy1WiMzvs44ClSmKR4y)  
**Governing rulebook:** [DICOMflux and Flux Imaging Suite Development Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view)  
**Parent handoff:** [DICOMflux Rust Coding-Agent Implementation Instructions v0.1.0](https://drive.google.com/file/d/1KyiALgcCrUzwiOM4RLFKZSq2f3JPxwWw/view)

> **Commission boundary:** This work order is an instruction for a *separately authorised* coding agent. Preparing or approving this document does not authorise the present assistant to create GitHub repositories, commit implementation code, run unapproved agents, or make clinical-use claims. The implementing owner must provide repository identity, branch/worktree, execution permissions and named reviewer before code actions. Implement only D0, not D1 or the application portfolio.

## 1. Outcome and dependency position

Produce a small, independent Rust DICOM foundation with validated core types, bounded input abstractions, explicit errors and lifetimes, deterministic resource limits, a minimal public C ABI, a functioning command-line capability probe and independent tests. D0 is **not** a complete DICOM parser. D1 will consume D0 without redefining its foundational contracts. D0 and Flux Imaging Suite F0 can proceed in parallel; neither is a prerequisite of the other.

```text
D0: tags / VRs / identifiers / bounded sources / errors / budget / ownership / ABI
  └── D1: Part 10 reader and dataset interpretation (separate commission)
F0: independent generic imaging contracts (parallel; no D0 dependency)
```

The Rust toolkit is separate from DICOMflux-3D, DICOM Capacitor and all workstation, mobile and browser applications. It must run without a GUI, JVM, GPU, proprietary SDK or any sibling Flux codec. The first supported library targets are Windows x64/ARM64, Linux x64/ARM64 and Android ARM64 (with Android x86_64 as a separate emulator profile); macOS remains secondary.

## 2. Exact source authority, precedence and licence

1. Follow the owner's instructions in this work order, then the governing manifesto v1.3.1 and the predecessor handoff. Do not expand the mandate from source-tree feature names or an unreviewed change on a development branch.
2. Use DICOMkit (repository spelling `raster-labs/DICOMKit`) **main** at `e127b5e9c69629eea8d45cb1395027b113f29985`, observed 10 November 2026; its published release is `v3.0.0` (7 November). Study `Sources/DICOMCore/Tag.swift`, `DataElement.swift`, `DICOMByteBuffer.swift`, `DICOMAllocationBudget.swift`, `DICOMOutcomes.swift`, `DICOMManagedFailure.swift`, and `Sources/DICOMKit/DataSet.swift` and `DICOMFile.swift`. The source uses Swift 6.4 and Apple OS 26; neither restriction is imposed on Rust.
3. At kickoff verify HEAD, tags, licenses, exact files and ancestry, then record a frozen implementation-reference commit. If a newer commit is materially relevant, analyse it before adoption, without silently changing the work-order target. The attached source register is an *inspection snapshot*, not an approval of every behaviour.
4. The official consolidated DICOM publication index currently identifies **2026d** (21 September 2026). Pin the applicable PS3.5, PS3.6 and PS3.10 documents and any approved Final Text changes; retain edition, clause and SHA-256 of the locally obtained normative sources. At review, some moving HTML pages still displayed `2026c`: agents must reconcile against the 2026d snapshot before treating clause wording as frozen.
5. The current DICOMkit repository licence file states **MIT**; verify ownership of copied/adapted paths and any included third-party code or data. The Rust output is MIT for programme-owned work; preserve original authorship and notices. External copyrightable contributions require a legally reviewed assignment before merging. Never import an external DICOM engine as a production shortcut. Record independent implementations versus project-owned adaptations precisely.

Reference source observations and source-use limitations are listed in `DICOMflux_Work_Order_Source_Register_v0.1.0.md` in this folder.

## 3. Included requirements — D0

| Requirement | Implementing-agent obligation | Observable output |
|---|---|---|
| **D0-REQ-001** | Bootstrap an independently consumable Rust package under the authorised `raster-labs` repository. Pin stable Rust edition, compiler/MSRV policy, Cargo lock/feature rules and target triples. | Reproducible clean build and external-consumer manifest; no accidental applications/codec dependencies. |
| **D0-REQ-002** | Model `Tag { group: u16, element: u16 }` and stable ordering, formatting, parsing, equality and hashing with no data-dependent panics. | Known tag bytes, ordering, invalid input and round-trip unit cases. |
| **D0-REQ-003** | Represent all relevant DICOM VR designators from the selected PS3.5/PS3.6 baseline, with recorded length/header rules; make unknown future VR handling explicit. | VR inventory from standard; test short/long-length classification and unknown handling. |
| **D0-REQ-004** | Define `ByteOrder`, validated UID, person-name and elementary value representations needed for D1 without conflating encoded bytes and semantic values. | Exact-value tests for endian, UID component rules and signed numeric boundaries. |
| **D0-REQ-005** | Provide a bounded `ByteSource` / cursor with checked `u64` offsets, exact reads/slices and explicit owned or retained borrowing. No mandatory whole-file copy. | Zero/EOF/overflow/unavailable-range tests; resource and lifetime proofs. |
| **D0-REQ-006** | Define finite `ResourceLimits`, per-operation accounting and optional host-level aggregate admission. Count backing allocations, caches, metadata and retained owners as declared; no claim of process-wide RSS guarantees. | Reservation/refusal/rollback, aggregation and concurrent-instance tests. |
| **D0-REQ-007** | Define typed failure categories (`InvalidArgument`, `MalformedInput`, `TruncatedInput`, `Unsupported`, `ResourceLimit`, `Cancelled`, `Io`, `Internal`) with deterministic context and no implicit patient-data logging. | Stable error mapping and negative tests; no panic from untrusted input. |
| **D0-REQ-008** | Define a synchronous, runtime-independent cancellation/deadline poll contract and worker/reentrancy boundaries; detached work may only retain owned/leased buffers. | Cancellation-before/during-operation and buffer-retirement tests. |
| **D0-REQ-009** | Define the ownership model for immutable byte spans, opaque bulk-data references and dataset element/value handles, even though D1 performs actual reading. Preserve backing until the last consumer releases it. | Direct owner, alias, close and use-after-close rejection cases. |
| **D0-REQ-010** | Publish a minimal **versioned C ABI** for capability/version querying, opaque owned handles and safe status reporting; no Rust `Vec`, `String`, references, trait objects or unwinding cross the boundary. | C header, independently linked C consumer and explicit release/allocator tests. |
| **D0-REQ-011** | Provide a real `dicomflux version`/`capabilities` CLI surface and reserved command diagnostics; do not pretend `inspect` is implemented in D0. | Executed CLI tests with stable machine-readable schema and exit codes. |
| **D0-REQ-012** | Establish tests, lint, CI, contribution policy, security contact, dependency/notice inventory and SHA-256 release-evidence templates. | Reviewer-readable source/build provenance and an executable external consumer. |

A proposed code organisation is `dicomflux-core` (pure library), a small `dicomflux-ffi` boundary and `dicomflux-cli` (application of the library); the exact Cargo package naming is a D0 implementation decision unless publication naming is already authorised. Keep the graph acyclic and avoid allocating a family of empty placeholder crates. `Tag`/VR/byte-cursor code should not depend on the FFI or CLI.

### Core contract notes to make decisions executable

- **Tags and VRs:** Reject invalid textual tags and unsupported VR headers explicitly. A DICOM tag is two 16-bit unsigned components; group 0002 File Meta Information remains a distinct domain in D1. Do not hard-code DICOM dictionary data based on unverified generated tables.
- **UIDs:** Validate the PS3.5 UID syntax and encoded-length constraints; preserve original encoded bytes where a future reader requests fidelity. A UID's semantic comparison is not permission to invent a new UID or trim malformed bytes silently.
- **Typed numeric values:** Retain raw encoded representation and byte order, including signedness and width; do not use lossy floating-point conversion to store arbitrary 64-bit integers.
- **Cursor/offsets:** Use checked arithmetic before computing end-of-range (`length <= source_length - offset` only after validating offset), perform bounded reads and distinguish `NeedMoreInput` for incremental input from `TruncatedInput` at final input.
- **Memory:** Document allocation classes and accounting scope. Historical DICOMkit `DICOMAllocationBudget` page-backed details are a reference, not a mandate to reproduce Apple-specific memory allocation in Rust. Do not label a slice zero-copy if it forces materialisation or an unreported copy.
- **Cancellation:** An exception, a Rust future drop or a foreign-language wrapper release is not evidence that underlying work has finished; in-flight owner leases remain valid until actual completion.
- **C ABI:** Provide an explicit ABI version, capability negotiation and status categories; null/invalid pointer arguments must be rejected at the safe perimeter to the extent possible. No foreign exception or Rust unwind may cross. Do not promise safe validation of arbitrary dangling foreign pointers.

## 4. Exclusions and negative scope

D0 explicitly excludes a complete Part 10 reader, DICOM writer, dictionary generation beyond the minimum validated VR/tag vocabulary, association/DIMSE/DICOMweb, media/DICOMDIR, de-identification, object/IOD validation, codecs, RLE, resampling, window/level presentation, 3D, UI, application access policy and product claims. A nonfunctional stub must not be reported as an implemented capability. Avoid unrelated CI, repository or external service modifications.

## 5. Execution sequence and authorised work products

1. **Preflight:** identify the actual authorised Rust repository/branch and contents; record working-tree state and rights clearance. Pin standard snapshot, DICOMkit source identity, licence notices and baseline Rust compiler/target toolchains. If repository authority is absent, return a source-and-gate report without creating a new repository.
2. **Core first:** implement validated `Tag`, VR classification, endian and UID/value types with hand-checkable unit cases. Record explicit decisions and unsupported forms.
3. **Bounded bytes:** implement the checked source/cursor and ownership/resource ledger, then verify sequential, random-access, empty and overflow cases.
4. **Foreign surface:** implement only the minimal version/capability/opaque-handle ABI needed to prove independence and allocator ownership, with C header and a consumer outside the Cargo workspace.
5. **CLI and evidence:** implement version/capability commands, tests, sanitiser/fuzz plan where supported, security/notice records, CI matrix and completion receipts. Do not advance into D1 by default.

Each stage must leave the working tree reviewable and maintain compiling independent consumers. No new dependency is admitted merely because DICOMkit had an equivalent Apple framework.

## 6. Target and build matrix

| Profile | Required evidence at D0 | What does *not* count |
|---|---|---|
| Windows x86_64 | Build, core/ABI consumer, native CLI tests on a named Windows version | Linux compilation or Wine alone |
| Windows aarch64 | Native ARM64 build and executable consumer/test run | Emulated x64 process |
| Linux x86_64 | Build and real execution on selected glibc distribution | Host-only unit test if another target claimed |
| Linux aarch64 | Native ARM64 execution on named distribution/ABI | QEMU run described as real hardware |
| Android arm64-v8a | NDK-targeted `aarch64-linux-android` library build and real device execution of a minimal native/host consumer | `cargo check` alone, simulator or JNI app scaffolding |
| Android x86_64 | Separate Android-target build and emulator smoke evidence where commissioned | Treating desktop Linux x64 artefact as Android output |
| macOS | Optional compatibility build without making Apple devices a D0 primary gate | Calling it first-release-required |

The work order does not freeze a minimum Android API level. Pin it in the implementation record from intended devices; use a recent stable Rust/NDK toolchain and record exact hashes. A full Kotlin/JNI AAR is a later host-adapter deliverable, not a covert requirement inside D0.

## 7. Verification plan — commands and independent expectations

These commands are **for future execution by the authorised agent** after concrete package names and targets are pinned. Never report them as already run.

```sh
rustc --version --verbose
cargo --version
cargo metadata --locked --format-version 1 > evidence/cargo-metadata.json
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo test --manifest-path examples/external-c-consumer/Cargo.toml --locked
cargo build --release --locked --target x86_64-pc-windows-msvc
cargo build --release --locked --target aarch64-pc-windows-msvc
cargo build --release --locked --target x86_64-unknown-linux-gnu
cargo build --release --locked --target aarch64-unknown-linux-gnu
cargo build --release --locked --target aarch64-linux-android
cargo build --release --locked --target x86_64-linux-android
# Run the relevant test/consumer binaries on each named real target.
# Record absent cross toolchains or linkers as BLOCKED / NOT RUN, never PASS.
```

The independent C consumer may be built by a C toolchain rather than Cargo; the example path above is illustrative and must not become a misleading test if there is no Cargo package there. Capture actual commands, exit status, compiler/SDK, target and fixture identity rather than fabricate a generic test transcript.

**Hand-checkable expected cases:** `Tag(0x0028,0x0010)` orders before `(0x0028,0x0011)`; little-endian bytes `[0x34,0x12]` read as `0x1234`, big-endian as `0x3412`; `u64::MAX` offset plus 1 fails without wrapping; a range ending exactly at source length succeeds, one byte beyond fails; a cancelled operation retains any already-active memory lease until its worker finishes; duplicate release is rejected by the ABI protocol. Test malformed UID components and precise error categories. None of these tests alone demonstrates a DICOM parser.

## 8. D0 acceptance gates

| Gate | Observable pass condition | Required evidence | Failure disposition |
|---|---|---|---|
| **D0-G01** | Authorised source and MIT/provenance review recorded; standard and upstream references pinned. | Snapshot/rights register with exact SHAs and notices. | BLOCKED for affected code reuse. |
| **D0-G02** | Core types and bounded input pass positive, malformed, edge/overflow and cancellation cases. | Commands, machine-readable test output and independent hand-calculated fixtures. | FAIL; D1 not admitted. |
| **D0-G03** | Ownership and resource ledger reject invalid aliasing and release only after actual completion. | Lifetime/concurrency tests; documented `unsafe` reviews. | FAIL; do not issue ABI stability claim. |
| **D0-G04** | C header and consumer use only versioned, opaque, nonunwinding ABI and matching allocator. | External C build/run, ABI version and negative tests. | FAIL; freeze pending correction. |
| **D0-G05** | Actual CLI responds to version and capabilities; unknown/reserved operations fail truthfully. | Process exit codes, stdout/stderr schema tests. | FAIL. |
| **D0-G06** | Primary target builds and real executions are accounted for individually. | Per-target compiler, binary, toolchain, device and test receipts. | NOT RUN/ BLOCKED for missing environments; no blanket multi-platform pass. |
| **D0-G07** | Release record has requirement trace, SBOM/notices, SHA-256 and reviewer disposition. | Signed-off evidence table, exact artifact hashes. | INCOMPLETE. |

**Admission distinction:** A code-correct host development slice may be ready for D1 experimentation once D0-G02 through G05 pass, **but D0 is not declared fully multi-platform accepted** until required D0-G06 profiles are qualified and G01/G07 are closed. A named owner may authorise constrained continuation with unexecuted platforms disclosed; no silent waiver is permitted.

## 9. Performance and security budgets

The agent must propose and obtain review of quantitative limits before implementation: source size, maximum slice size, number of retained handles, metadata bytes, budget granularity, number of workers, and any CLI response-size ceiling. D0's objective is bounded correctness, not a speculative throughput number. Measure resident memory separately from library-charged allocation; record whether the input source is borrowed, mapped or copied. Fuzz tag/VR/UID decoders, byte ranges and ABI scalar arguments with a fixed seed/corpus budget. Review each `unsafe` block and all paths from untrusted input. Do not implement TLS, cryptography or a new filesystem sandbox.

## 10. Known issues, decisions and reviewers

- **D0-DEC-01:** pin the stable Rust version, edition/MSRV, Windows CRT/ABI, Linux libc and Android minimum API with evidence. Proposed, not owner-approved numeric values.
- **D0-DEC-02:** choose one minimal core/FFI/CLI package layout; check whether an authorised repository already exists before creating anything.
- **D0-DEC-03:** freeze v0.1 C ABI field widths, status codes, handle release and negative-test behaviour in the agent's implementation contract.
- **D0-DEC-04:** record source licensing per exact path, including old Swift releases, despite current DICOMkit HEAD being MIT.
- **D0-DEC-05:** decide whether any external fixture or dictionary generator can be redistributed after rights review.

The repository owner, independent verifier, platform qualification owner and legal rights reviewer must be named in the commission record. No owner decision is represented as already approved merely because a candidate appears here.

## 11. Required coding-agent completion report

Supply `D0_COMPLETION.md`, `D0_REQUIREMENTS_EVIDENCE.md`, `D0_SOURCE_PROVENANCE.md`, `D0_TARGET_MATRIX.md` and `SHA256SUMS.txt`, or an equivalent compact, cross-linked set. Every line must distinguish **IMPLEMENTED / BUILT / EXECUTED / VERIFIED / NOT RUN / BLOCKED**. Include exact revision/branch, source/dictionary/fixture hashes, commands, machine/toolchain versions, tests, failure cases, memory measurements, ABI exports, `unsafe` inventory, rights/notice dispositions and the next bounded step. Do not claim clinical, vendor or platform qualification beyond executed observations.

## 12. Definition of done and reviewer sign-off

D0 is ready for review only after G01–G07 have individual receipts. The independent reviewer must verify the source tree and external C consumer against actual build logs and negative tests. The DICOMflux owner then records **ADMIT / ADMIT WITH EXPLICIT DEFERRED PLATFORM GATES / RETURN FOR CORRECTION**. An admission here does not authorise D1 implementation; a separate D1 commission is required.

## 13. Copy-ready prompt for the implementing D0 agent

```text
You are the separately authorised DICOMflux D0 Rust coding agent. Work only in the
repository and branch named by the developer's execution assignment. Read the
DICOMflux Development Manifesto v1.3.1, DICOMflux Rust Coding-Agent Instructions
v0.1.0, DICOMflux D0 Work Order v0.1.0 and its exact source register. Verify
the selected DICOMkit and normative-source commits/licences at kickoff.

Implement D0-REQ-001..012: safe Rust fundamental DICOM types, checked bounded
byte-source/cursor, explicit ownership and resource ledger, cancellation,
typed errors, minimal versioned C ABI, real version/capabilities CLI and
independently executable tests. Do not implement the D1 parser, codecs,
networking or applications. Do not wrap another DICOM engine.

Implement the smallest working slices, not unconnected scaffolding. Preserve
DICOM value meaning, reject unsupported inputs and avoid patient data in
logs. Use a supported stable Rust toolchain, document MSRV/features and do
not allow panics or exceptions across C ABI. Build an independent consumer.

Execute only commands permitted by the repository commission. Record actual
commands, logs, target devices, SHA-256 and test/qualification results. Report
missing physical targets as NOT RUN, never PASS. Submit evidence for D0-G01..07,
rights review and a D0 completion report. Stop at D0; request separately
issued D1 authority before implementing the Part 10 reader.
```

## 14. Reference links

- Governing manifesto and DICOMkit handoff: URLs in document header; full source register accompanies this work order.
- Current implementation reference: [DICOMkit source](https://github.com/raster-labs/DICOMKit/tree/e127b5e9c69629eea8d45cb1395027b113f29985), especially bounded input, core values and allocation contracts.
- Normative: [DICOM official current publication index](https://dicom.nema.org/medical/DICOM/current/); PS3.5 Data Structures and Encoding, PS3.6 Data Dictionary, PS3.10 Media Storage and File Format.
- Rust: [Rust platform support](https://doc.rust-lang.org/rustc/platform-support.html), [Cargo rust-version](https://doc.rust-lang.org/cargo/reference/rust-version.html), [Rust FFI reference](https://doc.rust-lang.org/reference/items/external-blocks.html).

**Status reminder:** Work order authored; D0 implementation, compilation, platform testing and acceptance are **not executed by this documentation commission**.
