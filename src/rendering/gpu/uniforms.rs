// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use crate::image::PixelPadding;
use crate::presentation::{PhotometricPolarity, VoiRequest};
use crate::rendering::cpu::eval_linear_voi_code;

const S_MIN: i32 = -32_768;
const S_MAX: i32 = 65_535;
const NO_LOW_CLAMP_SENTINEL: i32 = -32_769;
const NO_HIGH_CLAMP_SENTINEL: i32 = 65_536;

/// 64-byte (`16 x 4`-byte) analytic VOI uniform block matching `struct VoiUniforms` in
/// `voi_linear.wgsl` byte-for-byte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VoiUniformBlock {
    pub width: u32,
    pub height: u32,
    pub padding_mode: u32,
    pub pad_code: u32,
    pub pad_low: i32,
    pub pad_high: i32,
    pub constant_mode: u32,
    pub constant_code: u32,
    pub raw_low_clamp: i32,
    pub raw_high_clamp: i32,
    pub low_raw_code: u32,
    pub high_raw_code: u32,
    pub raw_pivot: i32,
    pub scale_f32: f32,
    pub offset_f32: f32,
    pub invert_polarity: u32,
}

const fn post_polarity(code: u8, polarity: PhotometricPolarity) -> u32 {
    match polarity {
        PhotometricPolarity::Monochrome2 => code as u32,
        PhotometricPolarity::Monochrome1 => (255 - code) as u32,
    }
}

/// Binary searches a monotonically non-increasing boolean predicate (`true` on a prefix of
/// `[lo..=hi]`, `false` on the remaining suffix) in at most 17 steps over `[-32768, 65535]`.
fn largest_satisfying_prefix<F>(mut lo: i32, mut hi: i32, pred: F) -> Option<i32>
where
    F: Fn(i32) -> bool,
{
    if !pred(lo) {
        return None;
    }
    let mut best = lo;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        if pred(mid) {
            best = mid;
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    Some(best)
}

/// Binary searches a monotonically non-decreasing boolean predicate (`false` on a prefix of
/// `[lo..=hi]`, `true` on the remaining suffix) in at most 17 steps over `[-32768, 65535]`.
fn smallest_satisfying_suffix<F>(mut lo: i32, mut hi: i32, pred: F) -> Option<i32>
where
    F: Fn(i32) -> bool,
{
    if !pred(hi) {
        return None;
    }
    let mut best = hi;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        if pred(mid) {
            best = mid;
            hi = mid - 1;
        } else {
            lo = mid + 1;
        }
    }
    Some(best)
}

impl VoiUniformBlock {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub(crate) fn solve(
        width: u32,
        height: u32,
        padding: PixelPadding,
        request: &VoiRequest,
        voi_center: f64,
        voi_width: f64,
        slope: f64,
        intercept: f64,
    ) -> Self {
        let (padding_mode, pad_low, pad_high) = match padding {
            PixelPadding::None => (0, 0, 0),
            PixelPadding::Single(v) => (1, v, v),
            PixelPadding::Range { start, end } => (2, start, end),
        };
        let pad_code = u32::from(request.padding_display_code);
        let invert_polarity = u32::from(request.polarity == PhotometricPolarity::Monochrome1);

        if slope == 0.0 {
            let constant_code = u32::from(eval_linear_voi_code(
                intercept,
                voi_center,
                voi_width,
                request.polarity,
            ));
            return Self {
                width,
                height,
                padding_mode,
                pad_code,
                pad_low,
                pad_high,
                constant_mode: 1,
                constant_code,
                raw_low_clamp: NO_LOW_CLAMP_SENTINEL,
                raw_high_clamp: NO_HIGH_CLAMP_SENTINEL,
                low_raw_code: 0,
                high_raw_code: 0,
                raw_pivot: 0,
                scale_f32: 0.0,
                offset_f32: 0.0,
                invert_polarity,
            };
        }

        let c_minus_half = voi_center - 0.5;
        let half_span = (voi_width - 1.0) * 0.5;
        let low = c_minus_half - half_span;
        let high = c_minus_half + half_span;
        let x_of = |s: i32| -> f64 { f64::from(s) * slope + intercept };

        let (raw_low_clamp, low_raw_code, raw_high_clamp, high_raw_code) = if slope > 0.0 {
            let low_bound = largest_satisfying_prefix(S_MIN, S_MAX, |s| x_of(s) <= low)
                .unwrap_or(NO_LOW_CLAMP_SENTINEL);
            let high_bound = smallest_satisfying_suffix(S_MIN, S_MAX, |s| x_of(s) > high)
                .unwrap_or(NO_HIGH_CLAMP_SENTINEL);
            (
                low_bound,
                post_polarity(0, request.polarity),
                high_bound,
                post_polarity(255, request.polarity),
            )
        } else {
            let low_bound = largest_satisfying_prefix(S_MIN, S_MAX, |s| x_of(s) > high)
                .unwrap_or(NO_LOW_CLAMP_SENTINEL);
            let high_bound = smallest_satisfying_suffix(S_MIN, S_MAX, |s| x_of(s) <= low)
                .unwrap_or(NO_HIGH_CLAMP_SENTINEL);
            (
                low_bound,
                post_polarity(255, request.polarity),
                high_bound,
                post_polarity(0, request.polarity),
            )
        };

        let (raw_pivot, scale_f32, offset_f32) = if voi_width > 1.0 {
            let ideal_pivot = (c_minus_half - intercept) / slope;
            let pivot = if ideal_pivot.is_nan() {
                0
            } else {
                ideal_pivot
                    .round()
                    .clamp(f64::from(S_MIN), f64::from(S_MAX)) as i32
            };
            let raw_scale = ((slope / (voi_width - 1.0)) * 255.0) as f32;
            let safe_scale = if raw_scale.is_finite() {
                raw_scale
            } else {
                0.0
            };
            let offset = ((((f64::from(pivot) * slope + intercept) - c_minus_half)
                / (voi_width - 1.0)
                + 0.5)
                * 255.0)
                .clamp(0.0, 255.0) as f32;
            (pivot, safe_scale, offset)
        } else {
            (0, 0.0, 0.0)
        };

        Self {
            width,
            height,
            padding_mode,
            pad_code,
            pad_low,
            pad_high,
            constant_mode: 0,
            constant_code: 0,
            raw_low_clamp,
            raw_high_clamp,
            low_raw_code,
            high_raw_code,
            raw_pivot,
            scale_f32,
            offset_f32,
            invert_polarity,
        }
    }

    #[must_use]
    pub(crate) fn to_le_bytes(self) -> [u8; 64] {
        let mut out = [0u8; 64];
        out[0..4].copy_from_slice(&self.width.to_le_bytes());
        out[4..8].copy_from_slice(&self.height.to_le_bytes());
        out[8..12].copy_from_slice(&self.padding_mode.to_le_bytes());
        out[12..16].copy_from_slice(&self.pad_code.to_le_bytes());
        out[16..20].copy_from_slice(&self.pad_low.to_le_bytes());
        out[20..24].copy_from_slice(&self.pad_high.to_le_bytes());
        out[24..28].copy_from_slice(&self.constant_mode.to_le_bytes());
        out[28..32].copy_from_slice(&self.constant_code.to_le_bytes());
        out[32..36].copy_from_slice(&self.raw_low_clamp.to_le_bytes());
        out[36..40].copy_from_slice(&self.raw_high_clamp.to_le_bytes());
        out[40..44].copy_from_slice(&self.low_raw_code.to_le_bytes());
        out[44..48].copy_from_slice(&self.high_raw_code.to_le_bytes());
        out[48..52].copy_from_slice(&self.raw_pivot.to_le_bytes());
        out[52..56].copy_from_slice(&self.scale_f32.to_le_bytes());
        out[56..60].copy_from_slice(&self.offset_f32.to_le_bytes());
        out[60..64].copy_from_slice(&self.invert_polarity.to_le_bytes());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{IntensityTransform, ModalityUnit, VoiParameters};

    fn simulate_wgsl_eval(u: &VoiUniformBlock, s: i32) -> u8 {
        if (u.padding_mode == 1 && s == u.pad_low)
            || (u.padding_mode == 2 && s >= u.pad_low && s <= u.pad_high)
        {
            return u.pad_code as u8;
        }
        if u.constant_mode == 1 {
            return u.constant_code as u8;
        }
        if s <= u.raw_low_clamp {
            return u.low_raw_code as u8;
        }
        if s >= u.raw_high_clamp {
            return u.high_raw_code as u8;
        }
        let mut y = (s - u.raw_pivot) as f32 * u.scale_f32 + u.offset_f32;
        if u.invert_polarity == 1 {
            y = 255.0 - y;
        }
        y.round_ties_even().clamp(0.0, 255.0) as u8
    }

    #[test]
    fn steep_negative_zero_and_width_one_match_f64_oracle_across_full_container_domain() {
        let cases = [
            (40.0, 400.0, 1.0, -1024.0, PhotometricPolarity::Monochrome2),
            (40.0, 1.0, 1.0, -1024.0, PhotometricPolarity::Monochrome1),
            (100.0, 250.0, -2.5, 500.0, PhotometricPolarity::Monochrome2),
            (100.0, 1.0, -2.5, 500.0, PhotometricPolarity::Monochrome1),
            (0.5, 100.0, 1.0e40, 0.0, PhotometricPolarity::Monochrome2),
            (50.0, 100.0, 0.0, 50.0, PhotometricPolarity::Monochrome2),
        ];

        let padding = PixelPadding::Range {
            start: -3024,
            end: -2000,
        };

        for (c, w, slope, intercept, polarity) in cases {
            let req = VoiRequest::new(
                VoiParameters::Linear {
                    center: c,
                    width: w,
                },
                IntensityTransform::LinearRescale {
                    slope,
                    intercept,
                    unit: ModalityUnit::HounsfieldUnit,
                },
                polarity,
                17,
            );
            let block = VoiUniformBlock::solve(64, 64, padding, &req, c, w, slope, intercept);
            if slope == 0.0 {
                assert_eq!(block.constant_mode, 1);
            } else {
                assert_eq!(block.constant_mode, 0);
            }
            assert!(block.scale_f32.is_finite());

            let low = (c - 0.5) - (w - 1.0) * 0.5;
            let high = (c - 0.5) + (w - 1.0) * 0.5;

            for s in S_MIN..=S_MAX {
                let got = simulate_wgsl_eval(&block, s);
                if padding.matches(s) {
                    assert_eq!(got, 17);
                    continue;
                }
                let x = f64::from(s) * slope + intercept;
                let expected = eval_linear_voi_code(x, c, w, polarity);
                if w == 1.0 || slope == 0.0 || x <= low || x > high {
                    assert_eq!(got, expected, "clamp/step mismatch at s={s}");
                } else {
                    assert!(
                        got.abs_diff(expected) <= 1,
                        "interior drift > 1 LSB at s={s}: got={got}, expected={expected}"
                    );
                }
            }
        }
    }
}
