// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

struct VoiUniforms {
    width: u32,
    height: u32,
    padding_mode: u32,
    pad_code: u32,
    pad_low: i32,
    pad_high: i32,
    constant_mode: u32,
    constant_code: u32,
    raw_low_clamp: i32,
    raw_high_clamp: i32,
    low_raw_code: u32,
    high_raw_code: u32,
    raw_pivot: i32,
    scale_f32: f32,
    offset_f32: f32,
    invert_polarity: u32,
}

@group(0) @binding(0)
var<uniform> u: VoiUniforms;

@group(0) @binding(1)
var src_sint: texture_2d<i32>;

@group(0) @binding(2)
var src_uint: texture_2d<u32>;

@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(vid & 1u) * 4 - 1);
    let y = f32(i32(vid & 2u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

fn eval_voi_sample(s: i32) -> u32 {
    if (u.padding_mode == 1u && s == u.pad_low) || (u.padding_mode == 2u && s >= u.pad_low && s <= u.pad_high) {
        return u.pad_code;
    }
    if u.constant_mode == 1u {
        return u.constant_code;
    }
    if s <= u.raw_low_clamp {
        return u.low_raw_code;
    }
    if s >= u.raw_high_clamp {
        return u.high_raw_code;
    }
    var y: f32 = fma(f32(s - u.raw_pivot), u.scale_f32, u.offset_f32);
    if u.invert_polarity == 1u {
        y = 255.0 - y;
    }
    return u32(clamp(round(y), 0.0, 255.0));
}

@fragment
fn fs_main_sint(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<u32> {
    let coords = vec2<i32>(i32(pos.x), i32(pos.y));
    let s = textureLoad(src_sint, coords, 0).r;
    return vec4<u32>(eval_voi_sample(s), 0u, 0u, 1u);
}

@fragment
fn fs_main_uint(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<u32> {
    let coords = vec2<i32>(i32(pos.x), i32(pos.y));
    let s = i32(textureLoad(src_uint, coords, 0).r);
    return vec4<u32>(eval_voi_sample(s), 0u, 0u, 1u);
}
