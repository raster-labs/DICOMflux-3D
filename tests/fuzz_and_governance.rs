// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

//! Seeded adversarial fuzzing (`FS01-T16`), dependency layering & SPDX licence governance
//! (`FS01-T18`), privacy redaction (`FS01-T23`), and README scope/non-clinical language (`FS01-T24`).

use std::collections::BTreeSet;
use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use dicomflux_3d::{
    AdapterSelectionPolicy, AllowedBackends, BitPrecision, CancellationToken, ColorModel,
    DecodedFrameLease, FrameDescriptor, GpuContext, IntensityTransform, LimitPoolKind, LpsPointMm,
    ModalityUnit, OperationId, PatientPlane, PhotometricPolarity, PixelIndex2D, PixelPadding,
    RenderError, ResourceLimits, ResourceTracker, SampleType, SourceIdentity, VoiParameters,
    VoiRequest, VolumeDescriptor, VolumeDimensions, admit_frame, admit_volume, probe_capabilities,
    render_offscreen, render_reference,
};

const SPLITMIX64_SEED: u64 = 0xDF3D_F501_2026_1110;
const FUZZ_ITERATIONS: usize = 100_000;

const ALLOWED_SPDX_LICENCES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "0BSD",
    "ISC",
    "Zlib",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "CC0-1.0",
    "Unlicense",
];

const FORBIDDEN_PACKAGE_TOKENS: &[&str] = &[
    "egui",
    "winit",
    "dicom",
    "openjpeg",
    "jpeg2000",
    "vtk",
    "gdcm",
    "dcmtk",
    "tokio",
    "wasm-bindgen",
    "web-sys",
];

#[derive(Debug, Clone)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_usize(&mut self, upper_exclusive: usize) -> usize {
        if upper_exclusive <= 1 {
            0
        } else {
            (self.next_u64() as usize) % upper_exclusive
        }
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }

    fn next_unit_signed_f64(&mut self) -> f64 {
        let bits = self.next_u64() >> 11;
        let unit = (bits as f64) * (1.0 / ((1_u64 << 53) as f64));
        unit * 2.0 - 1.0
    }

    fn next_adversarial_f64(&mut self) -> f64 {
        match self.next_usize(16) {
            0 => f64::NAN,
            1 => f64::INFINITY,
            2 => f64::NEG_INFINITY,
            3 => 0.0,
            4 => -0.0,
            5 => 1.0,
            6 => -1.0,
            7 => 0.5,
            8 => 1.0e-310,
            9 => -1.0e-310,
            10 => 1.0e-7,
            11 => 1.0e200,
            12 => -1.0e200,
            13 => f64::MAX,
            14 => f64::MIN,
            _ => self.next_unit_signed_f64() * 2048.0,
        }
    }
}

fn resolve_tool_binary(env_key: &str, fallback_name: &str) -> PathBuf {
    if let Ok(val) = std::env::var(env_key)
        && !val.trim().is_empty()
    {
        return PathBuf::from(val);
    }
    let homebrew_candidate = Path::new("/opt/homebrew/opt/rustup/bin").join(fallback_name);
    if homebrew_candidate.is_file() {
        return homebrew_candidate;
    }
    PathBuf::from(fallback_name)
}

fn command_with_rustup_path(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    let rustup_bin = Path::new("/opt/homebrew/opt/rustup/bin");
    if rustup_bin.is_dir() {
        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = vec![rustup_bin.to_path_buf()];
        paths.extend(std::env::split_paths(&current_path));
        if let Ok(joined) = std::env::join_paths(paths) {
            cmd.env("PATH", joined);
        }
    }
    cmd
}

fn split_spdx_expression(expr: &str) -> Result<Vec<String>, String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err("empty SPDX licence expression".to_string());
    }
    let cleaned = trimmed.replace(['(', ')'], " ");
    let mut tokens = Vec::new();
    for or_part in cleaned.split(" OR ") {
        for and_part in or_part.split(" AND ") {
            for slash_part in and_part.split('/') {
                let tok = slash_part.trim();
                if tok.is_empty() {
                    return Err(format!("malformed empty token in SPDX expression `{expr}`"));
                }
                tokens.push(tok.to_string());
            }
        }
    }
    if tokens.is_empty() {
        return Err(format!("no licence tokens parsed from `{expr}`"));
    }
    Ok(tokens)
}

fn validate_spdx_expression(expr: &str) -> Result<(), String> {
    let tokens = split_spdx_expression(expr)?;
    for tok in tokens {
        if !ALLOWED_SPDX_LICENCES.contains(&tok.as_str()) {
            return Err(format!(
                "disallowed or unrecognised SPDX licence token `{tok}` in `{expr}`"
            ));
        }
    }
    Ok(())
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Strips line comments (`//...`), block comments (`/*...*/`), and string literals (`"..."`)
/// while preserving line structure for static governance token scanning.
fn strip_comments_and_strings(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let mut block_depth = 0usize;

    while i < bytes.len() {
        if block_depth > 0 {
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                block_depth += 1;
                out.push_str("  ");
                i += 2;
            } else if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                block_depth -= 1;
                out.push_str("  ");
                i += 2;
            } else {
                out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                i += 1;
            }
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            block_depth = 1;
            out.push_str("  ");
            i += 2;
            continue;
        }

        if bytes[i] == b'"' {
            out.push('"');
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    out.push_str("  ");
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    out.push('"');
                    i += 1;
                    break;
                }
                out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                i += 1;
            }
            continue;
        }

        out.push(bytes[i] as char);
        i += 1;
    }

    out
}

fn contains_unsafe_keyword_token(stripped_src: &str) -> bool {
    let bytes = stripped_src.as_bytes();
    let needle = b"unsafe";
    for idx in 0..bytes.len().saturating_sub(needle.len() - 1) {
        if &bytes[idx..idx + needle.len()] == needle {
            let prev_is_ident =
                idx > 0 && (bytes[idx - 1].is_ascii_alphanumeric() || bytes[idx - 1] == b'_');
            let after_idx = idx + needle.len();
            let next_is_ident = after_idx < bytes.len()
                && (bytes[after_idx].is_ascii_alphanumeric() || bytes[after_idx] == b'_');
            if !prev_is_ident && !next_is_ident {
                return true;
            }
        }
    }
    false
}

fn assert_all_pools_zero(tracker: &ResourceTracker, context: &str) {
    let snap = tracker.snapshot();
    assert_eq!(
        snap.retained_source_bytes, 0,
        "leaked retained_source_bytes at {context}"
    );
    assert_eq!(
        snap.gpu_staging_upload_bytes, 0,
        "leaked gpu_staging_upload_bytes at {context}"
    );
    assert_eq!(
        snap.gpu_resident_bytes, 0,
        "leaked gpu_resident_bytes at {context}"
    );
    assert_eq!(
        snap.gpu_staging_readback_bytes, 0,
        "leaked gpu_staging_readback_bytes at {context}"
    );
    assert_eq!(
        snap.cpu_output_bytes, 0,
        "leaked cpu_output_bytes at {context}"
    );
    assert_eq!(snap.in_flight_jobs, 0, "leaked in_flight_jobs at {context}");
}

fn generate_fuzz_voi_request(rng: &mut SplitMix64) -> VoiRequest {
    let polarity = if rng.next_bool() {
        PhotometricPolarity::Monochrome2
    } else {
        PhotometricPolarity::Monochrome1
    };
    let pad_code = (rng.next_u64() & 0xFF) as u8;

    if rng.next_usize(4) == 0 {
        // Constructive valid window/rescale branch.
        let center = (rng.next_unit_signed_f64() * 2000.0).round();
        let width = match rng.next_usize(4) {
            0 => 1.0,
            1 => 2.0,
            2 => 400.0,
            _ => 1.0 + (rng.next_u64() % 4096) as f64,
        };
        let rescale = match rng.next_usize(4) {
            0 => IntensityTransform::Identity,
            1 => IntensityTransform::LinearRescale {
                slope: 1.0,
                intercept: -1024.0,
                unit: ModalityUnit::HounsfieldUnit,
            },
            2 => IntensityTransform::LinearRescale {
                slope: -0.5,
                intercept: 100.0,
                unit: ModalityUnit::Other,
            },
            _ => IntensityTransform::LinearRescale {
                slope: 0.0,
                intercept: 42.0,
                unit: ModalityUnit::Unspecified,
            },
        };
        return VoiRequest::new(
            VoiParameters::Linear { center, width },
            rescale,
            polarity,
            pad_code,
        );
    }

    let voi = match rng.next_usize(8) {
        0..=4 => VoiParameters::Linear {
            center: rng.next_adversarial_f64(),
            width: rng.next_adversarial_f64(),
        },
        5 => VoiParameters::LinearExact {
            center: 50.0,
            width: 100.0,
        },
        6 => VoiParameters::Sigmoid {
            center: 50.0,
            width: 100.0,
        },
        _ => VoiParameters::UnsupportedLutSequence,
    };

    let rescale = match rng.next_usize(5) {
        0 => IntensityTransform::Identity,
        1..=3 => IntensityTransform::LinearRescale {
            slope: rng.next_adversarial_f64(),
            intercept: rng.next_adversarial_f64(),
            unit: ModalityUnit::HounsfieldUnit,
        },
        _ => IntensityTransform::UnsupportedModalityLutSequence,
    };

    VoiRequest::new(voi, rescale, polarity, pad_code)
}

#[test]
fn fs01_t16_seeded_splitmix64_descriptor_and_voi_fuzz() {
    let mut limits = ResourceLimits::default_limits();
    limits.retained_source_bytes = 256;
    limits.cpu_output_bytes = 128;
    limits.max_volume_bytes = 65_536;
    limits.max_frame_dim = 16;
    limits.max_volume_dim = 32;
    limits.max_in_flight_jobs = 2;

    let tracker = ResourceTracker::new(limits);
    let mut rng = SplitMix64::new(SPLITMIX64_SEED);

    let mut admitted_frames = 0usize;
    let mut rejected_frames = 0usize;
    let mut rendered_cpu_ok = 0usize;
    let mut rendered_cpu_err = 0usize;
    let mut admitted_planes = 0usize;
    let mut rejected_planes = 0usize;
    let mut admitted_volumes = 0usize;
    let mut rejected_volumes = 0usize;

    for iter in 0..FUZZ_ITERATIONS {
        let owner = NonZeroU64::new((iter as u64) + 1).expect("non-zero owner");
        let id = SourceIdentity::new(owner, rng.next_u64());

        // --- Part A: FrameDescriptor + DecodedFrameLease + VoiRequest + render_reference ---
        let constructive_frame = rng.next_usize(4) == 0;
        let (desc, lease_res) = if constructive_frame {
            let width = (rng.next_usize(8) + 1) as u32;
            let height = (rng.next_usize(8) + 1) as u32;
            let sample_type = if rng.next_bool() {
                SampleType::I16
            } else {
                SampleType::U16
            };
            let bits_stored: u8 = match rng.next_usize(3) {
                0 => 12,
                1 => 16,
                _ => (rng.next_usize(16) + 1) as u8,
            };
            let precision = BitPrecision::new(16, bits_stored, bits_stored - 1);
            let extra_stride = rng.next_usize(3) * 2;
            let row_stride_bytes = (width as usize) * 2 + extra_stride;
            let byte_offset = rng.next_usize(3) * 2;
            let total_bytes = byte_offset + (height as usize) * row_stride_bytes;

            let pixel_padding = match (sample_type, rng.next_usize(3)) {
                (_, 0) => PixelPadding::None,
                (SampleType::I16, 1) => PixelPadding::Single(-3024),
                (SampleType::I16, _) => PixelPadding::Range {
                    start: -3024,
                    end: -2000,
                },
                (SampleType::U16, 1) => PixelPadding::Single(0),
                (SampleType::U16, _) => PixelPadding::Range {
                    start: 60000,
                    end: 65535,
                },
                _ => PixelPadding::None,
            };

            let mut d =
                FrameDescriptor::new(width, height, row_stride_bytes, sample_type, precision);
            d.byte_offset = byte_offset;
            d.pixel_padding = pixel_padding;

            let word_len = total_bytes / 2;
            let lease_res = match rng.next_usize(3) {
                0 => {
                    let mut raw = vec![0_u8; total_bytes];
                    for chunk in raw.chunks_exact_mut(2) {
                        let word: u16 = if sample_type == SampleType::I16 {
                            if rng.next_usize(6) == 0
                                && matches!(
                                    pixel_padding,
                                    PixelPadding::Single(-3024) | PixelPadding::Range { .. }
                                )
                            {
                                (-3024_i16) as u16
                            } else if bits_stored == 16 {
                                rng.next_u64() as i16 as u16
                            } else {
                                let shift = 16 - bits_stored;
                                let raw_i16 = rng.next_u64() as i16;
                                ((raw_i16 << shift) >> shift) as u16
                            }
                        } else if bits_stored == 16 {
                            rng.next_u64() as u16
                        } else {
                            let mask = (1_u16 << bits_stored) - 1;
                            (rng.next_u64() as u16) & mask
                        };
                        chunk.copy_from_slice(&word.to_le_bytes());
                    }
                    Ok(DecodedFrameLease::from_vec_u8(raw, id))
                }
                1 => {
                    let mut words = Vec::with_capacity(word_len);
                    for _ in 0..word_len {
                        let val = if sample_type == SampleType::I16 {
                            if bits_stored == 16 {
                                rng.next_u64() as i16
                            } else {
                                let shift = 16 - bits_stored;
                                ((rng.next_u64() as i16) << shift) >> shift
                            }
                        } else if bits_stored == 16 {
                            (rng.next_u64() & 0x7FFF) as i16
                        } else {
                            let mask = (1_u16 << bits_stored) - 1;
                            ((rng.next_u64() as u16) & mask) as i16
                        };
                        words.push(val);
                    }
                    DecodedFrameLease::from_vec_i16(words, id)
                }
                _ => {
                    let mut words = Vec::with_capacity(word_len);
                    for _ in 0..word_len {
                        let val = if sample_type == SampleType::U16 {
                            if bits_stored == 16 {
                                rng.next_u64() as u16
                            } else {
                                (rng.next_u64() as u16) & ((1_u16 << bits_stored) - 1)
                            }
                        } else if bits_stored == 16 {
                            rng.next_u64() as i16 as u16
                        } else {
                            let shift = 16 - bits_stored;
                            (((rng.next_u64() as i16) << shift) >> shift) as u16
                        };
                        words.push(val);
                    }
                    DecodedFrameLease::from_vec_u16(words, id)
                }
            };
            (d, lease_res)
        } else {
            let width = match rng.next_usize(8) {
                0 => 0,
                1 => 1,
                2 => 16,
                3 => 17,
                4 => 4096,
                5 => u32::MAX,
                6 => u32::MAX - 1,
                _ => (rng.next_usize(20)) as u32,
            };
            let height = match rng.next_usize(8) {
                0 => 0,
                1 => 1,
                2 => 16,
                3 => 17,
                4 => 4096,
                5 => u32::MAX,
                6 => u32::MAX - 1,
                _ => (rng.next_usize(20)) as u32,
            };
            let sample_type = match rng.next_usize(5) {
                0 => SampleType::I16,
                1 => SampleType::U16,
                2 => SampleType::U8,
                3 => SampleType::I32,
                _ => SampleType::F32,
            };
            let bits_allocated = match rng.next_usize(5) {
                0 => 0,
                1 => 8,
                2 => 16,
                3 => 32,
                _ => (rng.next_u64() & 0xFF) as u8,
            };
            let bits_stored = match rng.next_usize(6) {
                0 => 0,
                1 => 12,
                2 => 16,
                3 => 17,
                _ => (rng.next_usize(20)) as u8,
            };
            let high_bit = match rng.next_usize(5) {
                0 => bits_stored.wrapping_sub(1),
                1 => 15,
                2 => 11,
                3 => 0,
                _ => (rng.next_u64() & 0x1F) as u8,
            };
            let row_stride_bytes = match rng.next_usize(8) {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                4 => usize::MAX,
                5 => usize::MAX - 1,
                _ => rng.next_usize(64),
            };
            let mut d = FrameDescriptor::new(
                width,
                height,
                row_stride_bytes,
                sample_type,
                BitPrecision::new(bits_allocated, bits_stored, high_bit),
            );
            d.frame_count = match rng.next_usize(5) {
                0 => 0,
                1..=3 => 1,
                _ => 2 + (rng.next_usize(4) as u32),
            };
            d.byte_offset = match rng.next_usize(6) {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => usize::MAX,
                _ => rng.next_usize(48),
            };
            d.components = match rng.next_usize(4) {
                0..=2 => 1,
                _ => (rng.next_usize(4)) as u8,
            };
            d.color_model = match rng.next_usize(5) {
                0..=2 => ColorModel::Grayscale,
                3 => ColorModel::PaletteIndexed,
                _ => ColorModel::Color,
            };
            d.pixel_padding = match rng.next_usize(6) {
                0 => PixelPadding::None,
                1 => PixelPadding::Single(-32769),
                2 => PixelPadding::Single(65536),
                3 => PixelPadding::Single(-2000),
                4 => PixelPadding::Range {
                    start: 100,
                    end: -100,
                },
                _ => PixelPadding::Range {
                    start: i32::MIN,
                    end: i32::MAX,
                },
            };

            let byte_len = rng.next_usize(96);
            let mut buf = vec![0_u8; byte_len];
            for b in &mut buf {
                *b = (rng.next_u64() & 0xFF) as u8;
            }
            let lease_res = match rng.next_usize(4) {
                0 => Ok(DecodedFrameLease::from_vec_u8(buf, id)),
                1 => {
                    let cap = rng.next_usize(320);
                    let arc: Arc<[u8]> = Arc::from(buf.into_boxed_slice());
                    DecodedFrameLease::from_arc_bytes_with_capacity(arc, cap, id)
                }
                2 => {
                    let words: Vec<i16> =
                        (0..(byte_len / 2)).map(|_| rng.next_u64() as i16).collect();
                    let cap = rng.next_usize(320);
                    let arc: Arc<[i16]> = Arc::from(words.into_boxed_slice());
                    DecodedFrameLease::from_arc_i16_with_capacity(arc, cap, id)
                }
                _ => {
                    let words: Vec<u16> =
                        (0..(byte_len / 2)).map(|_| rng.next_u64() as u16).collect();
                    let cap = rng.next_usize(320);
                    let arc: Arc<[u16]> = Arc::from(words.into_boxed_slice());
                    DecodedFrameLease::from_arc_u16_with_capacity(arc, cap, id)
                }
            };
            (d, lease_res)
        };

        if let Ok(lease) = lease_res {
            match admit_frame(desc, lease, &tracker) {
                Ok(checked) => {
                    admitted_frames += 1;
                    assert!(checked.width() >= 1 && checked.width() <= limits.max_frame_dim);
                    assert!(checked.height() >= 1 && checked.height() <= limits.max_frame_dim);
                    assert_eq!(checked.precision().bits_allocated, 16);
                    assert!((1..=16).contains(&checked.precision().bits_stored));
                    assert_eq!(
                        checked.precision().high_bit,
                        checked.precision().bits_stored - 1
                    );

                    let s_first = checked.sample_i32(0, 0);
                    let s_last = checked.sample_i32(checked.width() - 1, checked.height() - 1);

                    let voi_req = generate_fuzz_voi_request(&mut rng);
                    let cancel = CancellationToken::new();
                    if rng.next_usize(32) == 0 {
                        cancel.cancel();
                    }
                    match render_reference(&checked, &voi_req, &tracker, &cancel) {
                        Ok(rendered) => {
                            rendered_cpu_ok += 1;
                            let expected_len =
                                (checked.width() as usize) * (checked.height() as usize);
                            assert_eq!(rendered.pixels().len(), expected_len);
                            assert_eq!(
                                rendered.descriptor().row_stride_bytes,
                                checked.width() as usize
                            );
                            assert_eq!(rendered.descriptor().bits_per_pixel, 8);
                            if checked.pixel_padding().matches(s_first) {
                                assert_eq!(rendered.pixels()[0], voi_req.padding_display_code);
                            }
                            assert_eq!(checked.sample_i32(0, 0), s_first);
                            assert_eq!(
                                checked.sample_i32(checked.width() - 1, checked.height() - 1),
                                s_last
                            );
                        }
                        Err(_) => {
                            rendered_cpu_err += 1;
                        }
                    }
                }
                Err(_) => {
                    rejected_frames += 1;
                }
            }
        } else {
            rejected_frames += 1;
        }

        // --- Part B: PatientPlane adversarial & constructive fuzzing ---
        let (origin, r_cos, c_cos, dr, dc, well_conditioned_plane) = if rng.next_usize(4) == 0 {
            let theta = rng.next_unit_signed_f64() * std::f64::consts::PI;
            let (s, c) = theta.sin_cos();
            let r = [c, s, 0.0];
            let col_c = [-s, c, 0.0];
            let orig = LpsPointMm::new(
                rng.next_unit_signed_f64() * 500.0,
                rng.next_unit_signed_f64() * 500.0,
                rng.next_unit_signed_f64() * 500.0,
            );
            let row_sp = 0.25 + (rng.next_usize(40) as f64) * 0.1;
            let col_sp = 0.25 + (rng.next_usize(40) as f64) * 0.1;
            (orig, r, col_c, row_sp, col_sp, true)
        } else {
            let orig = LpsPointMm::new(
                rng.next_adversarial_f64(),
                rng.next_adversarial_f64(),
                rng.next_adversarial_f64(),
            );
            let r = [
                rng.next_adversarial_f64(),
                rng.next_adversarial_f64(),
                rng.next_adversarial_f64(),
            ];
            let col_c = [
                rng.next_adversarial_f64(),
                rng.next_adversarial_f64(),
                rng.next_adversarial_f64(),
            ];
            (
                orig,
                r,
                col_c,
                rng.next_adversarial_f64(),
                rng.next_adversarial_f64(),
                false,
            )
        };

        match PatientPlane::new(origin, r_cos, c_cos, dr, dc) {
            Ok(plane) => {
                admitted_planes += 1;
                let n = plane.normal_cosine();
                let n_norm = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                assert!((n_norm - 1.0).abs() <= 1.0e-5);

                let idx = PixelIndex2D::new(3.5, -2.25);
                if let Ok(pt) = plane.pixel_to_patient(idx)
                    && let Ok((roundtrip, normal_dist)) = plane.patient_to_pixel(pt)
                {
                    assert!(roundtrip.col.is_finite() && roundtrip.row.is_finite());
                    assert!(normal_dist.is_finite());
                    if well_conditioned_plane {
                        assert!((roundtrip.col - idx.col).abs() < 1.0e-9);
                        assert!((roundtrip.row - idx.row).abs() < 1.0e-9);
                        assert!(normal_dist.abs() < 1.0e-9);
                    }
                }
            }
            Err(_) => {
                rejected_planes += 1;
            }
        }

        // --- Part C: VolumeDescriptor 4x4 matrix adversarial & constructive fuzzing ---
        let (vol_desc, well_conditioned_vol) = if rng.next_usize(4) == 0 {
            let dims = VolumeDimensions::new(
                (rng.next_usize(16) + 1) as u32,
                (rng.next_usize(16) + 1) as u32,
                (rng.next_usize(16) + 1) as u32,
            );
            let sign_z = if rng.next_bool() { 1.0 } else { -1.0 };
            let sx = 0.5 + (rng.next_usize(10) as f64) * 0.25;
            let sy = 0.5 + (rng.next_usize(10) as f64) * 0.25;
            let sz = (0.5 + (rng.next_usize(10) as f64) * 0.5) * sign_z;
            let affine = [
                [sx, 0.0, 0.0, rng.next_unit_signed_f64() * 250.0],
                [0.0, sy, 0.0, rng.next_unit_signed_f64() * 250.0],
                [0.0, 0.0, sz, rng.next_unit_signed_f64() * 250.0],
                [0.0, 0.0, 0.0, 1.0],
            ];
            let vd = VolumeDescriptor::new(
                dims,
                SampleType::I16,
                BitPrecision::new(16, 12, 11),
                affine,
                id,
            );
            (vd, true)
        } else {
            let dims = VolumeDimensions::new(
                match rng.next_usize(6) {
                    0 => 0,
                    1 => u32::MAX,
                    2 => 33,
                    _ => (rng.next_usize(16) + 1) as u32,
                },
                match rng.next_usize(6) {
                    0 => 0,
                    1 => u32::MAX,
                    _ => (rng.next_usize(16) + 1) as u32,
                },
                match rng.next_usize(6) {
                    0 => 0,
                    1 => u32::MAX,
                    _ => (rng.next_usize(16) + 1) as u32,
                },
            );
            let mut affine = [[0.0_f64; 4]; 4];
            if rng.next_usize(3) == 0 {
                // Near-singular or gantry-sheared matrix with valid bottom row.
                affine[0] = [1.0, 0.0, rng.next_adversarial_f64(), 10.0];
                affine[1] = [0.0, 1.0, 0.05, 20.0];
                affine[2] = [0.0, 0.0, rng.next_adversarial_f64(), 30.0];
                affine[3] = [0.0, 0.0, 0.0, 1.0];
            } else {
                for row in &mut affine {
                    for entry in row.iter_mut() {
                        *entry = rng.next_adversarial_f64();
                    }
                }
            }
            let sample_type = if rng.next_usize(4) == 0 {
                SampleType::F32
            } else {
                SampleType::U16
            };
            let vd =
                VolumeDescriptor::new(dims, sample_type, BitPrecision::new(16, 16, 15), affine, id);
            (vd, false)
        };

        match admit_volume(vol_desc, &limits) {
            Ok(vol) => {
                admitted_volumes += 1;
                assert!(vol.voxel_count() > 0);
                assert_eq!(vol.byte_len(), vol.voxel_count() * 2);
                assert!(vol.byte_len() <= limits.max_volume_bytes);
                if let Ok(pt) = vol.voxel_to_patient(2.0, 3.0, 1.5)
                    && let Ok(ijk) = vol.patient_to_voxel(pt)
                {
                    assert!(ijk.iter().all(|v| v.is_finite()));
                    if well_conditioned_vol {
                        assert!((ijk[0] - 2.0).abs() < 1.0e-9);
                        assert!((ijk[1] - 3.0).abs() < 1.0e-9);
                        assert!((ijk[2] - 1.5).abs() < 1.0e-9);
                    }
                }
            }
            Err(_) => {
                rejected_volumes += 1;
            }
        }

        if (iter & 0x3FF) == 0x3FF {
            assert_all_pools_zero(&tracker, "periodic fuzz checkpoint");
        }
    }

    assert_all_pools_zero(&tracker, "post-100k fuzz completion");
    assert!(admitted_frames > 1_000, "admitted_frames={admitted_frames}");
    assert!(rejected_frames > 1_000, "rejected_frames={rejected_frames}");
    assert!(rendered_cpu_ok > 500, "rendered_cpu_ok={rendered_cpu_ok}");
    assert!(
        rendered_cpu_err > 500,
        "rendered_cpu_err={rendered_cpu_err}"
    );
    assert!(admitted_planes > 1_000, "admitted_planes={admitted_planes}");
    assert!(rejected_planes > 1_000, "rejected_planes={rejected_planes}");
    assert!(
        admitted_volumes > 1_000,
        "admitted_volumes={admitted_volumes}"
    );
    assert!(
        rejected_volumes > 1_000,
        "rejected_volumes={rejected_volumes}"
    );
}

#[test]
fn fs01_t18_layering_forbid_unsafe_and_sbom_licence_check() {
    // Verify SPDX validator hard-fails copyleft, MPL, and unknown tokens first.
    assert!(validate_spdx_expression("GPL-3.0-only").is_err());
    assert!(validate_spdx_expression("MPL-2.0").is_err());
    assert!(validate_spdx_expression("MIT OR LGPL-2.1-or-later").is_err());
    assert!(validate_spdx_expression("Proprietary-Unknown").is_err());
    assert!(validate_spdx_expression("(MIT OR Apache-2.0) AND Unicode-3.0").is_ok());
    assert!(validate_spdx_expression("Apache-2.0 WITH LLVM-exception OR MIT").is_ok());

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let rustc_bin = resolve_tool_binary("RUSTC", "rustc");
    let cargo_bin = resolve_tool_binary("CARGO", "cargo");

    let rustc_vv = command_with_rustup_path(&rustc_bin)
        .arg("-vV")
        .output()
        .expect("failed to execute rustc -vV");
    assert!(rustc_vv.status.success(), "rustc -vV failed");
    let rustc_stdout = String::from_utf8(rustc_vv.stdout).expect("utf8 rustc -vV");
    let host_triple = rustc_stdout
        .lines()
        .find_map(|line| line.strip_prefix("host:").map(str::trim))
        .expect("missing `host:` line in rustc -vV");
    assert!(!host_triple.is_empty());

    let meta_out = command_with_rustup_path(&cargo_bin)
        .current_dir(&manifest_dir)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--filter-platform",
            host_triple,
        ])
        .output()
        .expect("failed to execute cargo metadata");
    assert!(
        meta_out.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&meta_out.stderr)
    );

    let meta: serde_json::Value =
        serde_json::from_slice(&meta_out.stdout).expect("valid cargo metadata JSON");

    let packages = meta["packages"]
        .as_array()
        .expect("packages array in cargo metadata");
    let resolve = &meta["resolve"];
    let root_id = resolve["root"]
        .as_str()
        .expect("resolve.root in cargo metadata");

    let root_pkg = packages
        .iter()
        .find(|p| p["id"].as_str() == Some(root_id))
        .expect("root package entry");
    assert_eq!(root_pkg["name"].as_str(), Some("dicomflux-3d"));

    // (a) Direct normal dependencies of dicomflux-3d must equal ["sha2", "wgpu"].
    let mut direct_normal_deps: Vec<String> = root_pkg["dependencies"]
        .as_array()
        .expect("root dependencies array")
        .iter()
        .filter(|dep| dep["kind"].is_null())
        .map(|dep| dep["name"].as_str().expect("dependency name").to_string())
        .collect();
    direct_normal_deps.sort();
    assert_eq!(
        direct_normal_deps,
        vec!["sha2".to_string(), "wgpu".to_string()],
        "unexpected normal direct dependencies"
    );

    // (b) & (c) Inspect every resolved package for forbidden tokens and SPDX licence allowlist.
    let resolved_ids: BTreeSet<&str> = resolve["nodes"]
        .as_array()
        .expect("resolve.nodes array")
        .iter()
        .map(|node| node["id"].as_str().expect("node.id"))
        .collect();
    assert!(resolved_ids.len() >= 3);

    for pkg_id in &resolved_ids {
        let pkg = packages
            .iter()
            .find(|p| p["id"].as_str() == Some(*pkg_id))
            .unwrap_or_else(|| panic!("missing package record for {pkg_id}"));
        let name = pkg["name"].as_str().expect("package name");
        let name_lower = name.to_ascii_lowercase();

        if name != "dicomflux-3d" {
            for forbidden in FORBIDDEN_PACKAGE_TOKENS {
                assert!(
                    !name_lower.contains(forbidden),
                    "resolved dependency `{name}` matched forbidden token `{forbidden}`"
                );
            }
        }

        let license_expr = pkg["license"]
            .as_str()
            .unwrap_or_else(|| panic!("resolved package `{name}` has no SPDX license field"));
        if let Err(reason) = validate_spdx_expression(license_expr) {
            panic!("SPDX licence check failed for `{name}`: {reason}");
        }
    }

    // (d) Static source audit: #![forbid(unsafe_code)], zero `unsafe` blocks, zero public wgpu types.
    let lib_rs_path = manifest_dir.join("src").join("lib.rs");
    let lib_rs = fs::read_to_string(&lib_rs_path).expect("read src/lib.rs");
    assert!(
        lib_rs.contains("#![forbid(unsafe_code)]"),
        "src/lib.rs must contain #![forbid(unsafe_code)]"
    );
    assert!(
        lib_rs.contains("#![deny(missing_debug_implementations)]"),
        "src/lib.rs must contain #![deny(missing_debug_implementations)]"
    );

    let mut rs_files = Vec::new();
    collect_rs_files(&manifest_dir.join("src"), &mut rs_files);
    assert!(!rs_files.is_empty());

    for file_path in rs_files {
        let raw_src = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", file_path.display()));
        let stripped = strip_comments_and_strings(&raw_src);
        let without_forbid = stripped.replace("#![forbid(unsafe_code)]", "");
        assert!(
            !contains_unsafe_keyword_token(&without_forbid),
            "found `unsafe` keyword in {}",
            file_path.display()
        );

        // Scan non-pub(crate)/non-pub(super) `pub` item headers up to `{` or `;` for `wgpu`.
        let lines: Vec<&str> = stripped.lines().collect();
        let mut idx = 0usize;
        while idx < lines.len() {
            let trimmed = lines[idx].trim_start();
            let is_crate_restricted = trimmed.starts_with("pub(crate)")
                || trimmed.starts_with("pub(super)")
                || trimmed.starts_with("pub(in ");
            let is_public_item = !is_crate_restricted
                && (trimmed.starts_with("pub ") || trimmed.starts_with("pub\t"));

            if is_public_item {
                let mut header = String::new();
                let mut scan = idx;
                while scan < lines.len() {
                    let seg = lines[scan];
                    if let Some(pos) = seg.find(['{', ';']) {
                        header.push_str(&seg[..=pos]);
                        break;
                    }
                    header.push_str(seg);
                    header.push(' ');
                    scan += 1;
                }
                assert!(
                    !header.contains("pub use wgpu")
                        && !header.contains("pub extern crate wgpu")
                        && !header.contains("wgpu::"),
                    "public signature leaks wgpu in {}: `{header}`",
                    file_path.display()
                );
            }
            idx += 1;
        }
    }
}

#[test]
fn fs01_t23_privacy_redaction_of_debug_and_display() {
    let tracker = ResourceTracker::with_default_limits();
    let id = SourceIdentity::new(NonZeroU64::new(77).unwrap(), 99);

    let sentinel_bytes: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF];
    let sentinel_i16: Vec<i16> = vec![-12345_i16, 23456_i16, -12345_i16, 23456_i16];
    let sentinel_u16: Vec<u16> = vec![54321_u16, 43210_u16, 54321_u16, 43210_i32 as u16];

    let lease_bytes = DecodedFrameLease::from_vec_u8(sentinel_bytes, id);
    let lease_i16 = DecodedFrameLease::from_vec_i16(sentinel_i16, id).expect("i16 lease");
    let lease_u16 = DecodedFrameLease::from_vec_u16(sentinel_u16, id).expect("u16 lease");

    let desc_i16 = FrameDescriptor::new(2, 2, 4, SampleType::I16, BitPrecision::new(16, 16, 15));
    let desc_u16 = FrameDescriptor::new(2, 2, 4, SampleType::U16, BitPrecision::new(16, 16, 15));

    let checked_bytes = admit_frame(desc_u16, lease_bytes.clone(), &tracker).expect("admit bytes");
    let checked_i16 = admit_frame(desc_i16, lease_i16.clone(), &tracker).expect("admit i16");
    let checked_u16 = admit_frame(desc_u16, lease_u16.clone(), &tracker).expect("admit u16");

    let sentinel_origin = LpsPointMm::new(98765.4321, -54321.9876, 13579.2468);
    let plane = PatientPlane::new(sentinel_origin, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5, 0.5)
        .expect("valid plane");

    let vol_desc = VolumeDescriptor::new(
        VolumeDimensions::new(4, 4, 4),
        SampleType::I16,
        BitPrecision::new(16, 16, 15),
        [
            [0.8, 0.0, 0.0, 98765.4321],
            [0.0, 0.8, 0.0, -54321.9876],
            [0.0, 0.0, 1.5, 13579.2468],
            [0.0, 0.0, 0.0, 1.0],
        ],
        id,
    );
    let checked_vol = admit_volume(vol_desc, tracker.limits()).expect("valid volume");

    let cancel = CancellationToken::new();
    let voi_req = VoiRequest::linear(100.0, 400.0);
    let cpu_img =
        render_reference(&checked_i16, &voi_req, &tracker, &cancel).expect("CPU reference render");

    let cap_report = probe_capabilities(&AdapterSelectionPolicy::default());
    let mut debug_corpus = vec![
        format!("{lease_bytes:?}"),
        format!("{lease_i16:?}"),
        format!("{lease_u16:?}"),
        format!("{checked_bytes:?}"),
        format!("{checked_i16:?}"),
        format!("{checked_u16:?}"),
        format!("{sentinel_origin:?}"),
        format!("{plane:?}"),
        format!("{vol_desc:?}"),
        format!("{checked_vol:?}"),
        format!("{cpu_img:?}"),
        format!("{cap_report:?}"),
        format!("{}", OperationId(42)),
        format!("{}", LimitPoolKind::RetainedSource),
    ];

    if !cap_report.gpu_adapters.is_empty() {
        let gpu_ctx =
            GpuContext::new(&AdapterSelectionPolicy::default(), &tracker).expect("GPU context");
        let gpu_img = render_offscreen(&gpu_ctx, &checked_i16, &voi_req, &cancel)
            .expect("GPU offscreen render");
        debug_corpus.push(format!("{gpu_ctx:?}"));
        debug_corpus.push(format!("{gpu_img:?}"));
    }

    // Exercise all 11 canonical RenderError variants (clean and adversarial static strings).
    let all_11_errors = [
        RenderError::InvalidDescriptor("origin 98765.4321 -54321.9876 13579.2468"),
        RenderError::UnsupportedPixelFormat {
            bits_allocated: 8,
            bits_stored: 8,
            components: 3,
        },
        RenderError::UnsupportedPresentation("payload 0xDEADBEEF /Users/john_doe/scan.dcm"),
        RenderError::UnsupportedGeometry("uid 1.2.840.10008.5.1.4.1.1.2"),
        RenderError::ResourceLimitExceeded {
            pool: LimitPoolKind::GpuResident,
            requested: 2048,
            available: 512,
            limit: 1024,
        },
        RenderError::NoSuitableAdapter,
        RenderError::UnsupportedGpuCapability("/home/patient_secret/driver_dump.dcm"),
        RenderError::GpuDeviceLost,
        RenderError::GpuOutOfMemory,
        RenderError::Cancelled,
        RenderError::InternalError("C:\\Users\\clinical_phi\\series_98765.4321"),
    ];
    assert_eq!(all_11_errors.len(), 11);

    for err in &all_11_errors {
        debug_corpus.push(format!("{err:?}"));
        debug_corpus.push(format!("{err}"));
    }

    // Also verify NoSuitableAdapter path never leaks driver paths.
    let deny_all = AdapterSelectionPolicy::new(AllowedBackends::none(), false, true);
    let adapter_err = GpuContext::new(&deny_all, &tracker).expect_err("must refuse");
    debug_corpus.push(format!("{adapter_err:?}"));
    debug_corpus.push(format!("{adapter_err}"));

    let forbidden_sentinels = [
        "deadbeef",
        "0xde",
        "0xad",
        "0xbe",
        "0xef",
        "-12345",
        "23456",
        "54321",
        "43210",
        "98765",
        "13579",
        "4321",
        "9876",
        "2468",
        "john_doe",
        "patient_secret",
        "clinical_phi",
        "scan.dcm",
        "1.2.840",
    ];

    for text in &debug_corpus {
        let lower = text.to_ascii_lowercase();
        for sentinel in forbidden_sentinels {
            assert!(
                !lower.contains(sentinel),
                "privacy redaction failure: `{sentinel}` leaked in `{text}`"
            );
        }
    }
}

#[test]
fn fs01_t24_readme_capability_and_non_clinical_language() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = fs::read_to_string(&readme_path).expect("failed to read ./README.md");
    let lower = readme.to_ascii_lowercase();

    // 1. Explicit FS-01 2D-only scope.
    assert!(
        lower.contains("fs-01"),
        "README must state FS-01 slice scope"
    );
    assert!(
        lower.contains("2d-only"),
        "README must explicitly state 2D-only capability scope"
    );

    // 2. Explicit exclusions of volumetric rendering, segmentation, parser, and codecs.
    for required_exclusion in ["mpr", "mip", "dvr", "segmentation", "codecs"] {
        assert!(
            lower.contains(required_exclusion),
            "README must explicitly mention excluded capability `{required_exclusion}`"
        );
    }
    assert!(
        lower.contains("does **not** implement") || lower.contains("does not implement"),
        "README must explicitly state what FS-01 does not implement"
    );

    // 3. Explicit non-clinical / non-medical-device safety notice.
    assert!(
        lower.contains("not a medical device") || lower.contains("not** a medical device"),
        "README must state dicomflux-3d is not a medical device"
    );
    assert!(
        lower.contains("clinical diagnostic"),
        "README must explicitly disclaim clinical diagnostic qualification"
    );

    // 4. Honest BoundedDevelopmentHost vs primary Windows/Linux target qualification notice.
    assert!(
        readme.contains("BoundedDevelopmentHost"),
        "README must document BoundedDevelopmentHost stamping for macOS Metal"
    );

    // 5. Zero false regulatory or marketing claims.
    for prohibited_claim in ["fda cleared", "fda-cleared", "ce marked", "60 fps"] {
        assert!(
            !lower.contains(prohibited_claim),
            "README contains prohibited unverified claim `{prohibited_claim}`"
        );
    }
}
