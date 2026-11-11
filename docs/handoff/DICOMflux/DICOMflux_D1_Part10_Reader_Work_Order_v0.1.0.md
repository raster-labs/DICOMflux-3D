# DICOMflux — D1 Bounded Part 10 Reader and Dataset Interpretation

**Work order:** D1 · **Version:** 0.1.0 · **Issued:** 10 November 2026 · **Status:** PROPOSED, DEPENDENT ON D0 ADMISSION; NOT IMPLEMENTED  
**Owner:** Raster Images Private Limited · **Executor:** separately authorised Rust coding agent · **Licence:** project-owned MIT with exact-source provenance  
**Folder:** [DICOMflux](https://drive.google.com/drive/folders/1w5e4Td51soP-Agy1WiMzvs44ClSmKR4y) · **Governing:** [Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view)  
**Parent instruction:** [DICOMflux Rust Coding-Agent Implementation Instructions v0.1.0](https://drive.google.com/file/d/1KyiALgcCrUzwiOM4RLFKZSq2f3JPxwWw/view)

> **Authority:** Documentation-only preparation. No source implementation or testing is authorised by writing this order. A separate commission must name the D0 revision accepted for D1, the repository/branch, permissions, datasets and reviewers. Do not quietly substitute incomplete D0 types or import a third-party DICOM reader to meet this gate.

## 1. Objective, predecessor and measurable vertical slice

Implement a **strict, bounded, standards-driven DICOM Part 10 reader** which parses the admitted transfer syntax, indexes the dataset with valid element/sequence nesting, exposes typed scalar metadata and retained bulk-data spans, and supports a real CLI inspector. It must work on a small rights-cleared, valid file and fail predictably on malformed files. This is a reader, not a decoder, writer or full IOD validator. A recognised Transfer Syntax is not proof that pixels can be decompressed.

**Predecessor:** D0 core and ABI contracts, with a pinned accepted revision and disclosed platform exceptions. D1 must consume D0 as published; public-breaking changes require a coordinated D0/D1 contract review. A software-correct D1 host test does not erase D0's missing physical-device gates.

## 2. Reference and normative priority

- **Normative authority:** DICOM PS3.10 2026d, especially chapter 7 and File Meta Information; PS3.5 2026d sections 6.2, 7.1, 7.5, 7.8, 10 and Annex A as applicable; PS3.6 2026d dictionary and Transfer Syntax UIDs. Pin 2026d source/archives and relevant approved Final Text changes at work-order kickoff. The official current index was checked 10 November 2026; some moving HTML chapter pages displayed 2026c, so retain the exact 2026d bytes/digests before implementing edition-sensitive rules.
- **Reference implementation:** [DICOMkit `e127b5e9c69629eea8d45cb1395027b113f29985`](https://github.com/raster-labs/DICOMKit/tree/e127b5e9c69629eea8d45cb1395027b113f29985); inspect `Sources/DICOMKit/DICOMParser.swift`, `DICOMParserOutcome.swift`, `DICOMFile.swift`, `DataSet.swift`, `DataSet+SpecificCharacterSet.swift`, `DataSet+ImplicitSignedVR.swift`, `Sources/DICOMCore/EncapsulatedPixelData.swift`, `Tag.swift`, `DataElement.swift`, `DICOMByteBuffer.swift`, and `Sources/DICOMDictionary` from this pinned revision.
- **Licence:** DICOMkit HEAD currently MIT. Check path-level ownership and licence history before adapting; other DICOM parsers are **development-time oracles only**. No `dicom-rs`, DCMTK, GDCM or equivalent parser in the shipping implementation.
- **Profile discipline:** The Swift implementation's `force` mode and broad compressed-syntax handling are informative, **not** permission to implement undocumented compatibility guessing in strict D1.

## 3. Initial admitted profile and explicit exclusions

**Mandatory D1-P01:** DICOM Part 10 files containing a 128-byte preamble and `DICM` marker, File Meta Information encoded **Explicit VR Little Endian**, and a following main Data Set encoded **Explicit VR Little Endian** (Transfer Syntax UID `1.2.840.10008.1.2.1`). Support a single SOP Instance. The source need not be fully materialised if it supports bounded random access.

**Conditional D1-P02:** Implicit VR Little Endian (`1.2.840.10008.1.2`) may be admitted *only* after the dictionary pin, ambiguity/unknown-value policy and dedicated test vectors are independently reviewed. It is a separately named coverage gate; D1-P01 may pass without it. Do not claim Implicit VR because a few tags appear to parse.

**Excluded first-slice input:** raw datasets without Part 10 wrapper, prefix-sniffing/force-reading legacy files, Big Endian/retired or deflated syntax, compressed Transfer Syntax decoding, DICOMDIR/file-set traversal, IOD validation, DICOM JSON/XML, networking, DICOMweb and full frame/pixel decoding. For an unsupported TS UID, report `UnsupportedTransferSyntax` without guessing the body syntax. A later read-only opaque preservation profile for compressed bulk data requires its own gate and cannot be conflated with D1-P01.

## 4. Included requirements — D1

| Requirement | Contract and expected behaviour |
|---|---|
| **D1-REQ-001** | Validate file shape and bounded preamble/prefix; reject missing `DICM` in strict mode and malformed/short Group 0002 File Meta Information. |
| **D1-REQ-002** | Parse File Meta Information in Explicit VR Little Endian **regardless** of main data set syntax, including TS UID, SOP Class/Instance UID, implemented version, group length and unknown future Group 0002 elements under a declared preservation/ignore policy. Reject `UN` in File Meta as required by PS3.10. |
| **D1-REQ-003** | Extract and validate Transfer Syntax UID `(0002,0010)` and enforce an explicit allowlist; no fallback to guessed VR or byte order. |
| **D1-REQ-004** | Decode element headers using D0 VR classification: 16-bit versus reserved-plus-32-bit lengths; validate reserved bytes and even lengths per selected strict policy. Reject truncation/overflow before indexing or materialisation. |
| **D1-REQ-005** | Traverse nested sequences and items, both explicit and undefined lengths where permitted; validate `FFFE,E000`, `FFFE,E00D`, `FFFE,E0DD`, item boundaries and nesting. Never search for delimiter *byte patterns* inside arbitrary values as a substitute for structured parsing. |
| **D1-REQ-006** | For admitted scalar VRs, expose exact typed numeric/string values with defined charset, multiplicity, padding and signedness. Retain the original encoded value or its source span for preservation. Unknown or ambiguous VRs must remain raw or yield a typed unsupported/ambiguous result. |
| **D1-REQ-007** | Provide a PS3.6 edition-pinned dictionary lookup and private creator resolution, including per-nested-item reservation scope, stable unknown handling and no invented private-VR semantics. |
| **D1-REQ-008** | Support Specific Character Set semantics for the admitted repertoire: default (`ISO_IR 6`) mandatory; optional `ISO_IR 100` and `ISO_IR 192` only after distinct vectors. Preserve original bytes; reject unsupported ISO-2022/escape/multibyte profiles, or expose opaque raw bytes without claiming decoded text. |
| **D1-REQ-009** | Expose bulk values (including native uncompressed Pixel Data where present) as bounded, source-retaining byte spans; permit metadata inspection without full frame/pixel decode. A view remains valid after parser object drop only through an explicit owner lease. |
| **D1-REQ-010** | Enforce limits on total input bytes, elements, items, depth, string/value materialisation, bulk descriptors, memory and work; reject incrementally and poll cooperative cancellation at bounded intervals. |
| **D1-REQ-011** | Define strict, deterministic errors with safe offsets/tag/VR contexts; no automatic patient-name/value echo in ordinary logs. |
| **D1-REQ-012** | Publish a usable Rust reader API and `dicomflux inspect` CLI with machine-readable versioned metadata output, optional explicit value disclosure and truthful unsupported-capability reporting. |
| **D1-REQ-013** | Provide a separate external consumer using published public APIs (not internal modules), and conformance/interoperability tests against independent normative expectations. |
| **D1-REQ-014** | Record exact preservation semantics: source-byte spans, typed scalar equivalence, unknown elements and nested values are not equal to a complete writer/byte-for-byte re-encoder. |

### Important semantic cases

**File Meta Information.** The 128-byte preamble may contain nonzero data and cannot be treated as 128 bytes of dataset tags. File Meta elements use Explicit VR Little Endian; the TS UID governs *only the following dataset*. Group 0002 has a distinct boundary; do not consume unrelated main-dataset tags as File Meta. Observe the mandatory File Meta types for full-file acceptance; malformed or missing required fields fail appropriately. Do not infer Transfer Syntax from a pixel header.

**Sequence parsing.** An undefined-length `SQ` requires structural traversal of Items and delimiters, not scanning for a byte sequence that could appear within an ordinary value. Explicit-length child boundaries must never extend beyond their containing sequence/item. An undefined-length Item's Item Delimitation must not be confused with Sequence Delimitation.

**Dictionary and VR.** For D1-P02 implicit VR, a tag may have a context-dependent VR (e.g. certain pixel-padding or LUT attributes depend on Pixel Representation). Preserve raw values or resolve within a well-defined nested dataset context; do not default to the first dictionary alternative or convert unsigned data to signed heuristically. Private creator reservations are item-local and do not bleed across sibling items.

**Text and numbers.** DICOM string padding, byte-level encoding and Value Multiplicity are not generic UTF-8 splitting. Preserve original values while exposing explicitly supported interpretations. Avoid floating-point conversions for integer attributes whose value/precision requires exact storage. Do not misinterpret packed pixel bits as signed scalar values when just inspecting Pixel Data.

## 5. Public API design target

Required public operation families (Rust spellings illustrative, to be frozen by the implementing agent after examining D0):

```rust
// Conceptual signatures, not an approved crate implementation.
Reader::open(source: OwnedOrBorrowedSource, options: ReadOptions)
    -> Result<Part10Object, DicomError>;
Part10Object::file_meta() -> FileMetaView;
Part10Object::dataset() -> DatasetView;
DatasetView::find(tag: Tag) -> Option<ElementView>;
DatasetView::elements() -> Iterator<ElementView>;
ElementView::raw_value() -> Result<RetainedByteSpan, DicomError>;
ElementView::typed_value() -> Result<TypedValue, DicomError>;
Reader::capabilities() -> ReaderCapabilities;
```

The object retains its source/owned backing; mutable consumer operations do not invalidate outstanding immutable views. Document whether a `DatasetView` can outlive `Reader`, which handles thread-safe immutable sharing, and what happens when the caller closes it while a span remains in use. No pointer survives a borrow without a retained owner. API failure must distinguish `Malformed`, `Truncated`, `UnsupportedTransferSyntax`, `UnsupportedCharacterSet`, `ResourceLimit`, `Cancelled`, `Io` and `Internal`.

`dicomflux inspect --input sample.dcm --output json --no-values` is an illustrative first process command. The CLI must declare actual syntax/capability status, element count and safe tag/VR/length information; optional `--show-values` is explicit and subject to privacy policy. Never dump patient data by default. The JSON schema has a version and tests; stdout carries result only, stderr carries bounded errors.

## 6. Implementation sequence and fixtures

1. **D1-A:** pin accepted D0 source and normative 2026d clauses; read current DICOMkit parser source and decide strict P01 semantics; prepare small, hand-checkable binary fixtures with provenance and independent byte layout expectations.
2. **D1-B:** implement Part 10 preamble, File Meta parser and TS allowlist; add positive/negative tests before dataset traversal.
3. **D1-C:** implement explicit VR dataset headers, bounds and typed scalar accessors. Validate order/duplicates in strict mode and preserve raw views.
4. **D1-D:** add sequence/item stacks (explicit/undefined boundaries), private creator scopes, charset and unknown-element disposition. Test value-contained delimiter patterns cannot terminate a sequence.
5. **D1-E:** implement deferred bulk-data spans, resource limits and cancellation. Prove no implicit full-pixel allocation for metadata-only inspection.
6. **D1-F:** add CLI and independent consumer, cross-platform builds and executed tests. Report P02 Implicit VR separately. Do not start DICOM writer or image codec adapter.

**Fixture design:** construct independent hand-audited Part 10 byte layouts and rights-cleared synthetic objects; store source digest, producer, expected metadata, SOP profile and permission. Distinguish *structural parser fixtures* (which may intentionally omit a complete IOD) from independently validated conforming SOP Instances. Include valid nonzero preamble, empty and nested sequences, defined/undefined item lengths, legitimate private creator at multiple levels, payload containing delimiter-like bytes, `SS`/`US` signedness, PN and selected character sets, and native Pixel Data large enough to expose accidental copies. The test corpus is internally versioned and its version changes whenever content or expectations change.

## 7. Negative tests and precise expected outcomes

| Case ID | Input | Expected disposition |
|---|---|---|
| **D1-T01** | 131-byte/short file or missing marker | Strict `MalformedPart10`/`TruncatedInput`; never fall back to guessed raw dataset. |
| **D1-T02** | Group 0002 encoded with wrong endian/VR or `UN` | `MalformedFileMeta`; not a successful main-dataset parse. |
| **D1-T03** | Unknown/unsupported TS UID | `UnsupportedTransferSyntax` before body interpretation. |
| **D1-T04** | VR/length contradiction, nonzero reserved bytes, odd value length where disallowed | Strict malformed error with tag/offset. |
| **D1-T05** | Item length exceeds parent, missing Item/Sequence Delimitation | Bounds/structural error; no scan or memory overrun. |
| **D1-T06** | Delimiter bytes embedded in ordinary opaque payload | Payload remains intact; no premature sequence termination. |
| **D1-T07** | Deep nesting or huge count/byte allocations | Resource-limit rejection before excessive work/allocation. |
| **D1-T08** | Private creator in one sequence item but absent in sibling | Sibling remains unresolved raw; no inherited reservation. |
| **D1-T09** | Ambiguous implicit VR when P02 included | Typed `AmbiguousVR` or preserved raw, never an invented signed value. |
| **D1-T10** | Unsupported character set | Explicit unsupported/raw disposition; no silent lossy replacement. |
| **D1-T11** | Cancellation during nested parse or bulk-index walk | `Cancelled`; owners and counters retained/released correctly. |
| **D1-T12** | Reader closes while retained bulk span exists | Span remains valid only with an explicit owner, or fails safely per API contract. |

Positive cases must cover the exact opposite boundaries, not merely `read(file).is_ok()`. Validate claimed scalar values against independently composed bytes. Differential comparison with DICOMkit and external tools is supplementary: related implementations can agree on a defect.

## 8. Platform, security and performance qualification

Execute D1 on the same primary Windows x64/ARM64, Linux x64/ARM64 and Android ARM64 profiles as D0; distinguish Android x86_64 emulator. Platform execution requires the actual reader and fixture corpus, not only the D0 capabilities CLI. GPU and UI tests are out of scope. A Java/JNI or full Android app binding is not required within D1; the C ABI must not be undermined by D1 additions, and a native C consumer is acceptable.

Choose finite resource budgets before testing; record source bytes, elements, total nesting, decoded materialised values, retained buffers, peak charged bytes and resident memory separately. Test latency for a small metadata-only file and a large bulk-value synthetic file in a pinned environment; report copy counts and materialised bytes. Do not invent universal throughput thresholds. Use fuzzing/property-based malformed element and sequence inputs with fixed seed/budget, and review all unsafe ranges/FFI edges. Audit logs must not include patient attributes or raw file paths by default.

Suggested executing-agent commands (not run by this document):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo run --release --bin dicomflux -- inspect --input fixtures/valid-p01.dcm --output json --no-values
cargo run --release --bin dicomflux -- inspect --input fixtures/invalid-sequence.dcm --output json --no-values
# Confirm exact expected process statuses and JSON fields in the test harness.
# Build and run the same reader against the declared primary target matrix.
```

CLI names, directories and flags are illustrative until D0/D1 public contract is fixed. Convert them to *actual* commands in the execution record; a successful command invocation alone is not a conformance result.

## 9. D1 acceptance gates and requirement trace

| Gate | Pass criteria and required evidence | Failure disposition |
|---|---|---|
| **D1-G01** | D0 ABI/source revision admitted, PS3.10/PS3.5/PS3.6 2026d inputs pinned, P01 extent and fixtures independently reviewed. | BLOCKED until authority/provenance. |
| **D1-G02** | P01 Part 10/File Meta/TS/explicit dataset read passes byte-accurate positives and D1-T01..04. | FAIL. |
| **D1-G03** | Nested item boundaries, raw/typed values, charset and private scopes pass D1-T05..10 and positive counterparts. | FAIL; no strict-parser claim. |
| **D1-G04** | Bulk spans, cancellation and bounded memory pass D1-T07/11/12 with lifetime/resource receipts. | FAIL. |
| **D1-G05** | CLI and independent consumer inspect a real, rights-cleared SOP instance; outputs/unsupported cases are truthful. | FAIL. |
| **D1-G06** | Target-specific real executions recorded for each required profile, including Android physical ARM64; emulator/build-only labelled separately. | NOT RUN/BLOCKED per missing target, no blanket pass. |
| **D1-G07** | Coverage register distinguishes P01, conditional P02, metadata inspection, pixel preservation and unsupported pixel decoding; documentation/SHA-256 complete. | INCOMPLETE. |

P02 Implicit VR has **D1-G08** as an **optional independent gate**: pin dictionary, demonstrate standard private/pixel-context ambiguities, run its own positive/negative fixtures and explicitly add coverage. P02 cannot be shown as supported solely through P01 evidence.

A host-only P01 implementation may be submitted for scoped continuation after G02–G05, but D1 is not fully multi-platform qualified while G06 remains open. Acceptances must state their exact scope and named reviewer.

## 10. Decisions, blockers and change control

- **D1-DEC-01:** review D0 and freeze P01 public `Reader`/`Part10Object`/bulk-span interfaces; no independent redefinition.
- **D1-DEC-02:** specify strict duplicate/order/unknown-Group-0002 handling and which semantic elements are decoded versus preserved raw. Strict mode must not silently select a legacy compatibility pathway.
- **D1-DEC-03:** choose the first admitted text repertoires and dictionary generation process; obtain rights to reproduce dictionary entries/test data.
- **D1-DEC-04:** decide whether conditional P02 is admitted now or remains later. It does not block P01.
- **D1-DEC-05:** confirm resource budgets and platform executors; supply named source and independent reviewers.

Material changes to D0 ownership, ABI, VR classification or errors must be reviewed in both workstreams before adoption. D1 cannot use this order to authorise full writer/network implementation.

## 11. Coding-agent completion report

Supply `D1_COMPLETION.md`, `D1_CAPABILITY_MATRIX.md`, `D1_SOURCE_AND_FIXTURE_REGISTER.md`, `D1_TEST_RESULTS.md`, `D1_PLATFORM_MATRIX.md` and `SHA256SUMS.txt` (or equivalent consolidated records). Each requirement and D1-T case maps to actual code path, fixture hash, command, result and reviewer. Separate **implemented**, **built**, **executed**, **verified**, **blocked**, **not run**, and **unqualified**. Report memory accounting and security/privacy results. Attach anonymised/synthetic evidence only; never place patient data in logs or public repositories.

## 12. Reviewer acceptance and definition of done

The independent reviewer checks the P01 fixture/oracle ledger, parser-source revision, resource results and external consumer. The owner records **ADMIT / ADMIT WITH EXPLICIT GATE HOLD / RETURN FOR CORRECTION**. A D1 acceptance does not imply writer completeness, image decompression or product clinical conformance. After approval, issue a distinct next work order for a Part 10 writer and/or DICOMflux-to-Flux codec adapter.

## 13. Copy-ready instruction to the D1 implementing agent

```text
You are the separately commissioned DICOMflux D1 coding agent. Read the actual
accepted D0 source/contract, DICOMflux Manifesto v1.3.1, the DICOMflux Rust
Implementation Instructions v0.1.0 and this D1 work order/source register.
Reverify the 2026d normative source and the pinned DICOMkit reference.

Implement only strict Part 10 D1-P01: preamble/prefix, mandatory File Meta
Information encoded Explicit VR Little Endian, explicit-Little main dataset,
correct element and nested sequence/item boundaries, selected VR/dictionary/
charset interpretation, private creator scope, bounded retained bulk spans,
cooperative cancellation and a real inspect CLI. Keep unknown values raw where
needed, reject unsupported Transfer Syntaxes, and never silently guess.

Do not implement the writer, complete IOD validator, compressed-pixel decoder,
network stack, DICOMflux-3D or any workstation/mobile UI. Add the optional
Implicit VR profile only if separately admitted and tested.

Use standards-derived fixtures and negative tests D1-T01..12. Execute D1-G01..07
and report accurate per-target status: built is not executed, emulator is not
physical ARM64, and DICOMkit agreement alone is not independent correctness.
Supply exact source/build/fixture identities, safety and memory results,
SHA-256 checksums, reviewer handoff and next bounded work order. Stop at D1.
```

## 14. Source pointers and status

- [DICOMflux Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view); [DICOMkit current source snapshot](https://github.com/raster-labs/DICOMKit/tree/e127b5e9c69629eea8d45cb1395027b113f29985).
- [DICOM PS3.10 current chapter 7](https://dicom.nema.org/medical/dicom/current/output/chtml/part10/chapter_7.html); [PS3.5 chapter 7](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/chapter_7.html); [PS3.5 private attributes](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/sect_7.8.html); [official 2026d publication index](https://dicom.nema.org/medical/DICOM/current/).
- Companion `DICOMflux_Work_Order_Source_Register_v0.1.0.md` documents inspected source heads, release/rights differences and the unexecuted source-to-Rust qualification boundary.

**Document status:** Proposed. No D1 Rust source was modified, compiled, tested or clinically qualified by this documentation commission.
