# DICOMflux — D0/D1 Source, Authority and Rights Register

**Version:** 0.1.0 · **Checked:** 10 November 2026 · **Status:** source-inspection evidence, NOT Rust runtime qualification.  
**Scope:** D0 and D1; other programme documents remain in their respective folders.

## 1. Governing file identities

| Record | Exact source | Authority and use |
|---|---|---|
| Development Manifesto v1.3.1 | [Google Drive `1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl`](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view); SHA-256 `b15ed5cea296ab9dc1c60e879a178eb18c6758e123cac5f97d3bacb5b868234c` | Governs independent Rust work, MIT, bounded memory, ownership, targets and provenance. |
| DICOMflux Rust Implementation Instructions v0.1.0 | [Google Drive `1KyiALgcCrUzwiOM4RLFKZSq2f3JPxwWw`](https://drive.google.com/file/d/1KyiALgcCrUzwiOM4RLFKZSq2f3JPxwWw/view); published SHA-256 `7620b890dd4e0caedadaed5555b76a5444dd0a52fe98cb58c166d44fb28888dd` | Predecessor handoff, not executable proof. |
| DICOMflux Human Guide v0.1.0 | [Google Drive `1ALIAFxvCyerhUGA_uH_2ooVe5Corvd4X`](https://drive.google.com/file/d/1ALIAFxvCyerhUGA_uH_2ooVe5Corvd4X/view); published SHA-256 `d49ab8ba92dec53fc0dde34ff816f43ad803e59398a0b0247d4e16e8e74afab9` | Human explanation, subordinate to actual normative text and work orders. |
| Prior SHA manifest | [Google Drive `10fxte3WPmxyUJv9rs5ZBiHsJlxlxNs--`](https://drive.google.com/file/d/10fxte3WPmxyUJv9rs5ZBiHsJlxlxNs--/view) | Checksum record read 10 November 2026. The owner's pasted URL used a typographic dash; this is the verified Drive ID. |

## 2. Current DICOMkit source selection

| Field | Observation |
|---|---|
| Current code repository | [raster-labs/DICOMKit](https://github.com/raster-labs/DICOMKit) (product name spelled **DICOMkit** in this programme) |
| Selected `main` snapshot | `e127b5e9c69629eea8d45cb1395027b113f29985`, observed 10 November 2026 10:34 UTC |
| Previously inspected snapshot | `9b402fef534c4d6778c36bd7498539fa80b1ee7a`, 7 November 2026 |
| Change since previous | Four commits ahead: mostly copyright/licence, release packaging, Studio work and minor tests; D0/D1 must still inspect exact source at selected commit. |
| Published release identity | `v3.0.0`, 7 November 2026, observed via GitHub Releases; use release tag for stable-source comparison and selected `main` for up-to-date API inspection. |
| Licence at selected HEAD | `LICENSE` begins `MIT License`, copyright 2026 Raster Images Private Limited. Verify any project-owned upstream adaptation and third-party material path-by-path. |
| Toolchain reference | `Package.swift` declares Swift tools 6.4 and Apple OS 26 minima; **not** a Rust platform dependency. |

### Source files inspected

- [`Sources/DICOMKit/DICOMParser.swift`](https://github.com/raster-labs/DICOMKit/blob/e127b5e9c69629eea8d45cb1395027b113f29985/Sources/DICOMKit/DICOMParser.swift): structure, strict bounded parsing, element/sequence limits, VR rules. Existing source includes legacy/compatibility behaviours that are **not** part of strict D1-P01.
- [`Sources/DICOMKit/DICOMFile.swift`](https://github.com/raster-labs/DICOMKit/blob/e127b5e9c69629eea8d45cb1395027b113f29985/Sources/DICOMKit/DICOMFile.swift): Part 10 wrapper, force/legacy read API; strict D1 must not silently inherit force mode.
- [`Sources/DICOMKit/DataSet.swift`](https://github.com/raster-labs/DICOMKit/blob/e127b5e9c69629eea8d45cb1395027b113f29985/Sources/DICOMKit/DataSet.swift): typed element view and storage concepts; note its duplicate-tag last-wins construction is not evidence that a strict parser should silently accept duplicates.
- [`Sources/DICOMCore/DICOMByteBuffer.swift`](https://github.com/raster-labs/DICOMKit/blob/e127b5e9c69629eea8d45cb1395027b113f29985/Sources/DICOMCore/DICOMByteBuffer.swift), [`DICOMAllocationBudget.swift`](https://github.com/raster-labs/DICOMKit/blob/e127b5e9c69629eea8d45cb1395027b113f29985/Sources/DICOMCore/DICOMAllocationBudget.swift) and [bounded allocation contract](https://github.com/raster-labs/DICOMKit/blob/e127b5e9c69629eea8d45cb1395027b113f29985/Documentation/BoundedInput/AllocationContract.md): meaningful owner/lease/resource vocabulary, not a requirement to reproduce Darwin page mappings in Rust.
- `Sources/DICOMCore/Tag.swift`, `DataElement.swift`, `DICOMOutcomes.swift`, `DICOMManagedFailure.swift`, `Sources/DICOMDictionary/**`, `Sources/DICOMKit/DataSet+SpecificCharacterSet.swift` and `DataSet+ImplicitSignedVR.swift`: **assigned direct source inspection**; no claim that their every public declaration was audited for this documentation package.

## 3. Normative edition / reference mismatch

- [Official DICOM publication index](https://dicom.nema.org/medical/DICOM/current/) on 10 November 2026 listed consolidated **2026d**, dated 21 September 2026. Use applicable approved Final Text changes after that snapshot as required.
- Moving chapter HTML pages checked during preparation—[PS3.10 chapter 7](https://dicom.nema.org/medical/dicom/current/output/chtml/part10/chapter_7.html), [PS3.5 chapter 7](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/chapter_7.html), [PS3.5 section 7.8](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/sect_7.8.html)—still displayed **2026c** in page titles. Do not confuse a moving URL's cache with the official release index. The implementing agent must obtain the exact 2026d archive/source, hash it and reconcile clause changes before freezing parsers/dictionaries.
- The normative work-order scope is PS3.10 7.1; PS3.5 6.2, 7.1, 7.5, 7.8, 10 and applicable Annex A; PS3.6 dictionary and Transfer Syntax UIDs. Any right to redistribute dictionary snapshots or samples requires its own source/rights record.
- DICOMkit source contains older `NEMA-verified 2026a` comments. Those are historical implementation attestations, **not** a substitute for current normative review.

## 4. Cross-programme codec interface references (not D0/D1 dependencies)

| Repo | Selected `main` SHA | Release status / licence on HEAD | D0/D1 relevance |
|---|---|---|---|
| [SwiftJ2K](https://github.com/raster-labs/SwiftJ2K) | `8b94527fb3057ea58cb49c754219f97961df3557` | Published `v12.1.0`, 12.2.0 untagged development; MIT HEAD | Later DICOMflux image adapter only. |
| [SwiftJLS](https://github.com/raster-labs/SwiftJLS) | `a014bac5611ca654079c655e8fd88eb25caeab15` | `v2.0.0`; MIT HEAD | Later image adapter. |
| [SwiftJXL](https://github.com/raster-labs/SwiftJXL) | `e3adb2d0f5d16e38d36fd13e7f4a2de385e66381` | `v2.0.0`; Apache-2.0 HEAD | Distinct licensing risk for any adaptation. |
| [SwiftJLI](https://github.com/raster-labs/SwiftJLI) | `97a50eee8af72e2a0b32f8a2444b8bc7b96a4c2c` | `v2.0.2`; MIT HEAD | Later image adapter. |
| [SwiftRLE](https://github.com/raster-labs/SwiftRLE) | `f81855537674ac5c3d64d6a984386bc54988b176` | `v1.0.1`; MIT HEAD | Later DICOM RLE frame adapter; F1-R separate. |

These codec heads were compared with the previous snapshots. Changes may affect interface treatment, but none enters the D0/D1 dependency graph. For codec details consult the distinct Flux Imaging Suite source register.

## 5. Source/rights and audit limits

**Inspected:** repository latest commit identity, published latest release identity, package/licence extracts, selected parser/dataset/storage files and the official standard index. **Not performed:** a full line-by-line API audit, verification of all upstream copyright assignments, a Rust build, exact 2026d archive download/digest, native-device execution, fuzzing, performance measurement or clinical qualification. Thus every applicable future acceptance test starts at **NOT RUN** in the new work orders.

At execution kickoff, the assigned agent creates a new immutable source snapshot register with actual chosen commit+tag+file/dictionary hashes; records per-file `owned/adapted/external/read-only oracle`, licence and notices; and obtains developer/legal sign-off before reusing third-party material. Do not assert a formal clean-room process unless its controls were actually followed.
