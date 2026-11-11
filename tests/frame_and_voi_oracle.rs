// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::num::NonZeroU64;
use std::sync::Arc;

use dicomflux_3d::{
    BitPrecision, CancellationToken, ColorModel, DecodedFrameLease, DisplayEncoding,
    FrameDescriptor, IntensityTransform, LimitPoolKind, ModalityUnit, PhotometricPolarity,
    PixelPadding, PresentationPath, RenderError, ResourceLimits, ResourceTracker, SampleType,
    SourceIdentity, VoiFunction, VoiParameters, VoiRequest, VolumeDescriptor, VolumeDimensions,
    admit_frame, admit_volume, render_reference,
};
use sha2::{Digest, Sha256};

fn test_identity(id: u64, generation: u64) -> SourceIdentity {
    SourceIdentity::new(
        NonZeroU64::new(id).expect("non-zero test owner id"),
        generation,
    )
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

fn sha256_i16_le(words: &[i16]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for &w in words {
        hasher.update(w.to_le_bytes());
    }
    hasher.finalize().into()
}

fn sha256_u16_le(words: &[u16]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for &w in words {
        hasher.update(w.to_le_bytes());
    }
    hasher.finalize().into()
}

fn analytic_linear_voi_u8(
    raw_sample: i32,
    slope: f64,
    intercept: f64,
    center: f64,
    width: f64,
    polarity: PhotometricPolarity,
) -> u8 {
    let x = f64::from(raw_sample) * slope + intercept;
    let low = center - 0.5 - (width - 1.0) * 0.5;
    let high = center - 0.5 + (width - 1.0) * 0.5;
    let y = if x <= low {
        0.0
    } else if x > high {
        255.0
    } else {
        ((x - (center - 0.5)) / (width - 1.0) + 0.5) * 255.0
    };
    let y_pol = match polarity {
        PhotometricPolarity::Monochrome2 => y,
        PhotometricPolarity::Monochrome1 => 255.0 - y,
    };
    y_pol.clamp(0.0, 255.0).round_ties_even() as u8
}

#[test]
fn fs01_t02_signed_12bit_i16_sign_extension_and_immutability() {
    let tracker = ResourceTracker::with_default_limits();
    let cancel = CancellationToken::new();
    let samples: Vec<i16> = vec![-2048, -1024, -1, 0, 1, 2047];
    let arc_samples: Arc<[i16]> = Arc::from(samples.as_slice());
    let digest_before = sha256_i16_le(&arc_samples);

    let precision = BitPrecision::new(16, 12, 11);
    let desc = FrameDescriptor::new(3, 2, 6, SampleType::I16, precision);
    let lease = DecodedFrameLease::from_arc_i16(Arc::clone(&arc_samples), test_identity(1, 1))
        .expect("valid i16 Arc lease");

    let frame = admit_frame(desc, lease, &tracker).expect("valid 12-bit signed i16 frame");
    assert_eq!(frame.sample_i32(0, 0), -2048);
    assert_eq!(frame.sample_i32(1, 0), -1024);
    assert_eq!(frame.sample_i32(2, 0), -1);
    assert_eq!(frame.sample_i32(0, 1), 0);
    assert_eq!(frame.sample_i32(1, 1), 1);
    assert_eq!(frame.sample_i32(2, 1), 2047);

    let req_a = VoiRequest::linear(0.0, 4096.0);
    let img_a = render_reference(&frame, &req_a, &tracker, &cancel).expect("CPU render A");
    for (idx, &raw) in samples.iter().enumerate() {
        let expected = analytic_linear_voi_u8(
            i32::from(raw),
            1.0,
            0.0,
            0.0,
            4096.0,
            PhotometricPolarity::Monochrome2,
        );
        assert_eq!(img_a.pixels()[idx], expected);
    }

    // Re-render with different W/L and verify Vec<i16> + padded byte-stride equivalence.
    let req_b = VoiRequest::linear(-500.0, 1000.0);
    let img_b = render_reference(&frame, &req_b, &tracker, &cancel).expect("CPU render B");
    assert_eq!(sha256_i16_le(&arc_samples), digest_before);

    let vec_lease =
        DecodedFrameLease::from_vec_i16(samples.clone(), test_identity(2, 1)).expect("vec i16");
    let vec_frame = admit_frame(desc, vec_lease, &tracker).expect("admit vec i16");
    let img_vec = render_reference(&vec_frame, &req_b, &tracker, &cancel).expect("vec CPU render");
    assert_eq!(img_b.pixels(), img_vec.pixels());

    // Sub-sliced strided byte buffer with sentinel prefix/padding bytes; prove full byte immutability.
    let mut raw_bytes = vec![0xAA_u8, 0x55_u8]; // 2-byte byte_offset
    for row in samples.chunks_exact(3) {
        for &w in row {
            raw_bytes.extend_from_slice(&w.to_le_bytes());
        }
        raw_bytes.extend_from_slice(&[0xDE, 0xAD]); // 2 bytes row tail padding -> stride 8
    }
    let raw_arc = Arc::new(raw_bytes);
    let bytes_digest_before = sha256_bytes(&raw_arc);
    let mut strided_desc = FrameDescriptor::new(3, 2, 8, SampleType::I16, precision);
    strided_desc.byte_offset = 2;
    let strided_frame = admit_frame(
        strided_desc,
        DecodedFrameLease::from_arc_bytes(Arc::clone(&raw_arc), test_identity(3, 1)),
        &tracker,
    )
    .expect("strided byte frame");
    let img_strided =
        render_reference(&strided_frame, &req_b, &tracker, &cancel).expect("strided CPU render");
    assert_eq!(img_strided.pixels(), img_b.pixels());
    assert_eq!(sha256_bytes(&raw_arc), bytes_digest_before);

    // Corrupted upper sign bits in 12-bit signed i16 must fail closed with InvalidDescriptor:
    // 0x17FF has bit 11 == 0 (+2047) but bit 12 == 1; 0x0FFF has bit 11 == 1 (-1) but bits 12..15 == 0.
    for bad_word in [0x17FF_u16 as i16, 0x0FFF_u16 as i16] {
        let bad_lease = DecodedFrameLease::from_vec_i16(vec![bad_word], test_identity(4, 1))
            .expect("bad word lease");
        let bad_desc = FrameDescriptor::new(1, 1, 2, SampleType::I16, precision);
        let err = admit_frame(bad_desc, bad_lease, &tracker).expect_err("corrupt sign extension");
        assert!(matches!(err, RenderError::InvalidDescriptor(_)));
    }
}

#[test]
fn fs01_t03_unsigned_16bit_precision_and_highbit_rules() {
    let tracker = ResourceTracker::with_default_limits();
    let cancel = CancellationToken::new();

    // Canonical 12-bit unsigned in u16 (`bits_stored = 12`, `high_bit = 11`).
    let words_12 = Arc::new(vec![0_u16, 1, 2048, 4095]);
    let hash_12 = sha256_u16_le(&words_12);
    let desc_12 = FrameDescriptor::new(2, 2, 4, SampleType::U16, BitPrecision::new(16, 12, 11));
    let frame_12 = admit_frame(
        desc_12,
        DecodedFrameLease::from_arc_u16(Arc::clone(&words_12), test_identity(10, 1))
            .expect("u16 12-bit lease"),
        &tracker,
    )
    .expect("admit u16 12-bit");
    assert_eq!(frame_12.sample_i32(0, 0), 0);
    assert_eq!(frame_12.sample_i32(1, 1), 4095);
    let out_12 = render_reference(
        &frame_12,
        &VoiRequest::linear(2048.0, 4096.0),
        &tracker,
        &cancel,
    )
    .expect("render u16 12-bit");
    assert_eq!(out_12.pixels()[0], 0);
    assert_eq!(out_12.pixels()[3], 255);
    assert_eq!(sha256_u16_le(&words_12), hash_12);

    // Canonical full-range 16-bit unsigned (`bits_stored = 16`, `high_bit = 15`).
    let words_16 = Arc::new(vec![0_u16, 1, 32768, 65535]);
    let hash_16 = sha256_u16_le(&words_16);
    let desc_16 = FrameDescriptor::new(2, 2, 4, SampleType::U16, BitPrecision::new(16, 16, 15));
    let frame_16 = admit_frame(
        desc_16,
        DecodedFrameLease::from_arc_u16(Arc::clone(&words_16), test_identity(11, 1))
            .expect("u16 16-bit lease"),
        &tracker,
    )
    .expect("admit u16 16-bit");
    assert_eq!(frame_16.sample_i32(0, 1), 32768);
    assert_eq!(frame_16.sample_i32(1, 1), 65535);
    let out_16 = render_reference(
        &frame_16,
        &VoiRequest::linear(32768.0, 65536.0),
        &tracker,
        &cancel,
    )
    .expect("render u16 16-bit");
    assert_eq!(out_16.pixels()[0], 0);
    assert_eq!(out_16.pixels()[3], 255);
    assert_eq!(sha256_u16_le(&words_16), hash_16);

    // Non-canonical `high_bit = 15` when `bits_stored = 12` must fail with InvalidDescriptor.
    let bad_hb_desc = FrameDescriptor::new(2, 2, 4, SampleType::U16, BitPrecision::new(16, 12, 15));
    let bad_hb_err = admit_frame(
        bad_hb_desc,
        DecodedFrameLease::from_arc_u16(Arc::clone(&words_12), test_identity(12, 1)).unwrap(),
        &tracker,
    )
    .expect_err("non-canonical high_bit must fail");
    assert!(matches!(bad_hb_err, RenderError::InvalidDescriptor(_)));

    // Dirty upper bit `0x8001` in 12-bit u16 must fail with InvalidDescriptor.
    let dirty_lease = DecodedFrameLease::from_vec_u16(vec![0x8001_u16], test_identity(13, 1))
        .expect("dirty u16 lease");
    let dirty_desc = FrameDescriptor::new(1, 1, 2, SampleType::U16, BitPrecision::new(16, 12, 11));
    let dirty_err =
        admit_frame(dirty_desc, dirty_lease, &tracker).expect_err("dirty upper bit must fail");
    assert!(matches!(dirty_err, RenderError::InvalidDescriptor(_)));
}

#[test]
fn fs01_t04_dicom_linear_voi_endpoints_midpoint_width_one_and_rounding() {
    let tracker = ResourceTracker::with_default_limits();
    let cancel = CancellationToken::new();

    // 1. DICOM PS3.3 C.11.2.1.2.1 specimen: c = 50.0, w = 100.0 -> low = 0.0, high = 99.0.
    let ps33_samples: Vec<i16> = vec![-25, 0, 1, 49, 50, 98, 99, 150];
    let ps33_frame = admit_frame(
        FrameDescriptor::new(8, 1, 16, SampleType::I16, BitPrecision::new(16, 16, 15)),
        DecodedFrameLease::from_vec_i16(ps33_samples.clone(), test_identity(20, 1)).unwrap(),
        &tracker,
    )
    .unwrap();
    let ps33_img = render_reference(
        &ps33_frame,
        &VoiRequest::linear(50.0, 100.0),
        &tracker,
        &cancel,
    )
    .unwrap();
    assert_eq!(ps33_img.pixels()[0], 0); // -25 <= 0.0
    assert_eq!(ps33_img.pixels()[1], 0); // 0.0 <= 0.0 exact lower clamp endpoint
    assert_eq!(ps33_img.pixels()[6], 255); // 99.0 exact upper ramp endpoint
    assert_eq!(ps33_img.pixels()[7], 255); // 150 > 99.0
    for (idx, &s) in ps33_samples.iter().enumerate() {
        assert_eq!(
            ps33_img.pixels()[idx],
            analytic_linear_voi_u8(
                i32::from(s),
                1.0,
                0.0,
                50.0,
                100.0,
                PhotometricPolarity::Monochrome2,
            )
        );
    }

    // 2. Binary step `w = 1.0`: `x <= c - 0.5` -> 0, `x > c - 0.5` -> 255 (no division by zero).
    let w1_frame = admit_frame(
        FrameDescriptor::new(4, 1, 8, SampleType::I16, BitPrecision::new(16, 16, 15)),
        DecodedFrameLease::from_vec_i16(vec![98, 99, 100, 101], test_identity(21, 1)).unwrap(),
        &tracker,
    )
    .unwrap();
    let mut w1_req = VoiRequest::linear(50.0, 1.0); // threshold c - 0.5 = 49.5
    w1_req.rescale = IntensityTransform::LinearRescale {
        slope: 0.5,
        intercept: 0.0,
        unit: ModalityUnit::Unspecified,
    }; // x = [49.0, 49.5, 50.0, 50.5]
    let w1_img = render_reference(&w1_frame, &w1_req, &tracker, &cancel).unwrap();
    assert_eq!(w1_img.pixels(), &[0, 0, 255, 255]);

    // 3. Exact IEEE-754 half-way ties (`0.5 -> 0`, `1.5 -> 2`, `2.5 -> 2`, `127.5 -> 128`, `128.5 -> 128`).
    // Using c = 0.5 (`c - 0.5 = 0.0`), w = 2.0 (`w - 1.0 = 1.0`), s = 1, intercept = 0.0:
    // For target_voi in [127.5, 128.5, 252.5, 253.5, 254.5], u in [0.5, 1.0) satisfies Sterbenz
    // exactness `(u - 0.5) + 0.5 == u` and surjectively hits every f64 in [127.5, 255.0).
    let one_frame = admit_frame(
        FrameDescriptor::new(1, 1, 2, SampleType::I16, BitPrecision::new(16, 16, 15)),
        DecodedFrameLease::from_vec_i16(vec![1_i16], test_identity(22, 1)).unwrap(),
        &tracker,
    )
    .unwrap();

    let find_exact_u = |target_voi: f64| -> f64 {
        let base_bits = (target_voi / 255.0).to_bits();
        for delta in -4_i64..=4_i64 {
            let candidate_bits = if delta >= 0 {
                base_bits + (delta as u64)
            } else {
                base_bits - ((-delta) as u64)
            };
            let u = f64::from_bits(candidate_bits);
            let x = u - 0.5;
            let y_voi = (x + 0.5) * 255.0;
            if y_voi.to_bits() == target_voi.to_bits() {
                return x;
            }
        }
        panic!("expected exact IEEE-754 preimage for {target_voi}");
    };

    // Monochrome1 maps y_voi in [254.5, 253.5, 252.5, 127.5] to exact [0.5, 1.5, 2.5, 127.5].
    for (target_voi, expected_even_u8) in [
        (254.5_f64, 0_u8),   // 255.0 - 254.5 == 0.5 -> 0 (ties to even)
        (253.5_f64, 2_u8),   // 255.0 - 253.5 == 1.5 -> 2 (ties to even)
        (252.5_f64, 2_u8),   // 255.0 - 252.5 == 2.5 -> 2 (ties to even, vs 3 for round-away)
        (127.5_f64, 128_u8), // 255.0 - 127.5 == 127.5 -> 128 (ties to even)
    ] {
        let slope = find_exact_u(target_voi);
        let req = VoiRequest::new(
            VoiParameters::Linear {
                center: 0.5,
                width: 2.0,
            },
            IntensityTransform::LinearRescale {
                slope,
                intercept: 0.0,
                unit: ModalityUnit::Unspecified,
            },
            PhotometricPolarity::Monochrome1,
            0,
        );
        let rendered = render_reference(&one_frame, &req, &tracker, &cancel).unwrap();
        assert_eq!(rendered.pixels()[0], expected_even_u8);
    }

    // Monochrome2 on exact 127.5 -> 128 and 128.5 -> 128 (distinguishing ties-to-even from 129).
    for (target_voi, expected_even_u8) in [(127.5_f64, 128_u8), (128.5_f64, 128_u8)] {
        let slope = find_exact_u(target_voi);
        let req = VoiRequest::new(
            VoiParameters::Linear {
                center: 0.5,
                width: 2.0,
            },
            IntensityTransform::LinearRescale {
                slope,
                intercept: 0.0,
                unit: ModalityUnit::Unspecified,
            },
            PhotometricPolarity::Monochrome2,
            0,
        );
        let rendered = render_reference(&one_frame, &req, &tracker, &cancel).unwrap();
        assert_eq!(rendered.pixels()[0], expected_even_u8);
        assert_eq!(rendered.receipt().rounding_policy, "round_ties_even_u8");
    }
}

#[test]
fn fs01_t05_rescale_slope_intercept_ct_hu_and_negative_slope() {
    let tracker = ResourceTracker::with_default_limits();
    let cancel = CancellationToken::new();

    // CT raw stored values [0, 1024, 1064, 2047] with slope = 1.0, intercept = -1024.0 -> HU [-1024, 0, 40, 1023].
    let raw_words = Arc::new(vec![0_i16, 1024, 1064, 2047]);
    let digest_before = sha256_i16_le(&raw_words);
    let frame = admit_frame(
        FrameDescriptor::new(4, 1, 8, SampleType::I16, BitPrecision::new(16, 12, 11)),
        DecodedFrameLease::from_arc_i16(Arc::clone(&raw_words), test_identity(30, 1)).unwrap(),
        &tracker,
    )
    .unwrap();

    // Soft-tissue HU window: center = 40.0, width = 400.0.
    let mut ct_req = VoiRequest::linear(40.0, 400.0);
    ct_req.rescale = IntensityTransform::LinearRescale {
        slope: 1.0,
        intercept: -1024.0,
        unit: ModalityUnit::HounsfieldUnit,
    };
    let ct_img = render_reference(&frame, &ct_req, &tracker, &cancel).unwrap();
    for (idx, &s) in raw_words.iter().enumerate() {
        assert_eq!(
            ct_img.pixels()[idx],
            analytic_linear_voi_u8(
                i32::from(s),
                1.0,
                -1024.0,
                40.0,
                400.0,
                PhotometricPolarity::Monochrome2,
            )
        );
    }
    assert_eq!(ct_img.receipt().modality_unit, ModalityUnit::HounsfieldUnit);
    assert_eq!(ct_img.receipt().applied_slope, 1.0);
    assert_eq!(ct_img.receipt().applied_intercept, -1024.0);
    assert_eq!(ct_img.receipt().path, PresentationPath::CpuReference);
    assert_eq!(
        ct_img.descriptor().display_encoding,
        DisplayEncoding::UncalibratedSrgbU8
    );
    assert_eq!(ct_img.receipt().voi_function, VoiFunction::Linear);

    // Fractional positive slope (0.5), negative slope (-2.0), and zero slope (0.0).
    for (slope, intercept, center, width) in [
        (0.5_f64, 10.0_f64, 270.0_f64, 600.0_f64),
        (-2.0_f64, 500.0_f64, -1500.0_f64, 2500.0_f64),
        (0.0_f64, 40.0_f64, 40.0_f64, 100.0_f64),
    ] {
        let mut req = VoiRequest::linear(center, width);
        req.rescale = IntensityTransform::LinearRescale {
            slope,
            intercept,
            unit: ModalityUnit::OpticalDensity,
        };
        let img = render_reference(&frame, &req, &tracker, &cancel).unwrap();
        for (idx, &s) in raw_words.iter().enumerate() {
            assert_eq!(
                img.pixels()[idx],
                analytic_linear_voi_u8(
                    i32::from(s),
                    slope,
                    intercept,
                    center,
                    width,
                    PhotometricPolarity::Monochrome2,
                )
            );
        }
    }
    assert_eq!(sha256_i16_le(&raw_words), digest_before);
}

#[test]
fn fs01_t06_pixel_padding_value_and_range_before_rescale() {
    let tracker = ResourceTracker::with_default_limits();
    let cancel = CancellationToken::new();
    let precision_12 = BitPrecision::new(16, 12, 11);

    // 1. Single padding value (-2000) within 12-bit signed range.
    let single_words = vec![-2000_i16, -1024, 0, 1024];
    let mut single_desc = FrameDescriptor::new(4, 1, 8, SampleType::I16, precision_12);
    single_desc.pixel_padding = PixelPadding::Single(-2000);
    let single_frame = admit_frame(
        single_desc,
        DecodedFrameLease::from_vec_i16(single_words, test_identity(40, 1)).unwrap(),
        &tracker,
    )
    .expect("admit single padding frame");

    let mut req = VoiRequest::linear(-1000.0, 4000.0);
    req.rescale = IntensityTransform::LinearRescale {
        slope: 1.0,
        intercept: -1024.0,
        unit: ModalityUnit::HounsfieldUnit,
    };
    req.padding_display_code = 19;
    let out_m2 = render_reference(&single_frame, &req, &tracker, &cancel).unwrap();
    assert_eq!(out_m2.pixels()[0], 19);
    assert!(out_m2.receipt().padding_applied);
    assert_eq!(out_m2.receipt().padding_display_code, 19);

    // 2. Container-domain CT air padding sentinel `-3024` (outside 12-bit [-2048..2047] but valid
    // in 16-bit container when declared in PixelPadding::Range { start: -3024, end: -2000 }).
    let ct_pad_words = vec![-3024_i16, -2500, -2000, -1024, 0, 1064];

    // Without pixel_padding declaration, -3024 fails 12-bit upper-bit validation!
    let unmasked_desc = FrameDescriptor::new(6, 1, 12, SampleType::I16, precision_12);
    let unmasked_err = admit_frame(
        unmasked_desc,
        DecodedFrameLease::from_vec_i16(ct_pad_words.clone(), test_identity(41, 1)).unwrap(),
        &tracker,
    )
    .expect_err("-3024 without padding exemption must be rejected for 12-bit stored");
    assert!(matches!(unmasked_err, RenderError::InvalidDescriptor(_)));

    // With container-domain PixelPadding::Range, -3024..-2000 is admitted and masked across
    // both Monochrome2 and Monochrome1 and across wide/narrow W/L changes!
    let mut range_desc = FrameDescriptor::new(6, 1, 12, SampleType::I16, precision_12);
    range_desc.pixel_padding = PixelPadding::Range {
        start: -3024,
        end: -2000,
    };
    let range_frame = admit_frame(
        range_desc,
        DecodedFrameLease::from_vec_i16(ct_pad_words, test_identity(42, 1)).unwrap(),
        &tracker,
    )
    .expect("container-domain padding range must be admitted");

    for polarity in [
        PhotometricPolarity::Monochrome2,
        PhotometricPolarity::Monochrome1,
    ] {
        for (center, width) in [(40.0, 400.0), (-3500.0, 500.0), (0.0, 10000.0)] {
            let pad_req = VoiRequest::new(
                VoiParameters::Linear { center, width },
                IntensityTransform::LinearRescale {
                    slope: 1.0,
                    intercept: -1024.0,
                    unit: ModalityUnit::HounsfieldUnit,
                },
                polarity,
                7,
            );
            let rendered = render_reference(&range_frame, &pad_req, &tracker, &cancel).unwrap();
            assert_eq!(&rendered.pixels()[0..3], &[7, 7, 7]);
            assert_eq!(
                rendered.pixels()[3],
                analytic_linear_voi_u8(-1024, 1.0, -1024.0, center, width, polarity)
            );
            assert_eq!(
                rendered.pixels()[5],
                analytic_linear_voi_u8(1064, 1.0, -1024.0, center, width, polarity)
            );
        }
    }
}

#[test]
fn fs01_t07_monochrome1_vs_monochrome2_polarity() {
    let tracker = ResourceTracker::with_default_limits();
    let cancel = CancellationToken::new();
    let samples = Arc::new(vec![0_u16, 25, 50, 75, 100, 150]);
    let hash_before = sha256_u16_le(&samples);

    let frame = admit_frame(
        FrameDescriptor::new(6, 1, 12, SampleType::U16, BitPrecision::new(16, 16, 15)),
        DecodedFrameLease::from_arc_u16(Arc::clone(&samples), test_identity(50, 1)).unwrap(),
        &tracker,
    )
    .unwrap();

    let req_m2 = VoiRequest::linear(50.0, 100.0);
    let mut req_m1 = VoiRequest::linear(50.0, 100.0);
    req_m1.polarity = PhotometricPolarity::Monochrome1;

    let img_m2 = render_reference(&frame, &req_m2, &tracker, &cancel).unwrap();
    let img_m1 = render_reference(&frame, &req_m1, &tracker, &cancel).unwrap();

    assert_eq!(img_m2.pixels()[0], 0);
    assert_eq!(img_m1.pixels()[0], 255);
    assert_eq!(img_m2.pixels()[5], 255);
    assert_eq!(img_m1.pixels()[5], 0);
    assert_eq!(
        img_m2.descriptor().polarity,
        PhotometricPolarity::Monochrome2
    );
    assert_eq!(
        img_m1.descriptor().polarity,
        PhotometricPolarity::Monochrome1
    );
    assert_eq!(img_m1.receipt().polarity, PhotometricPolarity::Monochrome1);

    for (idx, &s) in samples.iter().enumerate() {
        assert_eq!(
            img_m1.pixels()[idx],
            analytic_linear_voi_u8(
                i32::from(s),
                1.0,
                0.0,
                50.0,
                100.0,
                PhotometricPolarity::Monochrome1,
            )
        );
    }
    assert_eq!(sha256_u16_le(&samples), hash_before);
}

#[test]
fn fs01_t08_invalid_descriptors_and_unsupported_profiles() {
    let tracker = ResourceTracker::with_default_limits();
    let cancel = CancellationToken::new();
    let prec16 = BitPrecision::new(16, 16, 15);
    let valid_lease = || DecodedFrameLease::from_vec_u8(vec![0_u8; 32], test_identity(60, 1));

    // Zero dimensions & zero frame_count -> InvalidDescriptor.
    assert!(matches!(
        admit_frame(
            FrameDescriptor::new(0, 4, 8, SampleType::U16, prec16),
            valid_lease(),
            &tracker
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));
    assert!(matches!(
        admit_frame(
            FrameDescriptor::new(4, 0, 8, SampleType::U16, prec16),
            valid_lease(),
            &tracker
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));
    let mut zero_frames = FrameDescriptor::new(2, 2, 4, SampleType::U16, prec16);
    zero_frames.frame_count = 0;
    assert!(matches!(
        admit_frame(zero_frames, valid_lease(), &tracker),
        Err(RenderError::InvalidDescriptor(_))
    ));

    // Odd byte_offset, odd row_stride_bytes, short row_stride_bytes, short backing slice.
    let mut odd_offset = FrameDescriptor::new(2, 2, 4, SampleType::U16, prec16);
    odd_offset.byte_offset = 1;
    assert!(matches!(
        admit_frame(odd_offset, valid_lease(), &tracker),
        Err(RenderError::InvalidDescriptor(_))
    ));
    assert!(matches!(
        admit_frame(
            FrameDescriptor::new(2, 2, 5, SampleType::U16, prec16),
            valid_lease(),
            &tracker
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));
    assert!(matches!(
        admit_frame(
            FrameDescriptor::new(2, 2, 2, SampleType::U16, prec16),
            valid_lease(),
            &tracker
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));
    assert!(matches!(
        admit_frame(
            FrameDescriptor::new(4, 4, 8, SampleType::U16, prec16),
            DecodedFrameLease::from_vec_u8(vec![0_u8; 30], test_identity(61, 1)),
            &tracker
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));

    // retained_capacity_bytes < logical_byte_len -> InvalidDescriptor across all 3 typed Arc constructors.
    assert!(matches!(
        DecodedFrameLease::from_arc_bytes_with_capacity(
            Arc::new(vec![0_u8; 8]),
            7,
            test_identity(62, 1)
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));
    assert!(matches!(
        DecodedFrameLease::from_arc_i16_with_capacity(
            Arc::new(vec![0_i16; 4]),
            7,
            test_identity(62, 2)
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));
    assert!(matches!(
        DecodedFrameLease::from_arc_u16_with_capacity(
            Arc::new(vec![0_u16; 4]),
            7,
            test_identity(62, 3)
        ),
        Err(RenderError::InvalidDescriptor(_))
    ));

    // Inverted or out-of-container PixelPadding -> InvalidDescriptor.
    for bad_pad in [
        PixelPadding::Range { start: 10, end: 5 },
        PixelPadding::Single(-40_000),
        PixelPadding::Single(40_000),
    ] {
        let mut d = FrameDescriptor::new(2, 2, 4, SampleType::I16, prec16);
        d.pixel_padding = bad_pad;
        assert!(matches!(
            admit_frame(d, valid_lease(), &tracker),
            Err(RenderError::InvalidDescriptor(_))
        ));
    }
    let mut u16_neg_pad = FrameDescriptor::new(2, 2, 4, SampleType::U16, prec16);
    u16_neg_pad.pixel_padding = PixelPadding::Single(-1);
    assert!(matches!(
        admit_frame(u16_neg_pad, valid_lease(), &tracker),
        Err(RenderError::InvalidDescriptor(_))
    ));

    // FS01-R18 unsupported frame profiles -> UnsupportedPixelFormat / UnsupportedPresentation.
    let mut multiframe = FrameDescriptor::new(2, 2, 4, SampleType::U16, prec16);
    multiframe.frame_count = 2;
    assert!(matches!(
        admit_frame(multiframe, valid_lease(), &tracker),
        Err(RenderError::UnsupportedPixelFormat { .. })
    ));

    for bad_sample_type in [SampleType::U8, SampleType::I32, SampleType::F32] {
        assert!(matches!(
            admit_frame(
                FrameDescriptor::new(2, 2, 4, bad_sample_type, prec16),
                valid_lease(),
                &tracker
            ),
            Err(RenderError::UnsupportedPixelFormat { .. })
        ));
    }

    for bad_prec in [
        BitPrecision::new(8, 8, 7),
        BitPrecision::new(16, 0, 0),
        BitPrecision::new(16, 17, 16),
    ] {
        assert!(matches!(
            admit_frame(
                FrameDescriptor::new(2, 2, 4, SampleType::U16, bad_prec),
                valid_lease(),
                &tracker
            ),
            Err(RenderError::UnsupportedPixelFormat { .. })
        ));
    }

    let mut rgb_components = FrameDescriptor::new(2, 2, 4, SampleType::U16, prec16);
    rgb_components.components = 3;
    assert!(matches!(
        admit_frame(rgb_components, valid_lease(), &tracker),
        Err(RenderError::UnsupportedPixelFormat { components: 3, .. })
    ));

    for bad_color in [ColorModel::PaletteIndexed, ColorModel::Color] {
        let mut color_desc = FrameDescriptor::new(2, 2, 4, SampleType::U16, prec16);
        color_desc.color_model = bad_color;
        assert!(matches!(
            admit_frame(color_desc, valid_lease(), &tracker),
            Err(RenderError::UnsupportedPresentation(_))
        ));
    }

    // Valid admitted frame used to probe invalid & unsupported VoiRequest cases.
    let ok_frame = admit_frame(
        FrameDescriptor::new(2, 2, 4, SampleType::I16, prec16),
        valid_lease(),
        &tracker,
    )
    .unwrap();

    for invalid_req in [
        VoiRequest::linear(50.0, 0.5),
        VoiRequest::linear(50.0, 0.0),
        VoiRequest::linear(50.0, -10.0),
        VoiRequest::linear(f64::NAN, 100.0),
        VoiRequest::linear(50.0, f64::INFINITY),
        VoiRequest::linear(f64::NEG_INFINITY, 100.0),
        VoiRequest::linear(1.5e308, 1.5e308),
        VoiRequest::new(
            VoiParameters::Linear {
                center: 0.0,
                width: 100.0,
            },
            IntensityTransform::LinearRescale {
                slope: f64::NAN,
                intercept: 0.0,
                unit: ModalityUnit::Unspecified,
            },
            PhotometricPolarity::Monochrome2,
            0,
        ),
        VoiRequest::new(
            VoiParameters::Linear {
                center: 0.0,
                width: 100.0,
            },
            IntensityTransform::LinearRescale {
                slope: 1.0e305,
                intercept: 0.0,
                unit: ModalityUnit::Unspecified,
            },
            PhotometricPolarity::Monochrome2,
            0,
        ),
    ] {
        assert!(matches!(
            render_reference(&ok_frame, &invalid_req, &tracker, &cancel),
            Err(RenderError::InvalidDescriptor(_))
        ));
    }

    for unsupported_req in [
        VoiRequest::new(
            VoiParameters::LinearExact {
                center: 50.0,
                width: 100.0,
            },
            IntensityTransform::Identity,
            PhotometricPolarity::Monochrome2,
            0,
        ),
        VoiRequest::new(
            VoiParameters::Sigmoid {
                center: 50.0,
                width: 100.0,
            },
            IntensityTransform::Identity,
            PhotometricPolarity::Monochrome2,
            0,
        ),
        VoiRequest::new(
            VoiParameters::UnsupportedLutSequence,
            IntensityTransform::Identity,
            PhotometricPolarity::Monochrome2,
            0,
        ),
        VoiRequest::new(
            VoiParameters::Linear {
                center: 50.0,
                width: 100.0,
            },
            IntensityTransform::UnsupportedModalityLutSequence,
            PhotometricPolarity::Monochrome2,
            0,
        ),
    ] {
        assert!(matches!(
            render_reference(&ok_frame, &unsupported_req, &tracker, &cancel),
            Err(RenderError::UnsupportedPresentation(_))
        ));
    }
}

#[test]
fn fs01_t09_resource_budgets_exact_boundary_and_overflow() {
    let prec16 = BitPrecision::new(16, 16, 15);
    let cancel = CancellationToken::new();

    // 1. Exact boundary on retained_source_bytes and cpu_output_bytes (`limit` succeeds, `limit + 1` fails).
    let mut limits = ResourceLimits::default_limits();
    limits.retained_source_bytes = 32; // exact 4x4 u16 frame = 32 bytes
    limits.cpu_output_bytes = 16; // exact 4x4 u8 output = 16 bytes
    limits.max_frame_dim = 4;
    limits.max_in_flight_jobs = 1;
    let tracker = ResourceTracker::new(limits);

    // 33 retained bytes -> fails with ResourceLimitExceeded(RetainedSource) before allocation/scan.
    let over_source = DecodedFrameLease::from_arc_bytes_with_capacity(
        Arc::new(vec![0_u8; 32]),
        33,
        test_identity(70, 1),
    )
    .unwrap();
    assert!(matches!(
        admit_frame(
            FrameDescriptor::new(4, 4, 8, SampleType::U16, prec16),
            over_source,
            &tracker
        ),
        Err(RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::RetainedSource,
            requested: 33,
            available: 32,
            limit: 32,
        })
    ));
    assert_eq!(tracker.snapshot().retained_source_bytes, 0);

    // Exact 32 retained bytes and 4x4 max_frame_dim -> succeeds!
    let exact_source =
        DecodedFrameLease::from_arc_bytes(Arc::new(vec![0_u8; 32]), test_identity(71, 1));
    let frame = admit_frame(
        FrameDescriptor::new(4, 4, 8, SampleType::U16, prec16),
        exact_source,
        &tracker,
    )
    .expect("exact retained_source_bytes limit must succeed");
    assert_eq!(tracker.snapshot().retained_source_bytes, 32);

    // Frame dimension `limit + 1` (`5 > 4`) -> fails with FrameDimension.
    assert!(matches!(
        admit_frame(
            FrameDescriptor::new(5, 1, 10, SampleType::U16, prec16),
            DecodedFrameLease::from_arc_bytes(Arc::new(vec![0_u8; 10]), test_identity(72, 1)),
            &tracker
        ),
        Err(RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::FrameDimension,
            requested: 5,
            limit: 4,
            ..
        })
    ));

    // Exact 16-byte CPU output succeeds; second render while first RenderedImage is alive exceeds
    // cpu_output_bytes by holding all 16 bytes -> fails deterministically!
    let img_1 = render_reference(&frame, &VoiRequest::linear(100.0, 200.0), &tracker, &cancel)
        .expect("exact cpu_output_bytes limit must succeed");
    assert_eq!(tracker.snapshot().cpu_output_bytes, 16);
    assert_eq!(tracker.snapshot().in_flight_jobs, 0);

    assert!(matches!(
        render_reference(&frame, &VoiRequest::linear(100.0, 200.0), &tracker, &cancel),
        Err(RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::CpuOutput,
            requested: 16,
            available: 0,
            limit: 16,
        })
    ));

    drop(img_1);
    assert_eq!(tracker.snapshot().cpu_output_bytes, 0);
    assert_eq!(tracker.snapshot().peak_cpu_output_bytes, 16);

    drop(frame);
    assert_eq!(tracker.snapshot().retained_source_bytes, 0);
    assert_eq!(tracker.snapshot().peak_retained_source_bytes, 32);

    // 2. u32::MAX frame dimensions fail deterministically without panic or OOM even when max_frame_dim = u32::MAX.
    let mut huge_dim_limits = ResourceLimits::default_limits();
    huge_dim_limits.max_frame_dim = u32::MAX;
    let huge_tracker = ResourceTracker::new(huge_dim_limits);
    let huge_err = admit_frame(
        FrameDescriptor::new(u32::MAX, u32::MAX, usize::MAX - 1, SampleType::U16, prec16),
        DecodedFrameLease::from_arc_bytes(Arc::new(vec![0_u8; 8]), test_identity(73, 1)),
        &huge_tracker,
    )
    .expect_err("u32::MAX dimensions must fail safely");
    assert!(matches!(huge_err, RenderError::InvalidDescriptor(_)));

    // 3. Volume exact byte limit (`limit` succeeds, `limit + 1` fails) and u32::MAX overflow check.
    let mut vol_limits = ResourceLimits::default_limits();
    vol_limits.max_volume_dim = 16;
    vol_limits.max_volume_bytes = 2 * 2 * 2 * 2; // 2x2x2 voxels * 2 bytes = 16 bytes
    let identity_affine = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let exact_vol = admit_volume(
        VolumeDescriptor::new(
            VolumeDimensions::new(2, 2, 2),
            SampleType::I16,
            prec16,
            identity_affine,
            test_identity(74, 1),
        ),
        &vol_limits,
    )
    .expect("exact max_volume_bytes limit must succeed");
    assert_eq!(exact_vol.byte_len(), 16);

    vol_limits.max_volume_bytes = 15; // 1 byte below required 16 bytes -> refuses!
    assert!(matches!(
        admit_volume(
            VolumeDescriptor::new(
                VolumeDimensions::new(2, 2, 2),
                SampleType::I16,
                prec16,
                identity_affine,
                test_identity(75, 1),
            ),
            &vol_limits,
        ),
        Err(RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::VolumeBytes,
            requested: 16,
            available: 15,
            limit: 15,
        })
    ));

    let mut huge_vol_limits = ResourceLimits::default_limits();
    huge_vol_limits.max_volume_dim = u32::MAX;
    let huge_vol_err = admit_volume(
        VolumeDescriptor::new(
            VolumeDimensions::new(u32::MAX, u32::MAX, u32::MAX),
            SampleType::I16,
            prec16,
            identity_affine,
            test_identity(76, 1),
        ),
        &huge_vol_limits,
    )
    .expect_err("u32::MAX volume dimensions must fail without panic");
    assert!(matches!(
        huge_vol_err,
        RenderError::InvalidDescriptor(_) | RenderError::ResourceLimitExceeded { .. }
    ));

    // 4. Pre-cancelled CancellationToken returns Cancelled with 0 leaked quota bytes.
    let cancel_tracker = ResourceTracker::with_default_limits();
    let small_frame = admit_frame(
        FrameDescriptor::new(2, 2, 4, SampleType::U16, prec16),
        DecodedFrameLease::from_vec_u8(vec![0_u8; 8], test_identity(77, 1)),
        &cancel_tracker,
    )
    .unwrap();
    let tripped = CancellationToken::new();
    tripped.cancel();
    assert!(matches!(
        render_reference(
            &small_frame,
            &VoiRequest::linear(50.0, 100.0),
            &cancel_tracker,
            &tripped
        ),
        Err(RenderError::Cancelled)
    ));
    assert_eq!(cancel_tracker.snapshot().cpu_output_bytes, 0);
    assert_eq!(cancel_tracker.snapshot().in_flight_jobs, 0);
}
