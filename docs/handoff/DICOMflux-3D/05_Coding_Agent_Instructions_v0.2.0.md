# 05 — DICOMflux-3D FS-01 Coding-Agent Instructions

**Version/date:** 0.2.0 / 10 November 2026  
**Status:** Prepared instruction package, **not an active agent run or evidence of implementation**. Owner has approved architecture DF3D-DEC01–09 and requested this documentation; repository and execution commission remain developer-controlled.

## 1. Scope and authority

Implement **one narrow, independently testable Rust slice** of DICOMflux-3D: bounded greyscale 2D presentation with an analytic CPU DICOM LINEAR VOI reference and an offscreen wgpu/WGSL backend, plus checked immutable sample/2D plane/regular-volume descriptors. Do not turn the project into a general-purpose engine, move work into DICOMflux or Flux imaging crates, or import parsing/codec code. Stay within FS01-R01–R18 in `02_First_Slice_SRS_v0.2.0.md` and contracts in `03_Data_and_API_Contracts_v0.2.0.md`.

The project's GitHub authority is `https://github.com/raster-labs` ONLY. The historic C++ DICOMflux repository is retired and explicitly excluded from source and implementation authority. At 10 November 2026 no public Rust DICOMflux-3D repository was verified in this account; **do not guess a repo name, invent a branch/commit or modify an unrelated repository**. Start execution only after a developer supplies actual authorised repository URL, branch, access permissions, named reviewers and work-order ID. Do not create a repository from this document alone.

### Common governing rules

Read the original [Development Manifesto v1.3.1](https://drive.google.com/file/d/1J7KKk062Rv6kTZI7lirs5DAbNqOVFaCl/view) before a code change, especially independent Rust implementation/author rights, bounded memory, concurrency, platform profiles, API conventions, test evidence, and one-way dependencies. Approved DF3D-DEC01–09 are recorded in `06_Decisions_References_and_Licences_v0.2.0.md`. Later changes to an approved architectural boundary require explicit owner disposition; routine Rust coding decisions within the assigned work order do not.

## 2. Required preflight (stop if unresolved)

1. Record authorised `raster-labs` repository URL, branch, starting commit, who can review, issue/work-order ID and baseline protection rules.
2. Pin exact stable Rust toolchain/MSRV, edition, `wgpu`, WGSL tooling and optionally `glam` versions. Do not assume the documentation's examples reflect actual installed dependencies; verify `Cargo.lock` and licences/transitive assets.
3. Verify DICOM consolidated publication (2026d as dated common-manifest snapshot) plus any approved Final Text relevant to VOI and geometry on **actual agent kickoff date**. Record edition, clause, URL and digest where permitted. DICOM window formula, sign extension and pixel padding expectations must be written before shader code.
4. Verify the permitted project-owned reference revisions (Voxelia architecture, DICOMkit and DICOMcanvas where relevant), note any third-party source and avoid prohibited wholesale reuse. SolidCraft may inform design, not a substituted CAD/render engine.
5. Confirm the developer's first host platforms, available GPU/driver, permitted test hardware, image fixture rights and evidence location. If primary platform devices are unavailable, report a *bounded development build* rather than full platform qualification.
6. Read source and licence notices and confirm permissive dependency graph, external copyright assignments where accepted contributions exist, and mitigation for any restricted source. Do not claim MIT licences on unverified upstream code.

## 3. Recommended incremental implementation path

**Checkpoint F0 — repository readiness:** real Cargo crate with MIT licence, README, contribution/copyright policy, security policy, pinned toolchain, lockfile, CI skeleton, metadata and independent-consumer example path. Run `cargo fmt`, `clippy`, `test`, metadata tree and dependency licence checks; keep evidence.

**Checkpoint F1 — checked types and CPU oracle:** define minimal `FrameDescriptor`, `PatientPlane`, `CheckedVolume` validation, resource budgets and typed errors. Implement DICOM LINEAR centre/width and rescale for admitted 16-bit greyscale, immutable source, explicit polarity and padding policy. Use independent synthetic oracle values and negative cases first. No GPU requirement for these tests.

**Checkpoint F2 — wgpu offscreen:** select an adapter by host policy; report feature/format/limit capability; allocate bounded 2D source representation; run WGSL VOI transform; safely read back to a complete 8-bit image. Treat integer texture filterability and float conversion carefully; do not silently downcast. Implement device/queue lifetime/cancellation and allocation failure modes.

**Checkpoint F3 — correctness/ownership:** run analytic, CPU and GPU parity tests; fuzz malformed descriptors; demonstrate cancellation/resource release and no partial-success. Add documented output provenance, synthetic LPS geometry/orientation tests and source-preservation receipts.

**Checkpoint F4 — independent consumer and review:** build a separate consumer outside workspace; run on at least actual development hosts; provide PR, dependency/SBOM report, case-by-case results, platform gaps, benchmark observations and required screenshots/offscreen checks. Do not move to MPR/MIP/DVR/segmentation until a subsequent work order.

At each checkpoint, keep a small executable Rust deliverable; don't produce only new plans instead of code once an actual coding-agent work order has been authorised.

## 4. Strict prohibitions

- No DICOM parser, dataset model, codec, network service, SEG writer, SR/reporting or DICOM media handling in DICOMflux-3D.
- No VTK, OpenJPEG, GDCM/DCMTK/dicom-rs or substitute renderer/codec engine as a shipping shortcut.
- No UI toolkit in the minimal rendering library's public dependency graph. `egui`/`winit` permitted in optional example/harness only. No workstation GUI, Java/Swing, Kotlin/AAR, Android or WebGPU work in FS-01.
- No silent source mutation, irreversible 8-bit source conversion, automatic resampling of irregular volumes, inferred geometry from file order, hidden GPU memory oversubscription or unbounded staging/readback.
- No fabricated `cargo test`/CI receipts, unsupported performance numbers, cross-compiled-as-executed claims or clinical qualification.
- No 'refactor' of historical C++ DICOMflux or Apple Voxelia code and no automated code translation of third-party copyleft source.

## 5. Required deliverables from the future agent

1. Narrow Rust source changes in an authorised PR; public examples and internal WGSL shader code, plus unit and integration tests.
2. `SOURCE_REGISTER.md`, dependency lock/SBOM, source/adaptation/notice provenance, with actual Rust and shader versions.
3. FS01-R01–R18 requirements-to-test matrix, and FS01-T01–T24 status with executed environment and log hashes.
4. Independently defined numerical oracles and rights-cleared synthetic fixtures with versions and SHA-256.
5. CPU/GPU comparison report including every pixel or evidence-safe aggregate *plus* explicit threshold/boundary tests.
6. Resource/cancellation/error reports, including unsupported-GPU path and memory budget refusals.
7. Independent outside-workspace consumer, build/test receipts and benchmark records with hardware/driver/OS.
8. New `SHA256SUMS` manifest for the exact agent-produced evidence and package; clear remaining limitations and next-gate recommendation.

## 6. Copy-ready coding-agent instruction

> **ASSIGNMENT: DICOMflux-3D FS-01 — Rust 2D medical greyscale presentation foundation.** The owner approved architectural decisions DF3D-DEC01–09 on 10 November 2026. Work in the separately assigned and verified repository/branch under `raster-labs` only (repository URL, SHA, branch, work-order and reviewer supplied by developer). Do not use the retired C++ DICOMflux repository. Follow the canonical DICOMflux/Flux Development Manifesto v1.3.1 and this handoff v0.2.0; read source baselines and licences at exact revisions. Implement only FS01-R01–R18: validated immutable 16-bit greyscale input, an analytic CPU DICOM LINEAR window reference with modality rescale and explicit padding/polarity, a bounded offscreen wgpu/WGSL path, validated LPS plane/regular-volume descriptors, capability/resource/error contracts, tests and an independent consumer. Report source SHA and versions, exact commands, real test results, GPU/OS/device, failure and provenance evidence, SBOM, SHA-256 manifest and limitations. Do not implement a DICOM parser, codecs, MPR/MIP/DVR, segmentation, a GUI, foreign bindings, Android or browser runtime. Do not claim clinical readiness. Escalate changes to approved boundaries; proceed with unaffected authorised work.

## 7. Developer handoff request

Before invoking the agent, the developer completes `07_Admission_and_Review_Checklist_v0.2.0.md` with actual repository, branch, assigned coding agent, permissions, first-slice acceptance gate, test resources, source pins and reviewer. Approval of this **documentation** is not a substitute for the developer's explicit repo/branch execution authorisation.
