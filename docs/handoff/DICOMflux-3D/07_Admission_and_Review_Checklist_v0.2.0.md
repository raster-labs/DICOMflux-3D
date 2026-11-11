# 07 — FS-01 Admission and Reviewer Checklist

**Version:** 0.2.0 | **Date:** 10 November 2026  
**Status:** Handoff template; *unchecked items are unperformed*.

## A. Approvals and boundaries

- [x] Owner approved **DF3D-DEC01–DF3D-DEC09** on 10 November 2026.
- [x] Owner replaced old C++ direction with Rust-only `raster-labs` authority; old repository is excluded.
- [x] Separate DICOMflux, Flux Imaging Suite, DICOMflux-3D, UI/application and Voxelia responsibility boundaries documented.
- [x] Work scope explicitly narrowed to FS-01 for coding-agent handoff.
- [ ] Named owner/developer accepts the concrete FS-01 work order and assigns actual repository/branch/credentials.

## B. Repository/agent admission record — to complete before execution

| Required field | Current state |
|---|---|
| New GitHub repository URL under `raster-labs` | **TBD; no public Rust DICOMflux-3D repo verified** |
| Default/target branch and starting full commit SHA | **TBD** |
| Assigned coding agent and developer | **TBD** |
| Work-order/issue ID, scope and authorisation date | **TBD** |
| Named reviewer and source/code ownership procedure | **TBD** |
| Rust stable version, MSRV, target triples | **TBD** |
| `wgpu`, WGSL and optional egui revision, lockfile digest | **TBD** |
| Actual Windows/Linux x64 and ARM64 test devices/GPU drivers | **TBD** |
| Fixture dataset version, rights and hashes | **TBD** |
| Applicable DICOM edition/Final Text source digest | **TBD** |
| Agent evidence/report destination | **TBD** |

If any required execution authority is missing, this document remains a **documentation-ready** plan, not permission to run an agent against a guessed repository.

## C. Implementation acceptance (all initially open)

- [ ] Actual Rust crate built on a supported primary host; independent outside-workspace consumer operates.
- [ ] Exact scope `FS01-R01–R18` implemented or accurately recorded as blocked/nonconforming.
- [ ] CPU DICOM LINEAR oracle and negative/edge cases pass with fixed expected outputs.
- [ ] Signed 12-bit stored values, modality rescale, polarity and padding policies pass.
- [ ] wgpu offscreen output matches CPU and independent analytic oracles under signed tolerance.
- [ ] GPU capability limits and unsupported-device paths are explicitly tested.
- [ ] Patient-space plane and regular affine descriptions pass independent geometry phantoms.
- [ ] Memory accounting, in-flight ownership, cancellation and device-loss tests pass.
- [ ] Fuzz, clippy, dependency layering, licences/SBOM and provenance checks pass.
- [ ] No DICOM parser, codec, GUI, workstation, Android/web binding or premature 3D algorithms included in FS-01.
- [ ] Actual host/device/driver receipts, test manifest, limitations and SHA-256 evidence attached.
- [ ] Named reviewer signs FS-01 admission and opens separate subsequent work order(s) for MPR/MIP/DVR/segmentation.

## D. Requested agent result summary format

```text
Repository:
Commit/branch:
Work-order:
Reviewer:
Date:
Source/fixture revisions:
Rust toolchain / wgpu / shader SHA:
Host OS / target / CPU / GPU / driver:
Implemented requirements:
Case count: PASS / FAIL / BLOCKED / NOT_RUN / WAIVED:
CPU numerical oracle:
GPU parity/tolerance:
Memory and device-loss evidence:
Independent consumer:
Licence/SBOM status:
Known limitations:
Scope deviations and owner dispositions:
SHA256SUMS file:
Recommended next decision: ADMIT / HOLD / REWORK:
```

## E. Review decision and signature

**Owner architecture approval**: recorded; not being reopened.  
**First-slice implementation status**: **NOT STARTED / NO EXECUTION CLAIMED BY THIS PACKAGE**.  
**Final reviewer decision**: **PENDING**.  
**Clinical/device qualification**: **NOT COMMISSIONED / NOT CLAIMED**.

This is a preparation package only; the owner/developer will control any later implementation and its separate acceptance gates.
