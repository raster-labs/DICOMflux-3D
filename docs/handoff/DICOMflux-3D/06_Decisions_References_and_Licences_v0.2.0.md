# 06 — Approved Decisions, Source Register and Licence Plan

**Version/date:** 0.2.0 / 10 November 2026  
**Owner status:** **DF3D-DEC01–DF3D-DEC09 APPROVED on 10 November 2026.**  
**Interpretation:** These approve architecture and the preparation of a first-slice handoff; not an unverified project repository, code, test pass, qualified medical device or release.

## 1. Decision ledger — accepted

| ID | Approved architectural decision | Consequence for implementation |
|---|---|---|
| DF3D-DEC01 | Name **DICOMflux-3D**, intentionally including GPU-accelerated 2D | Public project scope includes 2D windowing and 3D, no split DICOMcanvas-like Rust foundation required |
| DF3D-DEC02 | Discrete MIT Rust component, one-way consumer of DICOMflux; no duplicate parser/codecs | DICOMflux-3D accepts checked decoded buffers + metadata, not DICOM files |
| DF3D-DEC03 | wgpu/WGSL; optional egui/winit harness; exclude VTK | Rendering usable without GUI toolkit or event loop |
| DF3D-DEC04 | Correct DICOM presentation and checked patient-space geometry before advanced 3D | Immutable samples, modality/VOI/presentation stages; LPS f64 geometry |
| DF3D-DEC05 | Bounded 3D texture residency initially; designed for later bricked storage; CPU reference operations | Capabilities, memory accounting, deterministic oracles; bricks deferred |
| DF3D-DEC06 | Threshold segmentation in planned initial *capability family*; surface extraction/advanced segmentation later | Basic threshold phase follows first 2D/MPR/DVR slices; do not assume FS-01 has segmentation |
| DF3D-DEC07 | Windows/Linux x64 and ARM64 priority; independent later Android/web/macOS qualification | No implicit platform or GPU-driver support claims |
| DF3D-DEC08 | Learn from SolidCraft and Voxelia without copying general-purpose hierarchies | Distinct internal modules, one focused public crate, no CAD/B-rep transfer |
| DF3D-DEC09 | Capability, resource, provenance, verification, licence and SHA-256 control | Test evidence, release gate and integrity files required |

### Post-review owner correction to repository authority

The owner has expressly designated **https://github.com/raster-labs** as the **new GitHub home**, with an entirely Rust-focused DICOMflux programme, and explicitly instructed **ignore the earlier C++ DICOMflux repository**. This instruction supersedes the prior discussion's accidental use of the old codebase. No transfer, migration or wrapper of historical C++ is authorised. The public `raster-labs/DICOMflux` or `raster-labs/DICOMflux-3D` Rust repo was **not verified** during package preparation: record repo/branch/commit as `TBD—DEVELOPER TO IDENTIFY`, not a claimed URL.

## 2. Governing evidence and source register

| Ref | Source | Version / identity and use |
|---|---|---|
| GOV-01 | [DICOMflux and Flux Imaging Suite Development Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view) | Issued 9 November 2026; owner directions, licence, independent implementation, §4.1 3D boundary, target OSes and evidence conventions |
| GOV-02 | [DICOMflux Documentation Index v0.2.0](https://drive.google.com/file/d/1Yh3FEgYYynEEqGJjCxFgETW10VKS2e9E/view) | Issued 10 November 2026; canonical three-library-folder filing policy and Markdown+guide+SHA requirement |
| REF-01 | [Voxelia Master Technical Architecture v0.1.1](https://drive.google.com/file/d/1dSEdx6JzFeGGfGu9a6nmXaIENf_CcxKX/view) | Conceptual reference: patient-space geometry, immutable source, value stages, scene state, CPU checks, validation. Duplicate copies exist in Drive; chosen accessible copy pinned by Drive file ID; no runtime dependency. |
| REF-02 | [SolidCraft](https://github.com/tangency-labs/solidcraft) | Observed HEAD `017c47a402769568ae702c1dba599390700fb2eb` (10 Nov 2026). Reviewed `docs/architecture.md`, `docs/layers.md`, `docs/render-backends.md`, `crates/viewport/src/lib.rs`, `crates/viewport-gpu/src/lib.rs`, `crates/viewport-gpu/src/select.rs`; design patterns only. |
| REF-03 | [DICOMkit Apple source](https://github.com/raster-labs/DICOMKit) | Historic GitHub repository spelling `DICOMKit`; product name **DICOMkit**. Apple reference for presentation semantics/API consistency; current commit to be pinned by agent, not a Rust implementation baseline. |
| REF-04 | [DICOM PS3.3, official](https://dicom.nema.org/medical/dicom/current/output/html/part03.html) | Image Plane Module, Pixel Data Module, Modality LUT/VOI LUT and presentation semantics. Common manifesto inspected consolidated 2026d; revalidate at actual kickoff. |
| REF-05 | [DICOM PS3.14, official](https://dicom.nema.org/medical/dicom/current/output/html/part14.html) | Display calibration reference only; **not** a qualification claim for FS-01. |
| REF-06 | [wgpu](https://github.com/gfx-rs/wgpu) | Candidate graphics/compute dependency. Actual release/lockfile, backend support, flags, texture format capabilities, transitive licences must be pinned by agent. |
| REF-07 | [egui](https://github.com/emilk/egui) | Optional development harness; not a dependency of the core. Exact licence/version/assets require lock/review. |
| REF-08 | [parry](https://github.com/dimforge/parry) | Optional future geometric-query library; not used in FS-01. Upstream permissive licence details/transitives to be checked when admitted. |
| REF-09 | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) | Naming, ownership and compatibility guidance, not an engine dependency |

**Do not treat the older C++ repository or the previous general-purpose `voxelia-rs` thought experiment as source authority.** Future `raster-labs` Rust repository pins must be added once actually observable. Links to DICOM 'current' are research entry points; an executable build must pin edition, specific clauses and permitted normative datasets.

## 3. Dependency and licence admission table

| Library | Intended role | First slice? | Compliance obligation |
|---|---|---|---|
| `wgpu` + Naga/WGSL toolchain | GPU graphics and compute | Yes | wgpu permissive dual licence at relevant pinned revision; preserve selected licence and notices; inspect transitive code/assets and driver/runtime components |
| `glam` or limited in-house matrices | Camera and affine math | Optional | Review selected version and ability to represent authoritative f64 geometry; avoid needless public dependency |
| `ndarray` | CPU array adapters | No default | Optional only when benefits observed; does not define core ownership/strides |
| `egui`, `egui-wgpu`, `winit` | Optional example/harness | Example only, later | Separate dependency lock/asset/font licence and event-loop ownership; no core import |
| `parry3d` | Mesh/spatial queries | No | Not a Marching Cubes backend; check pinned licence and transitives before later admission |
| VTK | General-purpose rendering toolkit | Prohibited | Excluded by approved decision |
| DICOM parsing/codec engines | Parser/codec function | Prohibited | DICOMflux and Flux Imaging Suite own these independently |
| SolidCraft code | Architecture reference | No copied source by default | Existing repository has MIT and Apache notices; preserve attribution and source-specific rights for any separately approved adaptation |
| Voxelia/DICOMkit code | Project-owned reference | No automatic copying | Verify ownership of any adapted source, transitive licences and exact revisions |

**Commercialisation:** MIT project code can be commercialised and redistributed subject to notice; it does not reserve commercial exploitation exclusively to Raster Images. Apache-2.0 is generally compatible with use by MIT-licensed commercial applications, but upstream code remains Apache-2.0 and has its own notice/patent conditions. The licence register must be generated from the *actual locked dependency versions*, not a generic statement that every crate is MIT. Contributor copyright assignment is an **inbound contribution condition** governed by the manifesto; a DCO/sign-off alone is not equivalent to executed assignment.

## 4. SolidCraft technical lessons and limits

Verified under `tangency-labs/solidcraft` observed commit: `viewport` owns CPU camera/scene math, `viewport-gpu` owns wgpu pipelines/GPU tessellation, `viewport-egui` adapts a viewport to egui; `docs/layers.md` describes dependency graph checks and independent shared-crate tests. `viewport-gpu` documents GPU vertex buffer production without round-tripping to CPU for CAD tessellation. Adopt independent headless/offscreen execution, adapter policy, render receipts and layer checks; do **not** import CAD B-rep, feature history, joint solver or claim that CAD tessellation proves medical Marching Cubes algorithms. Do not copy code without targeted review and rights provenance.

## 5. Items open for implementation review (not new architectural approvals)

1. **Repo identity**: New Rust repository under `raster-labs`, authorised branch, initial SHA, reviewer and agent.
2. **Rust environment**: exact stable compiler/MSRV, edition, linker/cross-target and updated wgpu/egui compatible releases and transitive notices.
3. **Validation subprofiles**: signed/unsigned 16-bit canonical sample convention, bit normalization, optional u8, rounding/tolerance for byte quantisation, padding rendering policy.
4. **GPU representation**: integer sampled texture/manual interpolation versus buffer texture/float-converted transient; feature/format compatibility proven on target GPU.
5. **Source boundary**: exact future DICOMflux decoded frame/lease/geometry descriptor compatibility (no release coupling until agreed).
6. **Hardware**: actual Windows/Linux x64/ARM64 devices, Vulkan/D3D12 drivers, headless modes and qualified capability matrix.

These are ordinary engineering review points within an already-approved architecture, not permission to reverse the nine accepted decisions.

## 6. Integrity and publication

`SHA256SUMS_v0.2.0.txt` records exact hashes for the generated files, and the ZIP is a self-contained publication copy. The checksum list deliberately excludes itself and the ZIP to avoid self-reference. Neither SHA-256 nor a Drive upload by itself proves clinical readiness, author identity or qualified code. Do not overwrite historical baselines without version increment and review.
