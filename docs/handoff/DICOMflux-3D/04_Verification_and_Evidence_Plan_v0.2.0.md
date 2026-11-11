# 04 — Verification and Evidence Plan

**Version:** 0.2.0 | **10 November 2026** | **Status:** prospective validation plan, **no tests executed by this documentation package**.

## 1. Evidence policy and acceptance model

The first slice must be verified by actual tests and independently justified expectations. A successful screenshot alone is not acceptable evidence of DICOM greyscale correctness, patient-coordinate accuracy or safe ownership. Any implementation must identify repository, commit SHA, precise crate lockfile, tools, OS, hardware, GPU, driver and validation result. Cross-compilation is not a device execution. Never report work as passed before running it.

Evidence states: `NOT_RUN`, `PASS`, `FAIL`, `BLOCKED`, `WAIVED` (with signed reason). The provided JSON manifest starts **all tests as NOT_RUN**. Its specimen values are examples for the future developer to construct; they are not assertions of finished fixtures. Build and performance evidence must be regenerated per implementation revision.

For a proposed stage gate, use three independent dimensions:
1. **Semantic correctness** — CPU reference vs independently calculated DICOM equations and geometry oracle.
2. **GPU agreement and reproducibility** — output comparison within explicitly justified per-format tolerance and tested capability profile.
3. **Runtime safety/resource behaviour** — no over-allocation, corruption, partial-success, panics from malformed input, lost owner leases or in-flight use-after-free.

## 2. Reference oracles, not another renderer's screenshots

Pin the applicable edition and clauses from DICOM PS3.3 (VOI LUT Module and Image Plane) and DICOM PS3.14 only where claimed. The source manifest currently identifies consolidated **DICOM 2026d** as its 9 November 2026 inspection snapshot; recheck current Final Text corrections before implementing. Use analytically computed `LINEAR` window expected values, signed arithmetic at `BitsStored`, exact known LPS transforms and controlled synthetic gradients. Third-party tool comparison may reveal discrepancies but does not itself define the standard. Comparison with Voxelia/DICOMcanvas output is useful regression context and should not be called an independent mathematical oracle.

Example DICOM LINEAR tests: with centre `50`, width `100`, lower threshold `-0.0` and upper threshold `99.0` (per PS3.3 half-sample convention), modality values at/under lower bound produce 0, at/above upper bound produce 255, interior values use the standard affine mapping. Independently compute expected values with the exact formula rather than hard-coding an unchecked intuitive HU range. Boundary cases with width 1 are mandatory. All output quantisation conventions must be recorded before comparing byte arrays.

For a 2D patient-space phantom with origin `(10,20,30)` LPS mm, column direction `(1,0,0)`, row direction `(0,1,0)`, spacing row=2 mm, column=0.5 mm, column 4/row 3 maps to `(12,26,30)` mm. Include rotated/oblique and negative direction cases. For a regular volume, check affine round trips, determinant, axis order and source provenance separately.

## 3. Planned test matrix

| Test ID | Test category | Expected disposition |
|---|---|---|
| FS01-T01 | Build minimal crate and independent consumer without UI/parser | Clean build; no forbidden dependency |
| FS01-T02 | Small `i16` frame with sign extension and 12-bit values | Raw values preserved; CPU oracle agrees |
| FS01-T03 | Small `u16` frame with nonzero high bit and validated precision | Correct valid-bit interpretation or explicit rejection |
| FS01-T04 | DICOM LINEAR centre/width: exact endpoints, midpoint and width=1 | Exact analytic branch behaviour and declared rounding |
| FS01-T05 | Negative slope or positive slope/intercept including HU example | Modality stage correct and immutable source |
| FS01-T06 | Pixel padding and padding-range mask treatment | Masking policy exact; no arbitrary value distortion |
| FS01-T07 | MONOCHROME1 versus MONOCHROME2 polarity | Declared output inversion and no codec-stage inversion |
| FS01-T08 | Invalid widths, non-finite values, illegal stride or short input | Typed failure, no panic/read past allocation |
| FS01-T09 | Huge dimensions/overflow and exhausted budget | Admission refuses before allocation |
| FS01-T10 | LPS patient-plane, orthonormality and row/column spacing phantom | Coordinates match analytic values |
| FS01-T11 | Synthetic 3D affine, singular transform and irregular descriptor | Admit valid; reject invalid/irregular explicitly |
| FS01-T12 | CPU vs wgpu output on actual adapter, representative image | Compare all pixels under recorded tolerance; strict boundary checks |
| FS01-T13 | Unsupported GPU texture and no suitable adapter | Typed capability error or explicit CPU policy |
| FS01-T14 | Repeated W/L changes | No source mutation or re-decoding; upload policy reported |
| FS01-T15 | GPU context drop/device loss/cancel during in-flight job | Safe lifetime and no false complete result |
| FS01-T16 | Fuzz invalid descriptors/strides/presentation options | No panics/UB/unbounded allocation on tested corpus |
| FS01-T17 | Headless offscreen output descriptor and receipt | Stable dimensions, stride, format, status, provenance |
| FS01-T18 | Dependency layering and licences/SBOM | No GUI/parser/codec backedge or restricted source |
| FS01-T19 | Windows x64 graphics + CPU run on qualified device | Actual device report, not just cross-compile |
| FS01-T20 | Linux x64 Vulkan graphics + CPU run on qualified device | Actual device report, not just cross-compile |
| FS01-T21 | Windows ARM64 / Linux ARM64 when available | Separate execution receipts; open until evidenced |
| FS01-T22 | Performance smoke (small/medium greyscale; repeated W/L) | Report medians/ranges and data; no unfounded FPS claim |
| FS01-T23 | No patient information in logs/errors/fixtures | Privacy and provenance audit |
| FS01-T24 | README capability and clinical-safety language | No unsupported operation claims |

The machine-readable `FS01_Test_Case_Manifest_v0.2.0.json` enumerates these as **NOT_RUN**. Future tests may add subcases without reassigning ID semantics; maintain case counts and dataset versions.

## 4. Dataset fixture governance

Use small original synthetic ramps/checkerboards/negative-valued slices and independently constructed simple LPS geometry first. Rights-cleared DICOM test data may be integrated **through DICOMflux** in a later adapter slice; FS-01 does not ingest DICOM files. If a future fixture comes from the Imaging Data Commons (IDC), preserve licensing/permissions and source manifests, download and version the admitted dataset locally for regression use, and access cloud data only for hosted execution where authorised. Do not publish or embed identifiable patient content in open reports or screenshots.

A fixture record must include dataset version (`fixture-set-0.1.0` proposal), source permission, generator version/commit, sample format, dimensions, data SHA-256, geometry and expected oracle hashes. Increment the internal fixture dataset version whenever samples or expected values change; prior versions remain retained. An expected-output correction requires rerunning tests rather than inheriting historical passes.

## 5. CPU vs GPU agreement

The CPU reference computes source-to-modality and VOI stages with documented precision, then uses a declared quantisation policy. WGSL typically uses `f32` for the admitted GPU presentation calculations; compare every pixel to CPU and independent analytical expected values. Default suggested acceptance: exact branch selection at the documented threshold cases and absolute per-channel difference no greater than **1 8-bit output code** for ordinary finite interior values. Any exceptions require per-case justification, before implementation; no whole-image RMS-only tolerance that masks local failures. The GPU output must not be compared only against its own golden images.

Test one software adapter separately if useful, but do not present software GPU success as proof of hardware GPU or driver qualification. Log backend (D3D12/Vulkan), adapter, actual driver, wgpu version and shader hashes. Tests requesting unsupported features should fail clearly, not silently return a different numerical profile.

## 6. Resource, failure and concurrency evidence

Track separately: source retained bytes, CPU output bytes, staging bytes, resident textures, in-flight commands, temporary GPU buffers and actual successful/completed readbacks. Compare planned reservations against successful releases. Fault injection should cover `ResourceLimits` at exact boundary, one byte over limit, integer overflow, denied adapter, denied allocation, device loss and cancellation before or after submission. Test repeated context creation/destruction, concurrent requests and abandoned consumers. Distinguish `UnsupportedGpuCapability`, `GpuDeviceLost`, `ResourceLimitExceeded`, `Cancelled` and unhandled defects.

No local file output or GPU texture is marked complete until the buffer is safely produced. A cancellation or error must not overwrite an earlier valid output or release a GPU-retained input prematurely. If a GPU operation cannot be cancelled after submission, mark it cancelled-to-caller but retain resources until fence/completion; document this limitation.

## 7. Performance baseline, not a sales claim

Run fixed small (e.g. 512×512) and medium (e.g. 2048×2048) bounded greyscale frames where admitted, with varying window values. Measure CPU work, GPU upload, shader, output readback and end-to-end separately using appropriate synchronisation. At least five warmup and twenty measured repetitions are **suggested**; report sample distribution, median, p95, peak memory and GPU driver. Do not promise 60 FPS, millisecond targets, no-copy paths or device compatibility until measured. Do not infer shared memory means zero transfer cost. GPU measurements from a virtual/CPU adapter are separately labelled.

## 8. Evidence deliverable structure for future agent

```text
evidence/
  SOURCE_REGISTER.md          # source versions and provenance
  EXECUTION_ENVIRONMENT.md    # toolchain, OS, target, GPU and driver
  TEST_RESULTS.json           # case IDs, status, date, oracle/dataset version
  TEST_LOGS/                 # bounded redacted logs
  COVERAGE_AND_FUZZ.md        # corpus/seed/iterations and limits
  PERFORMANCE.md             # reproducible methodology and raw results
  LIMITATIONS.md             # unsupported profiles and open gates
  SBOM.json                  # dependency/licence inventory
  SHA256SUMS.txt              # integrity of delivered evidence
```

The current document package has only **design artefacts**, not any such executed output. Agent must return a small, reviewable PR, independent-consumer receipt and transparent PASS/FAIL/BLOCKED tables; the reviewer decides qualification and admission.

## 9. Stop conditions

Block FS-01 admission on source ownership/licence ambiguity, unauthorised repository, unresolved DICOM formula differences, invalid storage ownership, missing hard resource limits, absence of an independent CPU oracle, or untested claims. Report blockers to owner/reviewer while preserving completed, unaffected work. Do not change DF3D-DEC01–09 by unilateral agent decision.
