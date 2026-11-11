# Contributing to DICOMflux-3D

## Outbound Licensing and Mandatory Copyright Assignment

`dicomflux-3d` is owned by **Raster Images** and distributed under the outbound **MIT License** (`Copyright (c) 2026 Raster Images`; see [LICENSE](LICENSE)).

- **Mandatory Executed Copyright Assignment**: Before any external contribution (pull request, patch, shader, test fixture, or documentation) can be accepted into `dicomflux-3d`, the contributor (and their employer, where applicable) **must execute a written copyright assignment agreement transferring ownership to Raster Images**.
- **DCO / Inbound=Outbound Insufficient**: A Developer Certificate of Origin (`Signed-off-by` / DCO) or reliance on GitHub's default `inbound=outbound` licensing terms alone is **not sufficient** for acceptance. Contributions without a verified copyright assignment on file with Raster Images will be closed unmerged.

## Provenance and Prohibited Reference Sources

`dicomflux-3d` is authored as an **independent implementation with documented provenance** against published standards (including consolidated DICOM PS3.3 `2026d` clauses and WebGPU / WGSL specifications) and Raster Images' approved specifications. Contributors must **never** claim a formal two-team clean-room process unless such a segregated protocol was formally executed and audited.

To protect licensing integrity and architectural separation (`DF3D-DEC02`, `DF3D-DEC08`, `DF3D-DEC09`):

- **Prohibited Sources**: Do **not** copy, port, translate, or derive code, shaders, or test fixtures from the retired experimental C++ `DICOMflux` prototype, Swift/Metal `Voxelia`, VTK, ITK, GDCM, DCMTK, Orthanc, 3D Slicer, or any GPL, LGPL, AGPL, MPL, or non-commercial/source-available codebase.
- **Permitted Dependencies & Notice Preservation**: Runtime dependencies are restricted to approved permissive crates (`wgpu` and `sha2` in `FS-01`). All locked third-party SPDX licences and copyright notices must be preserved accurately in `Cargo.lock`, `evidence/SBOM.json`, and `evidence/SOURCE_REGISTER.md`.
- **Synthetic Fixtures Only in `FS-01`**: Never commit clinical datasets or Protected Health Information (`PHI`). All `FS-01` tests use deterministic synthetic buffers and analytical geometry phantoms.

## Engineering and Pre-Commit Verification Gates

All changes are developed directly on `main` under `#![forbid(unsafe_code)]`, `#![deny(missing_debug_implementations)]`, and `clippy::all = "deny"`. Every change must pass all mandatory gates before commit:

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```
