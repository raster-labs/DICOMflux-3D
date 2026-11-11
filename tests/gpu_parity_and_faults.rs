// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::num::NonZeroU64;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use dicomflux_3d::{
    AdapterSelectionPolicy, AllowedBackends, BitPrecision, CancellationToken, CheckedFrame,
    DecodedFrameLease, DisplayEncoding, FrameDescriptor, GpuContext, IntensityTransform,
    LimitPoolKind, ModalityUnit, PhotometricPolarity, PixelPadding, PresentationPath,
    QualificationTier, RenderError, ResourceLimits, ResourceTracker, SampleType, SourceIdentity,
    VoiFunction, VoiParameters, VoiRequest, admit_frame, probe_capabilities, render_offscreen,
    render_reference,
};

fn host_gpu_policy() -> AdapterSelectionPolicy {
    AdapterSelectionPolicy::new(AllowedBackends::all_supported(), true, true)
}

fn require_host_gpu(tracker: &ResourceTracker) -> GpuContext {
    let policy = host_gpu_policy();
    let report = probe_capabilities(&policy);
    assert!(
        report.cpu_reference_available,
        "CPU reference renderer must always report available"
    );
    assert!(
        !report.gpu_adapters.is_empty(),
        "FS01 GPU test suite requires at least one hardware or software adapter on host"
    );
    GpuContext::new(&policy, tracker).expect("GpuContext initialization must succeed on host")
}

fn assert_parity_by_category(
    frame: &CheckedFrame,
    request: &VoiRequest,
    cpu_pixels: &[u8],
    gpu_pixels: &[u8],
) {
    assert_eq!(cpu_pixels.len(), gpu_pixels.len());

    let (center, width, polarity) = match request.voi {
        VoiParameters::Linear { center, width } => (center, width, request.polarity),
        _ => panic!("expected Linear VOI in parity helper"),
    };
    let (slope, intercept) = match request.rescale {
        IntensityTransform::Identity => (1.0, 0.0),
        IntensityTransform::LinearRescale {
            slope, intercept, ..
        } => (slope, intercept),
        _ => panic!("expected linear/identity rescale in parity helper"),
    };

    let low = (center - 0.5) - (width - 1.0) * 0.5;
    let high = (center - 0.5) + (width - 1.0) * 0.5;
    let padding = frame.pixel_padding();

    for row in 0..frame.height() {
        for col in 0..frame.width() {
            let idx = (row as usize) * (frame.width() as usize) + (col as usize);
            let c_code = cpu_pixels[idx];
            let g_code = gpu_pixels[idx];
            let s = frame.sample_i32(col, row);

            if padding.matches(s) {
                assert_eq!(
                    c_code, request.padding_display_code,
                    "CPU pad mismatch at ({col},{row})"
                );
                assert_eq!(
                    g_code, c_code,
                    "0-LSB padding mismatch at ({col},{row}), s={s}, polarity={polarity:?}"
                );
                continue;
            }

            let x = f64::from(s) * slope + intercept;
            if slope == 0.0 || width == 1.0 || x <= low || x > high {
                assert_eq!(
                    g_code, c_code,
                    "0-LSB clamp/threshold mismatch at ({col},{row}), s={s}, x={x}, low={low}, high={high}"
                );
            } else {
                assert!(
                    g_code.abs_diff(c_code) <= 1,
                    "interior ramp drift > 1 LSB at ({col},{row}), s={s}: gpu={g_code}, cpu={c_code}"
                );
            }
        }
    }
}

#[test]
fn fs01_t12_cpu_vs_gpu_parity() {
    let tracker = ResourceTracker::with_default_limits();
    let ctx = require_host_gpu(&tracker);
    let cancel = CancellationToken::new();

    // 1. 4x4 signed 12-bit i16 with container-domain PixelPadding::Range [-3024..-2000] (64x row pitch padding).
    let raw_4x4: [i16; 16] = [
        -3024, -2500, -2000, -2048, -1024, -500, -1, 0, 1, 40, 200, 511, 1024, 1500, 2046, 2047,
    ];
    let mut raw_4x4_bytes = Vec::with_capacity(32);
    for sample in raw_4x4 {
        raw_4x4_bytes.extend_from_slice(&sample.to_le_bytes());
    }
    let sha_before_4x4 = Sha256::digest(&raw_4x4_bytes);
    let backing_4x4: Arc<[u8]> = Arc::from(raw_4x4_bytes.into_boxed_slice());
    let lease_4x4 = DecodedFrameLease::from_arc_bytes(
        Arc::clone(&backing_4x4),
        SourceIdentity {
            owner_id: NonZeroU64::new(1001).unwrap(),
            generation: 1,
        },
    );
    let mut desc_4x4 =
        FrameDescriptor::new(4, 4, 8, SampleType::I16, BitPrecision::new(16, 12, 11));
    desc_4x4.pixel_padding = PixelPadding::Range {
        start: -3024,
        end: -2000,
    };
    let frame_4x4 = admit_frame(desc_4x4, lease_4x4, &tracker).expect("admit 4x4 CT frame");

    for polarity in [
        PhotometricPolarity::Monochrome2,
        PhotometricPolarity::Monochrome1,
    ] {
        for (center, width, slope, intercept) in [
            (40.0, 400.0, 1.0, -1024.0),
            (0.0, 1.0, 1.0, -1024.0),
            (-500.0, 1200.0, -1.5, -200.0),
            (0.5, 50.0, 1.0e40, 0.0),
            (25.0, 100.0, 0.0, 25.0),
        ] {
            let req = VoiRequest::new(
                VoiParameters::Linear { center, width },
                IntensityTransform::LinearRescale {
                    slope,
                    intercept,
                    unit: ModalityUnit::HounsfieldUnit,
                },
                polarity,
                13,
            );
            let cpu = render_reference(&frame_4x4, &req, &tracker, &cancel).expect("cpu 4x4");
            let gpu = render_offscreen(&ctx, &frame_4x4, &req, &cancel).expect("gpu 4x4");
            assert_parity_by_category(&frame_4x4, &req, cpu.pixels(), gpu.pixels());
        }
    }
    assert_eq!(Sha256::digest(backing_4x4.as_ref()), sha_before_4x4);

    // 2. 63x37 odd-prime unaligned frame with non-zero byte_offset (16) and padded source row_stride_bytes (140).
    let src_stride = 140usize;
    let src_offset = 16usize;
    let mut backing_prime = vec![0x5Au8; src_offset + 37 * src_stride];
    for row in 0..37usize {
        for col in 0..63usize {
            let val = ((row * 63 + col) as i16 * 3) - 1800;
            let val_clamped = if col == 0 && row == 0 {
                -2000i16
            } else {
                val.clamp(-2048, 2047)
            };
            let pos = src_offset + row * src_stride + col * 2;
            backing_prime[pos..pos + 2].copy_from_slice(&val_clamped.to_le_bytes());
        }
    }
    let sha_before_prime = Sha256::digest(&backing_prime);
    let arc_prime: Arc<[u8]> = Arc::from(backing_prime.into_boxed_slice());
    let lease_prime = DecodedFrameLease::from_arc_bytes(
        Arc::clone(&arc_prime),
        SourceIdentity {
            owner_id: NonZeroU64::new(1002).unwrap(),
            generation: 1,
        },
    );
    let mut desc_prime = FrameDescriptor::new(
        63,
        37,
        src_stride,
        SampleType::I16,
        BitPrecision::new(16, 12, 11),
    );
    desc_prime.byte_offset = src_offset;
    desc_prime.pixel_padding = PixelPadding::Single(-2000);
    let frame_prime = admit_frame(desc_prime, lease_prime, &tracker).expect("admit 63x37 frame");

    let req_prime = VoiRequest::new(
        VoiParameters::Linear {
            center: 120.0,
            width: 650.0,
        },
        IntensityTransform::LinearRescale {
            slope: 0.75,
            intercept: -500.0,
            unit: ModalityUnit::HounsfieldUnit,
        },
        PhotometricPolarity::Monochrome1,
        240,
    );
    let cpu_prime = render_reference(&frame_prime, &req_prime, &tracker, &cancel).unwrap();
    let gpu_prime = render_offscreen(&ctx, &frame_prime, &req_prime, &cancel).unwrap();
    assert_parity_by_category(
        &frame_prime,
        &req_prime,
        cpu_prime.pixels(),
        gpu_prime.pixels(),
    );
    assert_eq!(Sha256::digest(arc_prime.as_ref()), sha_before_prime);

    // 3. Full 256x256 i16 sweep covering all 65,536 signed 16-bit values [-32768..=32767].
    let words_i16: Vec<i16> = (i16::MIN..=i16::MAX).collect();
    let lease_i16 = DecodedFrameLease::from_vec_i16(
        words_i16,
        SourceIdentity {
            owner_id: NonZeroU64::new(1003).unwrap(),
            generation: 1,
        },
    )
    .expect("256x256 i16 lease");
    let mut desc_i16 = FrameDescriptor::new(
        256,
        256,
        512,
        SampleType::I16,
        BitPrecision::new(16, 16, 15),
    );
    desc_i16.pixel_padding = PixelPadding::Range {
        start: -32_768,
        end: -32_000,
    };
    let frame_i16 = admit_frame(desc_i16, lease_i16, &tracker).expect("admit 256x256 i16");

    for req in [
        VoiRequest::new(
            VoiParameters::Linear {
                center: 40.0,
                width: 400.0,
            },
            IntensityTransform::LinearRescale {
                slope: 1.0,
                intercept: -1024.0,
                unit: ModalityUnit::HounsfieldUnit,
            },
            PhotometricPolarity::Monochrome2,
            7,
        ),
        VoiRequest::new(
            VoiParameters::Linear {
                center: -250.0,
                width: 1.0,
            },
            IntensityTransform::LinearRescale {
                slope: -0.5,
                intercept: 100.0,
                unit: ModalityUnit::Other,
            },
            PhotometricPolarity::Monochrome1,
            250,
        ),
    ] {
        let cpu_full = render_reference(&frame_i16, &req, &tracker, &cancel).unwrap();
        let gpu_full = render_offscreen(&ctx, &frame_i16, &req, &cancel).unwrap();
        assert_parity_by_category(&frame_i16, &req, cpu_full.pixels(), gpu_full.pixels());
    }

    // 4. Full 256x256 u16 sweep covering all 65,536 unsigned 16-bit values [0..=65535].
    let words_u16: Vec<u16> = (u16::MIN..=u16::MAX).collect();
    let lease_u16 = DecodedFrameLease::from_vec_u16(
        words_u16,
        SourceIdentity {
            owner_id: NonZeroU64::new(1004).unwrap(),
            generation: 1,
        },
    )
    .expect("256x256 u16 lease");
    let mut desc_u16 = FrameDescriptor::new(
        256,
        256,
        512,
        SampleType::U16,
        BitPrecision::new(16, 16, 15),
    );
    desc_u16.pixel_padding = PixelPadding::Single(0);
    let frame_u16 = admit_frame(desc_u16, lease_u16, &tracker).expect("admit 256x256 u16");

    for req in [
        VoiRequest::new(
            VoiParameters::Linear {
                center: 32_768.0,
                width: 16_384.0,
            },
            IntensityTransform::Identity,
            PhotometricPolarity::Monochrome2,
            0,
        ),
        VoiRequest::new(
            VoiParameters::Linear {
                center: 12_345.0,
                width: 1.0,
            },
            IntensityTransform::LinearRescale {
                slope: 1.25,
                intercept: -200.0,
                unit: ModalityUnit::OpticalDensity,
            },
            PhotometricPolarity::Monochrome1,
            99,
        ),
    ] {
        let cpu_u16 = render_reference(&frame_u16, &req, &tracker, &cancel).unwrap();
        let gpu_u16 = render_offscreen(&ctx, &frame_u16, &req, &cancel).unwrap();
        assert_parity_by_category(&frame_u16, &req, cpu_u16.pixels(), gpu_u16.pixels());
    }
}

#[test]
fn fs01_t13_gpu_capability_and_adapter_refusals() {
    // 1. Disallowing all backends succeeds in probe_capabilities and refuses GpuContext::new cleanly.
    let none_policy = AdapterSelectionPolicy::new(AllowedBackends::none(), false, true);
    let none_report = probe_capabilities(&none_policy);
    assert!(none_report.cpu_reference_available);
    assert!(none_report.gpu_adapters.is_empty());
    assert!(!none_report.excluded_profiles.is_empty());

    let default_tracker = ResourceTracker::with_default_limits();
    let no_adapter_err = GpuContext::new(&none_policy, &default_tracker)
        .expect_err("AllowedBackends::none() must fail with NoSuitableAdapter");
    assert_eq!(no_adapter_err, RenderError::NoSuitableAdapter);

    // 2. Host capability report completeness & strict software-adapter policy check.
    let strict_hw_policy = AdapterSelectionPolicy::default();
    let hw_report = probe_capabilities(&strict_hw_policy);
    assert!(hw_report.cpu_reference_available);
    assert!(
        hw_report
            .gpu_adapters
            .iter()
            .all(|a| a.supports_16bit_grayscale_offscreen)
    );
    if cfg!(target_os = "macos") {
        assert_eq!(
            hw_report.host_qualification_tier,
            QualificationTier::BoundedDevelopmentHost
        );
    }

    // 3. Exact GPU pool boundary refusals (limit succeeds, limit - 1 fails) for 4x4 frame:
    //    upload_bytes = 256 * 4 = 1024; source_resident = 32; target_resident = 16; readback = 256 * 4 = 1024.
    let make_4x4_on = |trk: &ResourceTracker| -> CheckedFrame {
        let mut words = Vec::with_capacity(16);
        words.extend(0..16i16);
        let lease = DecodedFrameLease::from_vec_i16(
            words,
            SourceIdentity {
                owner_id: NonZeroU64::new(2001).unwrap(),
                generation: 1,
            },
        )
        .unwrap();
        let desc = FrameDescriptor::new(4, 4, 8, SampleType::I16, BitPrecision::new(16, 12, 11));
        admit_frame(desc, lease, trk).unwrap()
    };
    let req = VoiRequest::linear(8.0, 16.0);
    let cancel = CancellationToken::new();

    // 3a. GpuStagingUpload exact boundary (1024 succeeds, 1023 refuses).
    let mut limits_up_ok = ResourceLimits::default_limits();
    limits_up_ok.gpu_staging_upload_bytes = 1024;
    let trk_up_ok = ResourceTracker::new(limits_up_ok);
    let ctx_up_ok = require_host_gpu(&trk_up_ok);
    let frm_up_ok = make_4x4_on(&trk_up_ok);
    assert!(render_offscreen(&ctx_up_ok, &frm_up_ok, &req, &cancel).is_ok());

    let mut limits_up_fail = ResourceLimits::default_limits();
    limits_up_fail.gpu_staging_upload_bytes = 1023;
    let trk_up_fail = ResourceTracker::new(limits_up_fail);
    let ctx_up_fail = require_host_gpu(&trk_up_fail);
    let frm_up_fail = make_4x4_on(&trk_up_fail);
    assert_eq!(
        render_offscreen(&ctx_up_fail, &frm_up_fail, &req, &cancel).unwrap_err(),
        RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::GpuStagingUpload,
            requested: 1024,
            available: 1023,
            limit: 1023,
        }
    );

    // 3b. GpuResident exact boundary (32 source + 16 target = 48 succeeds, 47 refuses).
    let mut limits_res_ok = ResourceLimits::default_limits();
    limits_res_ok.gpu_resident_bytes = 48;
    let trk_res_ok = ResourceTracker::new(limits_res_ok);
    let ctx_res_ok = require_host_gpu(&trk_res_ok);
    let frm_res_ok = make_4x4_on(&trk_res_ok);
    assert!(render_offscreen(&ctx_res_ok, &frm_res_ok, &req, &cancel).is_ok());

    let mut limits_res_fail = ResourceLimits::default_limits();
    limits_res_fail.gpu_resident_bytes = 47;
    let trk_res_fail = ResourceTracker::new(limits_res_fail);
    let ctx_res_fail = require_host_gpu(&trk_res_fail);
    let frm_res_fail = make_4x4_on(&trk_res_fail);
    assert!(matches!(
        render_offscreen(&ctx_res_fail, &frm_res_fail, &req, &cancel).unwrap_err(),
        RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::GpuResident,
            ..
        }
    ));

    // 3c. GpuStagingReadback exact boundary (1024 succeeds, 1023 refuses).
    let mut limits_rb_ok = ResourceLimits::default_limits();
    limits_rb_ok.gpu_staging_readback_bytes = 1024;
    let trk_rb_ok = ResourceTracker::new(limits_rb_ok);
    let ctx_rb_ok = require_host_gpu(&trk_rb_ok);
    let frm_rb_ok = make_4x4_on(&trk_rb_ok);
    assert!(render_offscreen(&ctx_rb_ok, &frm_rb_ok, &req, &cancel).is_ok());

    let mut limits_rb_fail = ResourceLimits::default_limits();
    limits_rb_fail.gpu_staging_readback_bytes = 1023;
    let trk_rb_fail = ResourceTracker::new(limits_rb_fail);
    let ctx_rb_fail = require_host_gpu(&trk_rb_fail);
    let frm_rb_fail = make_4x4_on(&trk_rb_fail);
    assert_eq!(
        render_offscreen(&ctx_rb_fail, &frm_rb_fail, &req, &cancel).unwrap_err(),
        RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::GpuStagingReadback,
            requested: 1024,
            available: 1023,
            limit: 1023,
        }
    );

    // 3d. InFlightJobs == 0 refuses before allocation.
    let mut limits_jobs_fail = ResourceLimits::default_limits();
    limits_jobs_fail.max_in_flight_jobs = 0;
    let trk_jobs_fail = ResourceTracker::new(limits_jobs_fail);
    let ctx_jobs_fail = require_host_gpu(&trk_jobs_fail);
    let frm_jobs_fail = make_4x4_on(&trk_jobs_fail);
    assert_eq!(
        render_offscreen(&ctx_jobs_fail, &frm_jobs_fail, &req, &cancel).unwrap_err(),
        RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::InFlightJobs,
            requested: 1,
            available: 0,
            limit: 0,
        }
    );
}

#[test]
fn fs01_t14_repeated_wl_residency_subslice_and_retire_before_replace() {
    // Tight resident limit holding AT MOST one 512x512 R16Sint source (524,288 B) + one 512x512 R8Uint target (262,144 B).
    let mut limits = ResourceLimits::default_limits();
    limits.gpu_resident_bytes = 512 * 512 * 2 + 512 * 512;
    let tracker = ResourceTracker::new(limits);
    let ctx = require_host_gpu(&tracker);
    let cancel = CancellationToken::new();

    let words_a: Vec<i16> = (0..(512 * 512))
        .map(|i| ((i % 3000) as i16) - 1000)
        .collect();
    let lease_a = DecodedFrameLease::from_vec_i16(
        words_a,
        SourceIdentity {
            owner_id: NonZeroU64::new(3001).unwrap(),
            generation: 1,
        },
    )
    .unwrap();
    let mut desc_a = FrameDescriptor::new(
        512,
        512,
        1024,
        SampleType::I16,
        BitPrecision::new(16, 12, 11),
    );
    desc_a.pixel_padding = PixelPadding::Single(-1000);
    let frame_a = admit_frame(desc_a, lease_a, &tracker).unwrap();

    // Cold render 1 on Frame A.
    let req_1 = VoiRequest::linear(40.0, 400.0);
    let img_1 = render_offscreen(&ctx, &frame_a, &req_1, &cancel).expect("cold render A");
    assert!(!img_1.receipt().upload.source_cache_hit);
    assert_eq!(img_1.receipt().upload.source_bytes_uploaded, 512 * 512 * 2);

    // Warm render 2 & 3 on Frame A with distinct W/L, polarity, and padding code -> 0 bytes uploaded!
    let req_2 = VoiRequest::new(
        VoiParameters::Linear {
            center: -600.0,
            width: 1500.0,
        },
        IntensityTransform::Identity,
        PhotometricPolarity::Monochrome2,
        42,
    );
    let img_2 = render_offscreen(&ctx, &frame_a, &req_2, &cancel).expect("warm render A #2");
    assert!(img_2.receipt().upload.source_cache_hit);
    assert_eq!(img_2.receipt().upload.source_bytes_uploaded, 0);
    assert_eq!(img_2.receipt().upload.staging_upload_bytes_reserved, 0);
    let cpu_2 = render_reference(&frame_a, &req_2, &tracker, &cancel).unwrap();
    assert_parity_by_category(&frame_a, &req_2, cpu_2.pixels(), img_2.pixels());

    let req_3 = VoiRequest::new(
        VoiParameters::Linear {
            center: 250.0,
            width: 80.0,
        },
        IntensityTransform::Identity,
        PhotometricPolarity::Monochrome1,
        9,
    );
    let img_3 = render_offscreen(&ctx, &frame_a, &req_3, &cancel).expect("warm render A #3");
    assert!(img_3.receipt().upload.source_cache_hit);
    assert_eq!(img_3.receipt().upload.source_bytes_uploaded, 0);

    // Retire-before-replace: Frame B (512x512) must evict Frame A BEFORE reserving GpuResident bytes.
    let words_b: Vec<i16> = (0..(512 * 512))
        .map(|i| ((i * 7 % 2000) as i16) - 500)
        .collect();
    let lease_b = DecodedFrameLease::from_vec_i16(
        words_b,
        SourceIdentity {
            owner_id: NonZeroU64::new(3002).unwrap(),
            generation: 1,
        },
    )
    .unwrap();
    let desc_b = FrameDescriptor::new(
        512,
        512,
        1024,
        SampleType::I16,
        BitPrecision::new(16, 12, 11),
    );
    let frame_b = admit_frame(desc_b, lease_b, &tracker).unwrap();

    let img_b = render_offscreen(&ctx, &frame_b, &req_1, &cancel)
        .expect("retire-before-replace on tight single-frame GpuResident budget must succeed");
    assert!(!img_b.receipt().upload.source_cache_hit);
    assert_eq!(img_b.receipt().upload.source_bytes_uploaded, 512 * 512 * 2);

    // Sub-slice non-collision on shared DecodedFrameLease (distinct byte_offset values).
    let mut shared_words = Vec::with_capacity(16 * 32);
    shared_words.extend((0..(16 * 16)).map(|_| 100i16));
    shared_words.extend((0..(16 * 16)).map(|_| 900i16));
    let shared_arc: Arc<[i16]> = Arc::from(shared_words.into_boxed_slice());
    let shared_id = SourceIdentity {
        owner_id: NonZeroU64::new(3003).unwrap(),
        generation: 1,
    };
    let lease_sub0 = DecodedFrameLease::from_arc_i16(Arc::clone(&shared_arc), shared_id).unwrap();
    let lease_sub1 = DecodedFrameLease::from_arc_i16(Arc::clone(&shared_arc), shared_id).unwrap();

    let desc_sub0 =
        FrameDescriptor::new(16, 16, 32, SampleType::I16, BitPrecision::new(16, 12, 11));
    let mut desc_sub1 = desc_sub0;
    desc_sub1.byte_offset = 16 * 16 * 2;

    let frame_sub0 = admit_frame(desc_sub0, lease_sub0, &tracker).unwrap();
    let frame_sub1 = admit_frame(desc_sub1, lease_sub1, &tracker).unwrap();

    let req_sub = VoiRequest::linear(500.0, 1000.0);
    let out_sub0 = render_offscreen(&ctx, &frame_sub0, &req_sub, &cancel).unwrap();
    let out_sub1 = render_offscreen(&ctx, &frame_sub1, &req_sub, &cancel).unwrap();
    assert!(!out_sub1.receipt().upload.source_cache_hit);
    assert_ne!(out_sub0.pixels()[0], out_sub1.pixels()[0]);

    // Multi-thread concurrent &GpuContext sharing across 4 worker threads on 512x512 frame_b.
    let concurrent_tracker = ResourceTracker::new(ResourceLimits::default());
    let concurrent_ctx = require_host_gpu(&concurrent_tracker);
    std::thread::scope(|s| {
        let mut handles = Vec::new();
        for t in 0..4u32 {
            let ctx_ref = &concurrent_ctx;
            let trk_ref = &concurrent_tracker;
            let frm_ref = &frame_b;
            let c_tok = cancel.clone();
            handles.push(s.spawn(move || {
                let r = VoiRequest::linear(f64::from(t) * 50.0, 300.0);
                let out = render_offscreen(ctx_ref, frm_ref, &r, &c_tok).expect("concurrent gpu");
                let expected =
                    render_reference(frm_ref, &r, trk_ref, &c_tok).expect("concurrent cpu");
                assert_parity_by_category(frm_ref, &r, expected.pixels(), out.pixels());
                out.receipt().upload.source_cache_hit
            }));
        }
        let mut hits = 0usize;
        let mut misses = 0usize;
        for h in handles {
            if h.join().expect("worker thread panicked") {
                hits += 1;
            } else {
                misses += 1;
            }
        }
        assert_eq!(misses, 1, "first thread to submit uploads frame_b once");
        assert_eq!(hits, 3, "other 3 threads reuse resident frame_b");
    });
    concurrent_ctx.evict_resident_frame();
    drop(concurrent_ctx);
    let concurrent_end = concurrent_tracker.snapshot();
    assert_eq!(concurrent_end.gpu_resident_bytes, 0);
    assert_eq!(concurrent_end.gpu_staging_upload_bytes, 0);
    assert_eq!(concurrent_end.gpu_staging_readback_bytes, 0);
    assert_eq!(concurrent_end.cpu_output_bytes, 0);
    assert_eq!(concurrent_end.in_flight_jobs, 0);

    drop(img_1);
    drop(img_2);
    drop(img_3);
    drop(img_b);
    drop(cpu_2);
    drop(out_sub0);
    drop(out_sub1);
    drop(frame_a);
    drop(frame_b);
    drop(frame_sub0);
    drop(frame_sub1);

    ctx.evict_resident_frame();
    drop(ctx);

    let final_snap = tracker.snapshot();
    assert_eq!(final_snap.retained_source_bytes, 0);
    assert_eq!(final_snap.gpu_staging_upload_bytes, 0);
    assert_eq!(final_snap.gpu_resident_bytes, 0);
    assert_eq!(final_snap.gpu_staging_readback_bytes, 0);
    assert_eq!(final_snap.cpu_output_bytes, 0);
    assert_eq!(final_snap.in_flight_jobs, 0);
}

#[test]
fn fs01_t15_public_cancellation_and_context_reinit() {
    let tracker = ResourceTracker::with_default_limits();
    let policy = host_gpu_policy();

    {
        let ctx1 = GpuContext::new(&policy, &tracker).expect("first GpuContext::new");
        let words = vec![250i16; 32 * 32];
        let lease = DecodedFrameLease::from_vec_i16(
            words,
            SourceIdentity {
                owner_id: NonZeroU64::new(4001).unwrap(),
                generation: 1,
            },
        )
        .unwrap();
        let desc = FrameDescriptor::new(32, 32, 64, SampleType::I16, BitPrecision::new(16, 12, 11));
        let frame = admit_frame(desc, lease, &tracker).unwrap();

        let pre_cancelled = CancellationToken::new();
        pre_cancelled.cancel();

        let err = render_offscreen(
            &ctx1,
            &frame,
            &VoiRequest::linear(100.0, 200.0),
            &pre_cancelled,
        )
        .expect_err("pre-cancelled token must fail immediately");
        assert_eq!(err, RenderError::Cancelled);

        let snap = tracker.snapshot();
        assert_eq!(snap.gpu_resident_bytes, 0);
        assert_eq!(snap.gpu_staging_upload_bytes, 0);
        assert_eq!(snap.gpu_staging_readback_bytes, 0);
        assert_eq!(snap.cpu_output_bytes, 0);
        assert_eq!(snap.in_flight_jobs, 0);
    }

    // Clean re-initialization after dropping prior context.
    let ctx2 = GpuContext::new(&policy, &tracker).expect("second GpuContext::new");
    assert!(ctx2.is_healthy());
    let words2 = vec![500u16; 16 * 16];
    let lease2 = DecodedFrameLease::from_vec_u16(
        words2,
        SourceIdentity {
            owner_id: NonZeroU64::new(4002).unwrap(),
            generation: 2,
        },
    )
    .unwrap();
    let desc2 = FrameDescriptor::new(16, 16, 32, SampleType::U16, BitPrecision::new(16, 16, 15));
    let frame2 = admit_frame(desc2, lease2, &tracker).unwrap();
    let ok_img = render_offscreen(
        &ctx2,
        &frame2,
        &VoiRequest::linear(500.0, 100.0),
        &CancellationToken::new(),
    )
    .expect("re-initialized context renders cleanly");
    assert_eq!(ok_img.pixels().len(), 256);
}

#[test]
fn fs01_t17_offscreen_output_descriptor_and_receipt_provenance() {
    let tracker = ResourceTracker::with_default_limits();
    let ctx = require_host_gpu(&tracker);
    let cancel = CancellationToken::new();

    let words: Vec<u16> = (0..(63 * 37) as u16).collect();
    let source_id = SourceIdentity {
        owner_id: NonZeroU64::new(5001).unwrap(),
        generation: 9,
    };
    let lease = DecodedFrameLease::from_vec_u16(words, source_id).unwrap();
    let mut desc =
        FrameDescriptor::new(63, 37, 126, SampleType::U16, BitPrecision::new(16, 12, 11));
    desc.pixel_padding = PixelPadding::Single(0);
    let frame = admit_frame(desc, lease, &tracker).unwrap();

    let req = VoiRequest::new(
        VoiParameters::Linear {
            center: 1000.0,
            width: 2000.0,
        },
        IntensityTransform::LinearRescale {
            slope: 1.5,
            intercept: -100.0,
            unit: ModalityUnit::HounsfieldUnit,
        },
        PhotometricPolarity::Monochrome2,
        5,
    );

    let img_1 = render_offscreen(&ctx, &frame, &req, &cancel).expect("first offscreen render");
    let img_2 = render_offscreen(&ctx, &frame, &req, &cancel).expect("second offscreen render");

    let out_desc = img_1.descriptor();
    assert_eq!(out_desc.width, 63);
    assert_eq!(out_desc.height, 37);
    assert_eq!(out_desc.row_stride_bytes, 63);
    assert_eq!(out_desc.bits_per_pixel, 8);
    assert_eq!(out_desc.polarity, PhotometricPolarity::Monochrome2);
    assert_eq!(
        out_desc.display_encoding,
        DisplayEncoding::UncalibratedSrgbU8
    );
    assert_eq!(img_1.pixels().len(), 63 * 37);

    let wgsl_bytes = include_bytes!("../src/rendering/gpu/voi_linear.wgsl");
    let expected_digest = Sha256::digest(wgsl_bytes);
    let mut expected_hex = String::with_capacity(64);
    for b in expected_digest {
        use std::fmt::Write as _;
        let _ = write!(&mut expected_hex, "{b:02x}");
    }

    let r1 = img_1.receipt();
    let r2 = img_2.receipt();
    assert!(r1.operation_id < r2.operation_id);
    assert_eq!(r1.source_identity, source_id);
    assert_eq!(r1.path, PresentationPath::GpuOffscreen);
    assert_eq!(r1.backend, Some(ctx.capabilities().backend));
    assert_eq!(r1.qualification_tier, ctx.capabilities().qualification_tier);
    if cfg!(target_os = "macos") {
        assert_eq!(
            r1.qualification_tier,
            QualificationTier::BoundedDevelopmentHost
        );
    }
    let shader_sha = r1
        .shader_sha256
        .as_deref()
        .expect("GPU receipt must record shader_sha256");
    assert_eq!(shader_sha.len(), 64);
    assert_eq!(shader_sha, expected_hex);
    assert_eq!(r1.voi_function, VoiFunction::Linear);
    assert_eq!(r1.applied_center, 1000.0);
    assert_eq!(r1.applied_width, 2000.0);
    assert_eq!(r1.applied_slope, 1.5);
    assert_eq!(r1.applied_intercept, -100.0);
    assert_eq!(r1.modality_unit, ModalityUnit::HounsfieldUnit);
    assert!(r1.padding_applied);
    assert_eq!(r1.padding_display_code, 5);
    assert_eq!(r1.rounding_policy, "round_ties_even_u8");
    assert_eq!(r1.max_gpu_vs_cpu_interior_lsb_tolerance, 1);
    assert!(r1.timings.total >= r1.timings.gpu_wait_and_unpack);

    let ctx_dbg = format!("{ctx:?}");
    let caps_dbg = format!("{:?}", ctx.capabilities());
    let img_dbg = format!("{img_1:?}");
    assert!(ctx_dbg.contains("<redacted>"));
    assert!(caps_dbg.contains("<redacted>"));
    assert!(img_dbg.contains("<redacted>"));
}
