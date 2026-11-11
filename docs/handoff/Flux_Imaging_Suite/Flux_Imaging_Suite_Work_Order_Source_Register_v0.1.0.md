# Flux Imaging Suite — F0/F1 Source, Contract and Rights Register

**Version:** 0.1.0 · **Observed:** 10 November 2026 · **Status:** source inspection, not a Rust verification result.  
**Work orders:** F0 common contracts; F1-R PackBits/DICOM RLE and F1-S initial resampling.

## 1. Governing records

| Record | Exact identifier and observation |
|---|---|
| Development Manifesto v1.3.1 | [Drive file `1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl`](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view); SHA-256 `b15ed5cea296ab9dc1c60e879a178eb18c6758e123cac5f97d3bacb5b868234c`. |
| Flux Imaging Suite Rust Implementation Instructions v0.1.0 | [Drive file `1bFhqX1t4Qh0tezeyXXt4ofbWqTTShRdx`](https://drive.google.com/file/d/1bFhqX1t4Qh0tezeyXXt4ofbWqTTShRdx/view); published SHA-256 `c781f6692a1e58551c0fa56b47bd1c90b19f0be8cd4ea7d6f50b8a61b660fd2e`. |
| Flux Imaging Suite Human Guide v0.1.0 | [Drive file `1Dfv3TjhReVijbaqIqyratJQLTvSOn0Mg`](https://drive.google.com/file/d/1Dfv3TjhReVijbaqIqyratJQLTvSOn0Mg/view); published SHA-256 `fbf31bc0a5d1ef5369df4782888c404dcdee020d054acf42eb35835b1bac6387`. |
| Previous integrity manifest | [Drive `10qp17EGu7gSGphhxWXeIgnKXviOAz-u4`](https://drive.google.com/file/d/10qp17EGu7gSGphhxWXeIgnKXviOAz-u4/view). |

## 2. Current Swift successor snapshots

| Reference | Selected `main` SHA; last observed commit | Latest published release | HEAD licence / notable difference |
|---|---|---|---|
| [SwiftJ2K](https://github.com/raster-labs/SwiftJ2K) | `8b94527fb3057ea58cb49c754219f97961df3557`; 10 Nov 11:35 UTC | `v12.1.0` | `VERSION=12.2.0`, **unreleased** development, MIT HEAD; clean-room HTJ2K work, JP3D/JPIP qualification and some external interop holds. Previous `3059a3c8...`. |
| [SwiftJLS](https://github.com/raster-labs/SwiftJLS) | `a014bac5611ca654079c655e8fd88eb25caeab15`; 10 Nov 12:13 UTC | `v2.0.0` | MIT HEAD after licence work; previous `48b65b29...`. |
| [SwiftJXL](https://github.com/raster-labs/SwiftJXL) | `e3adb2d0f5d16e38d36fd13e7f4a2de385e66381`; 10 Nov 13:06 UTC | `v2.0.0` | **Apache-2.0** HEAD; later optimisation commit to Modular, VarDCT, JPEG reconstruction/CLI. Previous `2c7638ca...`. |
| [SwiftJLI](https://github.com/raster-labs/SwiftJLI) | `97a50eee8af72e2a0b32f8a2444b8bc7b96a4c2c`; 10 Nov 12:22 UTC | `v2.0.2` | MIT HEAD after licence work; previous `b6b9f99c...`. |
| [SwiftRLE](https://github.com/raster-labs/SwiftRLE) | `f81855537674ac5c3d64d6a984386bc54988b176`; 10 Nov 11:41 UTC | `v1.0.1` | MIT HEAD after licence work; previous `4eaaf913...`. |
| [DICOMkit](https://github.com/raster-labs/DICOMKit) | `e127b5e9c69629eea8d45cb1395027b113f29985`; 10 Nov 10:34 UTC | `v3.0.0` | MIT HEAD, later **consumer**, not a dependency of F0/F1. |

**Source rule:** These are selected inspection references, not irreversible freezes. At agent kickoff compare the current HEAD with these SHAs and record a reasoned admission decision for any new material. A head source tree with a `VERSION` value is **not** the same as a published tagged release or executed platform qualification. Code and docs on older tags preserve their then-applicable licence texts.

## 3. Verified contract files and drift

- [`SwiftRLE/Documentation/COMMON_API.md`](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Documentation/COMMON_API.md) describes `Encoder`, `Decoder`, `ImageDescriptor`, `PlaneDescriptor`, `Image`, `ImageDestination`, source/destination storage, options/capabilities, `OperationReport` and `CodecError`. Its **heading remains contract `0.7.0`**, which is not proof of current universal 0.7.0 policy.
- [`SwiftRLE/Documentation/MEMORY_CONTRACT.md`](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Documentation/MEMORY_CONTRACT.md) also labels contract `0.7.0` and establishes low-bit alignment, signed extension, explicit strides/plane origin, shared backing ownership and source/destination asymmetry. The inspected `MEMORY_CONTRACT.md` blob digest was identical in at least SwiftJ2K, SwiftJLS and SwiftRLE at the reviewed heads.
- [`SwiftJ2K/Documentation/SUITE_POLICY.md`](https://github.com/raster-labs/SwiftJ2K/blob/8b94527fb3057ea58cb49c754219f97961df3557/Documentation/SUITE_POLICY.md) and [`SwiftJLS/.../SUITE_POLICY.md`](https://github.com/raster-labs/SwiftJLS/blob/a014bac5611ca654079c655e8fd88eb25caeab15/Documentation/SUITE_POLICY.md) report **0.11.0** on 10 November 2026. At the same inspection SwiftRLE/SwiftJXL report **0.10.0**, while SwiftJLI's heading still reports **0.10.0**. Require exact `Documentation/COMMON_CONTRACT_SHA256.txt` and path-level reconciliation in F0.
- [`SwiftRLE/Sources/SwiftRLE/DICOMRLEFrame.swift`](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Sources/SwiftRLE/DICOMRLEFrame.swift) implements header/segment validation, raw allocated-cell vs logical-sample paths; [`RLEDescriptor.swift`](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Sources/SwiftRLE/RLEDescriptor.swift) shows external frame geometry and metadata; [`R2_FRAME_PROFILE.md`](https://github.com/raster-labs/SwiftRLE/blob/f81855537674ac5c3d64d6a984386bc54988b176/Documentation/R2_FRAME_PROFILE.md) records strict byte lanes, trailing padding, 64-byte header and independent tests. These are first-priority F1-R references, **not** a substitute for DICOM PS3.5.
- Other public modules to inspect on the pinned revisions: SwiftJ2K `Sources/SwiftJ2K/CodecAPI.swift`, SwiftJLS `Sources/SwiftJLS/CodecAPI.swift`, SwiftJXL `Sources/SwiftJXL/CodecAPI.swift`, SwiftJLI `Sources/SwiftJLI/CodecAPI.swift`, plus image descriptors, storage and CLI tests. No claim of full symbol-by-symbol equivalence is made in this documentation pass.

## 4. SwiftResample recovery outcome

- [Drive SwiftResample folder](https://drive.google.com/drive/folders/1G1rubxhzRlD72giX7it_xPDVcX_U-5n8) was readable but **empty** when inspected 10 November 2026.
- GitHub `raster-labs/SwiftResample` lookup returned not-found; repository search `SwiftResample` returned no matching repository. This establishes an **unverified current repository/specification identity**, not proof that no implementation has ever existed.
- Authorised programme scope is seven kernels: nearest, bilinear, bicubic, Mitchell-Netravali, Catmull-Rom, Lanczos and area/box. Coordinate centres, edge extension, ties, rounding, bicubic definition, anti-aliasing, colour and precision details need recovered source/accepted contract. F1-S-G00 is open until that issue is resolved. No obstacle to F1-R follows from it.

## 5. Normative and legal sources

- [DICOM official current publication index](https://dicom.nema.org/medical/DICOM/current/) listed **2026d** as consolidated on 10 November 2026; pin actual 2026d PS3.5 §8.2.2 and Annex G source byte/digest. Moving HTML can show older editions; do not claim a specific clause was verified from 2026d without the archived text.
- [DICOM PS3.5 RLE Annex G](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/chapter_G.html) and §8.2.2 are the format authority, alongside actual codec standard for other later modules.
- [OSI MIT licence](https://opensource.org/license/mit) and actual `LICENSE`, `NOTICE`, SPDX identifiers, third-party fixture/asset records for each selected revision. Rust will be project-owned MIT, **not** an automatic relicensing of imported Apache-2.0 material. External contributions require an executed assignment with legal review where applicable.

## 6. Inspection/evidence limits and next-source tasks

**Performed:** live GitHub HEAD/commit/release/licence checks, source/repository comparison against prior handoff SHAs, selected public API, memory/PackBits/DICOM-RLE source inspection, programme Drive read, SwiftResample repository/folder lookup, official DICOM index verification. **Not performed:** full Swift API symbol audit, actual Swift build, Rust code creation, Rust test run, independent codec oracle run, platform hardware tests, full current DICOM archive rehash, source-ownership legal conclusion or clinical validation.

At implementation kickoff, pin exactly selected and modified source commits/tags; capture local file hashes and `COMMON_CONTRACT_SHA256` resolutions; create a per-file provenance/rights ledger; examine package dependency graphs; reconcile F0 API inventory against *actual* Swift public source; and establish independent test expectations and a versioned rights-cleared corpus. No historic test count constitutes future Rust qualification.
