# DICOMflux-3D — Foundation Architecture and First-Slice Coding-Agent Handoff

**Package version:** 0.2.0  
**Issue date:** 10 November 2026  
**Programme owner:** Raster Images  
**Approval recorded:** Architectural decisions **DF3D-DEC01–DF3D-DEC09 approved by the owner on 10 November 2026**  
**Package status:** Documentation handoff prepared; first-slice work order is **proposed for separately authorised agent execution**. No implementation or qualification is claimed.  
**Canonical Drive destination:** https://drive.google.com/drive/folders/14rM4Vkl3WjmAyy3n1_cSKXDwB6_mZRQ4  
**Canonical GitHub account:** https://github.com/raster-labs  
**Licensing direction:** MIT for project-owned code and documents, Raster Images attribution, third-party notice preservation, external-contributor copyright assignment as governed by the manifesto.

## Read this first

DICOMflux-3D is a **DICOM-specific Rust imaging-presentation and 3D visualisation library**, with GPU-accelerated **2D** in its scope. It consumes checked decoded images and patient-space metadata from **DICOMflux**, uses **wgpu/WGSL** for graphics/compute, and does **not** implement a DICOM parser, codec, DICOM SEG writer, network service, workstation UI, or a generic Voxelia replacement. Its library interface is independent of `egui`, `winit`, Java and any application framework. The existing Apple-native path remains DICOMkit/SIS/Voxelia.

**Repository authority correction.** The owner expressly retired the historical lightweight C++ DICOMflux direction. Do **not** use or modify `Raster-Lab/DICOMflux` as a source of this project, do not list it as an implementation baseline, and do not migrate its C++ code. Use the new `raster-labs` account only. At package preparation on 10 November 2026, a public Rust `raster-labs/DICOMflux` or `raster-labs/DICOMflux-3D` repository could not be verified; a precise repository URL, initial commit, access and protected branch must be supplied by the authorised developer before agent execution. `raster-labs/DICOMKit` (historic repository spelling) is an Apple reference only, not a Rust implementation.

## Authority and order of reading

1. **Owner-approved decisions** in `06_Decisions_References_and_Licences_v0.2.0.md` (DF3D-DEC01–09; approval date 10 November 2026).
2. **DICOMflux and Flux Imaging Suite Development Manifesto v1.3.1** (9 November 2026), governing common rules, especially §2, §4.1, §7–13, §15–17: https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view
3. **This component's specific architecture** in `01_Foundation_Architecture_v0.2.0.md`.
4. **First-slice SRS and contract proposals** in `02_First_Slice_SRS_v0.2.0.md` and `03_Data_and_API_Contracts_v0.2.0.md`.
5. **Verification and agent handoff** in `04_Verification_and_Evidence_Plan_v0.2.0.md`, `05_Coding_Agent_Instructions_v0.2.0.md`, and `07_Admission_and_Review_Checklist_v0.2.0.md`.

Where the component documents conflict with the common manifesto, the owner's directions and manifesto prevail. Proposed types, test thresholds and implementation profiles in this handoff are **engineering recommendations**, not additional owner approvals. Material changes to the approved decisions require another recorded decision.

## Exactly what first slice is commissioned *for planning*

**FS-01: checked greyscale 2D presentation, with a minimal checked 3D geometry/volume descriptor.** The agent's future bounded implementation target is: a real Rust crate, validated immutable image descriptors and ownership, documented DICOM physical-coordinate metadata as caller-supplied checked input, deterministic CPU reference windowing, GPU wgpu/WGSL offscreen windowing, capability and resource admission, one independent consumer and rights-cleared/synthetic fixtures. There is no DICOM file reader in this crate.

The first slice **does not** implement MPR, MIP, DVR, transfer-function editor, threshold segmentation, Marching Cubes, bricked residency, a medical workstation, an embedded Java viewport, JNI/AAR or clinical qualification. These are explicitly later, separately gated delivery slices even though they form the accepted architectural scope.

## Package contents

| File | Purpose |
|---|---|
| `00_START_HERE_v0.2.0.md` | Scope, authority, status and reading sequence |
| `01_Foundation_Architecture_v0.2.0.md` | Responsibility boundary, rendering, device, storage, host architecture and roadmap |
| `02_First_Slice_SRS_v0.2.0.md` | Verifiable FS-01 requirements and exclusions |
| `03_Data_and_API_Contracts_v0.2.0.md` | Semantic types, ownership, presentation stages and ABI boundaries |
| `04_Verification_and_Evidence_Plan_v0.2.0.md` | Fixture taxonomy, oracles, tests and evidence rules |
| `05_Coding_Agent_Instructions_v0.2.0.md` | Copy-ready bounded agent prompt and checkpoints |
| `06_Decisions_References_and_Licences_v0.2.0.md` | Nine approved decisions, source register, dependency/licence decisions |
| `07_Admission_and_Review_Checklist_v0.2.0.md` | Execution prerequisites, completion criteria, sign-off fields |
| `FS01_Test_Case_Manifest_v0.2.0.json` | Machine-readable *planned* cases; not evidence of executed tests |
| `DICOMflux-3D_Foundation_Human_Guide_v0.2.0.pdf` | Human-readable, non-normative summary |
| `SHA256SUMS_v0.2.0.txt` | Integrity manifest for all packaged documents except itself and the ZIP |
| `DICOMflux-3D_Foundation_Handoff_v0.2.0.zip` | Archive of these documents and SHA manifest |

## Interpretation of evidence

- **APPROVED** applies to architectural decisions DF3D-DEC01–09, not to code, test results, publication readiness or clinical use.
- **PROPOSED** applies to FS-01 interfaces, execution order and objective pass criteria until a developer commissions a real repo/branch.
- **NOT EXECUTED** applies to all test cases and benchmarks in this package.
- Source links and SHA-256 values are provenance/integrity evidence only; they do not establish operational correctness or authenticity of the publisher.

**Next action:** nominate the new `raster-labs` Rust repository/branch, agent credentials, reviewer and verified toolchain/dependency lock; then issue the bounded FS-01 work order through the authorised development workflow. No new repository, commits or application code were created while preparing this package.
