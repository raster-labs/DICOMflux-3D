# Source Register, Provenance & Traceability (`FS-01`)

- **Crate**: `dicomflux-3d` `0.1.0` (`FS-01`)
- **Copyright & Outbound Licence**: `Copyright (c) 2026 Raster Images` — MIT (`LICENSE`)
- **Executed Git Commit (`HEAD`)**: `7ed0d8d5fa67414fc4a6b66c2d04322d4236de55`
- **Target Remote Authority**: `TBD—DEVELOPER TO IDENTIFY` under `https://github.com/raster-labs` (per `DF3D-DEC01`–`DEC09`)
- **Authorship & Provenance Statement**: Independent Rust implementation authored with documented provenance against DICOM PS3.3 `2026d` and the sealed `DICOMflux-3D` `v0.2.0` specification pack. Does not claim a formal two-team clean-room process. Zero code was ported, translated, or adapted from the retired C++ `DICOMflux` prototype, `Voxelia`, `SolidCraft`, `VTK`, `GDCM`, `DCMTK`, or any GPL/LGPL source.

## 1. Sealed `DICOMflux-3D` `v0.2.0` Handoff Authority Digests

| Authority File | SHA-256 Digest |
| :--- | :--- |
| `docs/handoff/DICOMflux-3D/00_START_HERE_v0.2.0.md` | `64b25f51e57389b85816a40d5bad230bc65525a47e5cb482fdbb24569a40f2a0` |
| `docs/handoff/DICOMflux-3D/01_Foundation_Architecture_v0.2.0.md` | `2afb737feb48fdd005265e49405e3b15cfb9c2b18f37d1b3c6db5cb8f7321b66` |
| `docs/handoff/DICOMflux-3D/02_First_Slice_SRS_v0.2.0.md` | `fcf049c1ff98ffd54d132bf96d153769395f5d7087b46bbd950761cec5888db6` |
| `docs/handoff/DICOMflux-3D/03_Data_and_API_Contracts_v0.2.0.md` | `da9eeb7d11fd1ef4d63e851f147f3e5ad825a51a0d606020eb65b0c0b813d60d` |
| `docs/handoff/DICOMflux-3D/04_Verification_and_Evidence_Plan_v0.2.0.md` | `ef25bfd086e1be308369250fe729ee10464450b42cef7f605347625d276d1106` |
| `docs/handoff/DICOMflux-3D/05_Coding_Agent_Instructions_v0.2.0.md` | `5bac4332aadb24c88b5aaccf4cb03e69657e9b37221d601a04704f7cdde97a4d` |
| `docs/handoff/DICOMflux-3D/06_Decisions_References_and_Licences_v0.2.0.md` | `88f2a4395bc206e8caa672331315ecc1a1c9b3ccadae0f89d2d3be61f05acdf3` |
| `docs/handoff/DICOMflux-3D/07_Admission_and_Review_Checklist_v0.2.0.md` | `080d077c6dd4e05423940294a5127dafa6be9db864ec86f591a6519eb2096fec` |
| `docs/handoff/DICOMflux-3D/DICOMflux-3D_Foundation_Human_Guide_v0.2.0.pdf` | `dc1950b506e3a019c4c121e1794f9357847abc7117287cb936a35c36f9034f48` |
| `docs/handoff/DICOMflux-3D/FS01_Test_Case_Manifest_v0.2.0.json` | `3dccee9cd4794a1f22e94c1df9393fd5eb5d16fdeef5894048700782700afb1c` |
| `docs/handoff/DICOMflux/DICOMflux_and_Flux_Imaging_Suite_Development_Manifesto_v1.3.1.md` | `b15ed5cea296ab9dc1c60e879a178eb18c6758e123cac5f97d3bacb5b868234c` |
| `docs/handoff/DICOMflux/DICOMflux_D0_Foundation_Work_Order_v0.1.0.md` | `6363ec4e63949893a45a82e4bd28518faa872e81aa62ee40c302d75ae4531e19` |
| `docs/handoff/Flux_Imaging_Suite/Flux_Imaging_Suite_F0_Foundation_Work_Order_v0.1.0.md` | `89a3b925061b26e5304263dabdaec4499dbb9dbf680ca95e0e9358378d1e87f9` |

## 2. Implementation & Lockfile Artifact Digests

| Artifact | SHA-256 Digest |
| :--- | :--- |
| `Cargo.toml` | `3cc9858128f8d616c3c3b822c53aecff4ee01ceb98a7e81ffb3ed0a403f9183f` |
| `Cargo.lock` | `263d07e1af9ab0938aa96540269eee6d51a8bbc42eb849385c6e668e13865444` |
| `rust-toolchain.toml` | `24a5037a8f0a264369ef37bb46e019ade6c524beaead1c28f77a429f856a066b` |
| `LICENSE` | `744cf02040db2b51c4615804aceb0c8b812f8fc753369615a5d5c240f07bbdcf` |
| `README.md` | `8823c12b3d277104b6fb8a760ad2fa36f1d8d9fd1e538fbc2faaa9dab381f7e4` |
| `src/rendering/gpu/voi_linear.wgsl` | `b4b0ad1b6fd0a1b24555d83544d1bfe4390bc11e296a674c7ec7a66818383701` |
| `examples/independent-consumer/Cargo.toml` | `6f5941181fa3693693053b1e2df9df4d8c0f2fe82d2dfddcfaef7624318c60ee` |
| `examples/independent-consumer/Cargo.lock` | `a7c4cc157d9a7e141e02e437b7304bb2622795ac05b8142595520478cfc75733` |

## 3. Normative DICOM Standard Clause Pins (Consolidated Edition `2026d`)

| Standard Clause | Role in `FS-01` |
| :--- | :--- |
| DICOM `PS3.3` (`2026d`) `C.7.6.2.1.1` (Image Plane Module) | Patient `LPS` millimetre coordinate frame, `ImagePositionPatient`, `ImageOrientationPatient [Rx,Ry,Rz,Cx,Cy,Cz]`, and `PixelSpacing [dr, dc]` row-then-column convention (`src/spatial.rs`, `src/volume.rs`). |
| DICOM `PS3.3` (`2026d`) `C.7.6.3.1` & `PS3.5` `§8.1.1` (Pixel Data) | Canonical 16-bit sample container (`BitsAllocated = 16`, `1 <= BitsStored <= 16`, `HighBit = BitsStored - 1`), signedness, and stored-domain `PixelPaddingValue` / `PixelPaddingRangeLimit` (`src/image.rs`). |
| DICOM `PS3.3` (`2026d`) `C.11.1.1.2` (Modality LUT) | Finite linear modality transformation `x = s * RescaleSlope + RescaleIntercept` (`src/presentation.rs`, `src/rendering/cpu.rs`, `src/rendering/gpu/uniforms.rs`). |
| DICOM `PS3.3` (`2026d`) `C.11.2.1.2.1` (VOI LUT `LINEAR`) | Exact half-sample `LINEAR` windowing (`c - 0.5`, `w - 1.0`, `w >= 1.0`) including `w == 1.0` binary threshold (`src/rendering/cpu.rs`, `src/rendering/gpu/voi_linear.wgsl`). |
| DICOM `PS3.3` (`2026d`) `C.7.6.3.1.2` (`PhotometricInterpretation`) | Final-stage `MONOCHROME2` identity vs. `MONOCHROME1` (`255.0 - y`) polarity prior to `round_ties_even` quantization. |
| DICOM `PS3.14` (GSDF Reference Boundary Only) | Referenced solely to declare that `FS-01` outputs `DisplayEncoding::UncalibratedSrgbU8` and claims **no** GSDF diagnostic calibration. |

## 4. Direct Third-Party Dependency Licence Summary

Full transitive package versions, lockfile SHA-256 checksums, and SPDX expressions for all `83` resolved crates are recorded in [`evidence/SBOM.json`](SBOM.json).

| Crate | Scope | Locked Version / Feature Profile | SPDX Licence |
| :--- | :--- | :--- | :--- |
| `wgpu` | Direct Runtime | `30.0.1` (`default-features = false`, `features = ["std", "parking_lot", "wgsl", "metal", "vulkan", "dx12"]`) | `MIT OR Apache-2.0` |
| `sha2` | Direct Runtime | `0.11.x` (RustCrypto SHA-256 digest for WGSL shader provenance) | `MIT OR Apache-2.0` |
| `serde_json` | Dev-Only (`[dev-dependencies]`) | `1.x` (layering metadata & evidence JSON generation only) | `MIT OR Apache-2.0` |

## 5. Requirements-to-Verification Traceability Matrix (`FS01-R01`–`FS01-R18`)

| Requirement | Implementing Modules | Verification Cases |
| :--- | :--- | :--- |
| `FS01-R01` | `Cargo.toml`, `rust-toolchain.toml`, `LICENSE`, `src/lib.rs` | `FS01-T01`, `FS01-T18` |
| `FS01-R02` | `src/image.rs`, `src/error.rs` | `FS01-T02`, `FS01-T03`, `FS01-T08`, `FS01-T16` |
| `FS01-R03` | `src/image.rs`, `src/presentation.rs` | `FS01-T02`, `FS01-T03`, `FS01-T05`, `FS01-T14` |
| `FS01-R04` | `src/spatial.rs`, `src/volume.rs` | `FS01-T10`, `FS01-T11`, `FS01-T16` |
| `FS01-R05` | `src/rendering/cpu.rs`, `src/presentation.rs` | `FS01-T04`, `FS01-T07` |
| `FS01-R06` | `src/image.rs`, `src/rendering/cpu.rs`, `src/rendering/gpu/uniforms.rs` | `FS01-T02`, `FS01-T05`, `FS01-T06` |
| `FS01-R07` | `src/rendering/gpu/{mod.rs,pipeline.rs,uniforms.rs,voi_linear.wgsl}` | `FS01-T12`, `FS01-T17` |
| `FS01-R08` | `Cargo.toml`, `src/lib.rs` | `FS01-T01`, `FS01-T18` |
| `FS01-R09` | `src/rendering/gpu/mod.rs`, `src/presentation.rs` | `FS01-T14` |
| `FS01-R10` | `src/rendering/gpu/capabilities.rs` | `FS01-T13` |
| `FS01-R11` | `src/resource.rs`, `src/rendering/gpu/{pitch.rs,mod.rs}` | `FS01-T09`, `FS01-T13` |
| `FS01-R12` | `src/error.rs`, `src/resource.rs`, `src/rendering/gpu/mod.rs` | `FS01-T15` |
| `FS01-R13` | `src/presentation.rs`, `src/rendering/{cpu.rs,gpu/mod.rs}` | `FS01-T12`, `FS01-T17` |
| `FS01-R14` | `examples/generate_evidence.rs`, `evidence/*` | `FS01-T17`–`FS01-T22` |
| `FS01-R15` | `README.md`, `SECURITY.md`, `src/error.rs`, `evidence/LIMITATIONS.md` | `FS01-T23`, `FS01-T24` |
| `FS01-R16` | `examples/independent-consumer/*` | `FS01-T01` |
| `FS01-R17` | `Cargo.toml`, `tests/fuzz_and_governance.rs` | `FS01-T18` |
| `FS01-R18` | `src/image.rs`, `src/presentation.rs` | `FS01-T08`, `FS01-T16` |

## 6. Synthetic Fixture & Oracle Register (`fixture-set-0.1.0`)

All `FS-01` fixtures are original deterministic synthetic ramps, geometric phantoms, and adversarial seeds authored by Raster Images (`Copyright (c) 2026 Raster Images`, MIT) containing zero patient data.

| Fixture Key | SHA-256 Digest |
| :--- | :--- |
| `F01-INDEPENDENT-CONSUMER` | `9e6e520fdcbaeea2eba7e90e9fd02d2b0c8ed5ccc948ef37e37b1ca0df47f4ed` |
| `F02-SIGNED-12BIT-I16` | `1c39cff42a378ccfa55131e905c5e9447635432b68bbfcfb24d16588a5c9e36d` |
| `F03-UNSIGNED-16BIT-U16` | `2f5ca7c468b5c080948bb7529dc56b8cee1502426cf92835d5667db2d44dd413` |
| `F04-DICOM-LINEAR-VOI` | `46f90e6d352e06c67db586824002f5b1bcf312c4a4e3ed4baa9dc2d4eba01bad` |
| `F05-MODALITY-RESCALE` | `e87a0706d2afd39ad88c38ce0cab4c3e522f01629e733c0f4d1eede6c14ae50b` |
| `F06-PIXEL-PADDING` | `bc9ae4c7405fb1d68372d4a1578215227fe708f44e2a876763a9d41878f55563` |
| `F07-MONOCHROME-POLARITY` | `06424edbfb6aff1008eb3138fec532c1f2f3cf6eebdbce417badd99c76d70c46` |
| `F08-INVALID-AND-UNSUPPORTED` | `62fcdf286d3ca1e20a15ac89d96c06aba3836ea683c47b11fadb9d390ec49605` |
| `F09-BUDGET-BOUNDARIES` | `c1ed9ddf2fc3cd736742e70f67f612b7da6f4e6e7d0fa2f0aad6e802c7cee37a` |
| `F10-PATIENT-PLANE-LPS` | `8ca0b92719b92cf0325636f3489f017db928ecc88249a9664649bb7b5d882bd5` |
| `F11-REGULAR-VOLUME-3D` | `e896a896efe62a2c7bc9939cd12f76f6e2f7478e1b76f4d546c11505924e54e0` |
| `F12-PARITY-512-I16` | `c9f0c0ac9ca433afcb2685bc95d44ae94d8fba7da331bd2ec85957d959c27b2f` |
| `F13-GPU-CAPABILITIES` | `06e941f87f5bda51e268008001012425e18e89ba94e73895143087f8957cca55` |
| `F14-WL-RESIDENCY` | `eed956ebd2b0cc15dd2f25f9a999c67d7ffc49d3ac3e1da6957b677783551c35` |
| `F15-GPU-FAULTS-AND-CANCEL` | `7273f1f214bdb69b96cbb6d1711d2ff2329d257e117395a4a082e5706aa0018d` |
| `F16-SPLITMIX64-SEED` | `1cc7bd565b6b02b52926691f480eb6f1c3f7a8e876b62cdfa7d3d3861eb97506` |
| `F17-WGSL-SHADER` | `b4b0ad1b6fd0a1b24555d83544d1bfe4390bc11e296a674c7ec7a66818383701` |
| `F18-CARGO-LOCK` | `263d07e1af9ab0938aa96540269eee6d51a8bbc42eb849385c6e668e13865444` |
| `F22-BENCH-2048-U16` | `876e4593d0eb3decfe4d36b9ff86d738f6966997c17f1acffbf4a14c4e7b7ff5` |
| `F23-PRIVACY-REDACTION` | `bd7444826344bc1d7b7e016f9a93baf564ec383e0661ff51a26b1bc8c0bac9ae` |
| `F24-README-SCOPE` | `8823c12b3d277104b6fb8a760ad2fa36f1d8d9fd1e538fbc2faaa9dab381f7e4` |
| `BENCH-ORACLE-512x512` (CPU `f64` output) | `264b2d64e91997ab24f692fdf02aac831ee0afc775a820eb6675ceb8a9130b9b` |
| `BENCH-GPU-512x512` (`wgpu` output) | `264b2d64e91997ab24f692fdf02aac831ee0afc775a820eb6675ceb8a9130b9b` |
| `BENCH-ORACLE-2048x2048` (CPU `f64` output) | `01f0a1c9309645e221d214838b81b30b0fbb37d377749cc91b8daeec00616cc0` |
| `BENCH-GPU-2048x2048` (`wgpu` output) | `01f0a1c9309645e221d214838b81b30b0fbb37d377749cc91b8daeec00616cc0` |
