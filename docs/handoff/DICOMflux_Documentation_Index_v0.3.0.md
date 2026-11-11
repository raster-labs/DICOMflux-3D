# DICOMflux — Documentation Index and Filing Policy

**Index version:** 0.3.0 · **Issued:** 10 November 2026 · **Owner:** Raster Images Private Limited  
**Top-level folder:** https://drive.google.com/drive/folders/1QKA7baj8xLz8nRH-rEMTNsRNxK68vD2W  
**Publication state:** Four **proposed** Rust coding-agent work orders, two human guides, source/traceability records. Documentation-only; no implementation or clinical-use authorisation.

## Folder responsibilities and boundaries

| Folder | Google Drive location | Permitted documentation |
|---|---|---|
| **DICOMflux** | [Library folder](https://drive.google.com/drive/folders/1w5e4Td51soP-Agy1WiMzvs44ClSmKR4y) | Rust DICOM core, shared manifesto, C ABI, dataset/Part 10, later DICOM networking/writer/codec adapters and independent library qualification. |
| **DICOMflux-3D** | [Library folder](https://drive.google.com/drive/folders/14rM4Vkl3WjmAyy3n1_cSKXDwB6_mZRQ4) | Separately authorised volume/3D imaging and GPU library specifications and verification. **Unaffected by this D0–F1 commission.** |
| **Flux Imaging Suite** | [Library folder](https://drive.google.com/drive/folders/1RQ7qcjhZntu3_9QMjv8_MKdA2PJJOFlP) | DICOM-independent Rust codecs, image storage/memory contracts, resampling, cross-codec pipelines and benchmarks. |

**Never file application documentation here.** Native Windows/Linux workstation, responsive web viewer/workstation, Oviyam | Mobile, DICOM Camera and DICOM Capacitor application planning and implementation belong to other folders to be specified by the owner. Library-facing API/verification contracts required by those applications can be filed here when the owning library is in scope.

## Current governing documents and predecessor guides

| Document | Version | Location |
|---|---|---|
| DICOMflux and Flux Imaging Suite Development Manifesto | **1.3.1** | [DICOMflux Markdown](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view) |
| DICOMflux Rust Coding-Agent Implementation Instructions | **0.1.0** | [DICOMflux Markdown](https://drive.google.com/file/d/1KyiALgcCrUzwiOM4RLFKZSq2f3JPxwWw/view) |
| DICOMflux Rust Implementation Human Guide | **0.1.0** | [DICOMflux PDF](https://drive.google.com/file/d/1ALIAFxvCyerhUGA_uH_2ooVe5Corvd4X/view) |
| Flux Imaging Suite Rust Coding-Agent Implementation Instructions | **0.1.0** | [Suite Markdown](https://drive.google.com/file/d/1bFhqX1t4Qh0tezeyXXt4ofbWqTTShRdx/view) |
| Flux Imaging Suite Rust Implementation Human Guide | **0.1.0** | [Suite PDF](https://drive.google.com/file/d/1Dfv3TjhReVijbaqIqyratJQLTvSOn0Mg/view) |

## Initial coding-agent work orders — published 10 November 2026

**DICOMflux**

| Published document | Version | Direct link |
|---|---|---|
| **D0 — Foundation and core contracts** | 0.1.0 | [Markdown](https://drive.google.com/file/d/1pvgop0cMQl3w6MwH5lfe7qkno2EG22fQ/view?usp=drivesdk) |
| **D1 — Bounded Part 10 reader** | 0.1.0 | [Markdown](https://drive.google.com/file/d/1FuvyO1o8SZkWMogJZyolT06VW66EIvpk/view?usp=drivesdk) |
| D0–D1 Human Guide | 0.1.0 | [PDF](https://drive.google.com/file/d/1vezBOLK7Lyf0im0ih5XGrM1BBA0Vn8QG/view?usp=drivesdk) |
| D0/D1 Dependencies and Acceptance Matrix | 0.1.0 | [Markdown](https://drive.google.com/file/d/10QV1skFbngyL5W2MnC-_IAkET7Ti0U74/view?usp=drivesdk) |
| D0/D1 Source, Authority and Rights Register | 0.1.0 | [Markdown](https://drive.google.com/file/d/1CBo0X2VrsY9Z3gBWBfNHyKAP4yoFF2gV/view?usp=drivesdk) |
| D0/D1 integrity manifest | 0.1.0 | [SHA256SUMS](https://drive.google.com/file/d/1PSrqFOi2oFGXhEp9SwveYxkVRneb8IBl/view?usp=drivesdk) |

**Flux Imaging Suite**

| Published document | Version | Direct link |
|---|---|---|
| **F0 — Common image API and memory contracts** | 0.1.0 | [Markdown](https://drive.google.com/file/d/1CcLi-GfuGDJPb26qUkqqIlBNqNRFms8q/view?usp=drivesdk) |
| **F1 — Initial FluxRLE and FluxResample algorithms** | 0.1.0 | [Markdown](https://drive.google.com/file/d/1P7iYB6GOK5QBdGWeBF1F0EU4OMQkQvOC/view?usp=drivesdk) |
| F0–F1 Human Guide | 0.1.0 | [PDF](https://drive.google.com/file/d/1MOX6eijALPNZWSGeZvqsr6kMI4CK99Ej/view?usp=drivesdk) |
| F0/F1 Dependencies and Acceptance Matrix | 0.1.0 | [Markdown](https://drive.google.com/file/d/1VQSfsXG0e9gaALSFMHFXy08EbAuE3I1K/view?usp=drivesdk) |
| F0/F1 Source, Contract and Rights Register | 0.1.0 | [Markdown](https://drive.google.com/file/d/1kDo9aRzqa4RAG_6G8um46apbfEVIkHtr/view?usp=drivesdk) |
| F0/F1 integrity manifest | 0.1.0 | [SHA256SUMS](https://drive.google.com/file/d/1mOVn080mrDc4g-rpcpyKF6w9_-mHXK2K/view?usp=drivesdk) |

## Recommended work order sequence and status

```text
D0: Rust DICOM foundation  ------> D1: Strict Part 10 read (P01)
  (parallel, independent)
F0: Generic image foundation ---> F1-R: FluxRLE (independent gate)
                                 F1-S: FluxResample (F1-S-G00 open)

Later *separately commissioned* DICOMflux image adapter may consume FluxRLE.
There are no DICOMflux dependencies inside the generic imaging suite.
```

All four work orders are currently **PROPOSED / NOT EXECUTED**. D1 depends on admitted D0 contracts; F1-R and F1-S depend on admitted F0 contracts. The F1-S numerical/source gate may remain open while F1-R progresses. Real target/device tests and independent verification remain future implementing-agent obligations.

## Current reference refresh and open source-identity issue

The work-order source registers reflect source inspection on **10 November 2026**: DICOMkit `main` `e127b5e9...` (published v3.0.0); SwiftJ2K `8b94527f...` (12.2.0 development line, published v12.1.0); SwiftJLS `a014bac5...`; SwiftJXL `e3adb2d0...`; SwiftJLI `97a50eee...`; SwiftRLE `f8185553...`. Current inspected source licensing is MIT except SwiftJXL HEAD Apache-2.0; historic releases and third-party resources retain their own terms. The Swift shared contract documents include version drift that F0 must reconcile.

A currently accessible [SwiftResample Drive folder](https://drive.google.com/drive/folders/1G1rubxhzRlD72giX7it_xPDVcX_U-5n8) contained no files; an exact current GitHub source/public API was not verified. F1-S must use an expressly accepted numerical baseline, not an invented Swift parity claim. The official DICOM index listed consolidated 2026d; the executing agent must hash the actual normative snapshot and approved Final Text inputs before freezing edition-sensitive behaviour.

## Integrity records and file policy

- The D0/D1 and F0/F1 `SHA256SUMS_WorkOrders_v0.1.0.txt` files verify each respective Markdown source register, matrix, work order and PDF by exact file bytes. A checksum does not establish signing or authorship.
- Previous [DICOMflux v0.1.0 manifest](https://drive.google.com/file/d/10fxte3WPmxyUJv9rs5ZBiHsJlxlxNs--/view), [Flux Imaging Suite v0.1.0 manifest](https://drive.google.com/file/d/10qp17EGu7gSGphhxWXeIgnKXviOAz-u4/view), and [Manifesto SHA companion](https://drive.google.com/file/d/13zoE74dUZ6El2TmgrjdowpwzVZabVY-s/view) remain unchanged.
- Author normative instructions, API/source inventories, agent handoffs and test requirements in **Markdown**. Human-readable **PDF** guides and optional illustrations belong alongside their owning library documents. Preserve every versioned issue; never silently overwrite previous releases.
- Record verified source SHA and licence, tested versus untested features, exact toolchains and real executions. Do not report planned future coding-agent tests as executed and do not authorise clinical use by publishing documentation.
- Keep names **DICOMkit** and **Swift Imaging Suite (SIS)** in newly authored content; preserve historical GitHub filenames verbatim where necessary.

## Publication and review status

The indexed work orders specify implementation decisions and acceptance gates. Their publication **does not launch coding agents, modify GitHub repositories, release Rust software, adopt source licences on behalf of counsel or authorise clinical use**. Repository authority and named reviewers must be granted through a separate owner-issued execution assignment. Future versions are published into their respective existing library subfolders and referenced from a new root index revision.

**Index change v0.3.0:** appended the four D0/D1/F0/F1 work orders, two human guides, separate matrices/source registers, checksum manifests, refreshed source/rights observations and explicit independent acceptance rules; preserved the prior v0.2.0 filing policy and links.
