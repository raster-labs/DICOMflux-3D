# 01 — DICOMflux-3D Foundation Architecture

**Version:** 0.2.0 | **Date:** 10 November 2026 | **Status:** Component architecture under approved DF3D-DEC01–09; FS-01 is a proposed bounded implementation slice.

## 1. Mission and explicit non-goals

Build a focused DICOM-aware Rust library for GPU-accelerated 2D presentation, geometry-aware slicing and 3D visualisation, with later MPR, MIP/MinIP, DVR, transfer functions, segmentation overlays and bounded mesh work. The library is DICOM-specific rather than Voxelia's general-purpose, multi-layer toolkit. No VTK, no shadow DICOM parser, no imported compression engine, no medical workstation implementation, no host UI or hidden clinical claims.

It may define **DICOM-specific semantic descriptors** to consume DICOMflux outputs; DICOMflux remains responsible for dataset parsing, SOP/Transfer Syntax interpretation, encapsulation, frame decoding orchestration, series assembly policy, metadata validation and DICOM output. Flux Imaging Suite owns codecs and generic resampling. A DICOM SEG writer belongs in DICOMflux, not DICOMflux-3D. Calling applications own studies/worklists, UI layout, commands, persistence and network access. Apple applications continue to use Voxelia for 3D.

## 2. Dependency direction and seam

```text
Windows/Linux native host       Java/Swing DICOM Capacitor        headless consumer
          |                                |                           |
          +------------ versioned application-facing contracts --------+
                                        |
                                   DICOMflux-3D
                    checked image/geometry, views, session state
                           /                 |               \
                   CPU oracle           GPU wgpu/WGSL      output adapters
                                            |
                                    GPU/offscreen output

DICOMflux [DICOM source/decoding/geometry admission] ---> checked buffers + metadata
      |
Flux Imaging Suite [codec/resampling algorithms; independent of DICOMflux]
```

**Dependency prohibitions:** `DICOMflux` and `Flux*` crates MUST NOT depend on DICOMflux-3D, wgpu, egui or the UI; the DICOMflux-3D core MUST NOT require an egui event loop, file dialog, network stack, patient database, cloud endpoint or GPU to validate an input descriptor. No dataset or SOP instance is opened inside the renderer.

The **seam** includes buffer owner/lease, sample format, shape/strides, dimensions, row/column pixel spacing, image position/orientation or validated index-to-patient affine, pixel-value transform, photometric/presentation attributes, precision, provenance identifiers and explicit source frame identity. Constructing such an admitted descriptor from real DICOM is the DICOMflux adapter's job. A synthetic producer may construct one for tests. Release APIs must reject unsupported input instead of guessing.

## 3. Internal architecture without artificial product layers

One publishable library, `dicomflux-3d` (subject to Cargo name availability), private modules for: `image`, `volume`, `spatial`, `presentation`, `camera`, `rendering`, `segmentation`, `resource`, `error`. Graphics implementation under `rendering::gpu` and WGSL files; CPU numerical reference under `rendering::cpu`. `egui` and `winit` appear only under examples or optional standalone harness tooling. Internal modules are for correctness and testability, not separately versioned VoxeliaCore/VoxeliaMedical analogues.

**Core contract**: validated immutable source + metadata, render session/context independent of UI, requests as value objects, explicit capability/availability decisions, resource leases, cancellable tasks, typed results and no mutation of the underlying quantitative source. Public API must hide `wgpu::{Device, Texture, Queue}` except in clearly named expert host interop, which is deferred until lifetime/synchronisation semantics are evidenced.

## 4. Image representations and presentation semantics

1. Retain meaningful sample width, storage bits, signedness, endian-normalised decoded access, strides, frame identity and original values.
2. Apply the modality transform (rescale slope/intercept or applicable Modality LUT) **without irrevocably changing the canonical source**. Window/level does not belong in the DICOMflux decoder or input upload step.
3. Apply the applicable VOI LUT/window function according to DICOM PS3.3, including clear handling of `LINEAR`, `LINEAR_EXACT` and `SIGMOID` where admitted. Distinguish width/centre semantics from arbitrary exposure/contrast UI adjustments.
4. Apply photometric polarity, presentation LUT/shape and an explicit display/output transform. `MONOCHROME1` inversion is not a codec operation. Honor pixel padding in masks/histograms as specified for the operation.
5. Make output colour space, 8-/higher-bit encoding, alpha, calibration policy and presentation provenance explicit. Display pixels must never replace quantitative samples.

On FS-01, only the exact subset declared in the SRS is supported. VOI LUT tables, Presentation States, display GSDF and colour/HDR are future explicit profiles unless separately qualified; unknown modes fail with `UnsupportedPresentation`, not silent fallback.

## 5. Spatial geometry and regular-volume admission

Use **DICOM patient LPS** (millimetres) as the declared external convention, `f64` for authoritative transforms and physical measurement, and a derived rendering transform in `f32` with documented precision/coordinate rebasing. Index coordinates refer to voxel centres at integer indices; half-open bounds; fast X/column index in row-major data must be documented unambiguously.

DICOMflux is responsible for verifying functional-group transforms, orientation and position from the applicable Image Plane / Plane Position / Plane Orientation attributes. A regular 3D volume must have a validated index-to-patient affine and dimension/spacings/regularity tolerances. Slice Thickness is nominal thickness, **not** a replacement for interslice geometric position. Orientation, ordering and spacing must never come from filename sort or instance numbers alone. Reject unsupported irregular/mixed/multiphase/time/energy acquisitions or declare a separate explicit resampling operation later.

A `CheckedVolume` represents admitted, regular 3D image geometry; a `FrameSet` can remain distinct for future irregular data but is **not** silently promoted to a 3D texture. The first slice should validate a minimal synthetic volume descriptor, not assemble patient series.

## 6. Rendering architecture and operations

- **2D:** fragment shader samples a 2D texture or suitable storage/sampled representation; VOI settings changed as uniforms/bind groups, not CPU source reprocessing. CPU reference is mandatory for accepted greyscale modes. For integer textures, use explicit/manual interpolation or an accurately characterised conversion; do not assume filterability.
- **MPR:** later sample a checked regular 3D volume at patient-space slice planes with explicit interpolation and thickness; nearest-neighbour for label masks, appropriate interpolation for continuous intensities, and physical-coordinate crosshair alignment.
- **MIP/MinIP:** later use direction-aware ray/slab sampling, length/step limits and explicit source-scale intensity treatment; no hidden windowing before projection.
- **DVR:** later front-to-back compositing, bounded sampling, opacity/colour transfer function, clipping, quality controls and provenance. Begin with graphics/fragment ray marching; evaluate compute only after profiling.
- **Segmentation:** later basic threshold masks and overlays first; durable masks include geometry/identity/provenance and never masquerade as DICOM SEG without DICOMflux export/validation. Defer Marching Cubes and parry3d until an independently reviewed surface-extraction slice.
- **Surface extraction:** later WGSL compute with a bounded count/scan/emit method, ambiguity handling, memory limits, normals and mesh oracles; the ability to keep GPU vertex buffers resident is an optimisation, not a validation bypass.

## 7. GPU memory and capability policies

`TextureDimension::D3` is a preferred *resident representation for admitted volumes*, not the only volume store. Query `wgpu::Adapter` features/limits, texture usage/format capabilities, shader-format support and max dimensions before admission. For example 512×512×1024 samples require 512 MiB at 16 bits and 1 GiB at 32 bits, **before** staging, masks, buffers, intermediate render targets and in-flight frames. `R16Sint`/`R16Uint` textures do not grant automatic linear filtering; `R32Float` filtering is feature-dependent. A 3D texture allocation may fail on an otherwise useful machine.

Maintain separately accounted quotas for decoded retained CPU storage, upload staging, GPU resident textures, transient outputs, masks and in-flight operations. Use checked `u64`/`usize` arithmetic and total-budget admission before allocating. Buffer leases remain valid until GPU completion; cancellation must not release in-flight resources. Device loss, no adapter, OOM, unsupported format/features and submission errors are typed failure outcomes. A resource or GPU failure cannot imply a corrupted but reported-complete image.

FS-01 must work with a small bounded 2D image and may use a software GPU adapter for tests if explicitly labelled. No universal software DVR fallback is required. Later large volumes need documented brick/cache policy, LOD and streaming; do not prematurely build that in FS-01.

## 8. Platform qualification and host boundary

**Primary first-release targets:** Windows x86_64 and aarch64; Linux x86_64 and aarch64. Windows graphics policy: qualified D3D12 or Vulkan; Linux: qualified Vulkan. These are goals requiring actual machines/driver evidence. Android remains a primary target for DICOMflux/Flux Imaging Suite and a separately commissioned future GPU DICOMflux-3D profile; WebGPU/browser and macOS also require separate qualification. `raster-labs` is the GitHub home; do not assume public Rust repos yet exist.

For Rust hosts, offscreen texture/results are the first interface. For Swing, a bounded offscreen readback may be explored later, with copies and latency reported; native AWT/JAWT graphics integration is a separate release gate. `egui` is a development harness, not an API requirement. Host-specific code remains in adapters; no duplicate codec/DICOM bindings in UI code.

## 9. Design lessons from references

Voxelia Master Technical Architecture v0.1.1 supplies conceptual guidance: index vs patient space, immutable source, reference vs display values, scene snapshots, camera and presentation provenance, independent CPU validation, explicit resource limits. Do not port its general-purpose module hierarchy or Apple/Metal API. SolidCraft's `viewport`, `viewport-gpu`, `viewport-egui` separation and enforced dependency checks are useful patterns; its CAD B-rep engine and tessellation algorithm are not medical image renderers or segmentation oracles. Refer to the source register for pins/rights. Avoid wholesale adoption of any external engine.

## 10. Milestone roadmap with hard gates

| Stage | First new capability | Gate (evidence required) |
|---|---|---|
| FS-01 | Checked image/volume descriptors + CPU/GPU 2D greyscale W/L | Independent consumer, published oracles, two target-host build receipts, bounded errors |
| FS-02 | Regular-volume MPR and crosshair geometry | Physical-space/orientation phantoms; interpolation tolerances |
| FS-03 | MIP/MinIP, bounded DVR and transfer functions | Reproducible volume visual/quantitative oracles and GPU memory profiling |
| FS-04 | Threshold masks, segmentation overlays and source provenance | CPU/GPU masks, overlap/value handling, DICOM SEG handoff contract |
| Future | Bricked volume residency, surface extraction, richer segmentation/fusion | Separate approved work order/standards and safety review |

FS-02 onward are roadmap intentions, **not** part of the first-slice agent commission.
