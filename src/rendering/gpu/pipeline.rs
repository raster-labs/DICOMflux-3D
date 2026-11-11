// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::borrow::Cow;
use std::future::Future as _;
use std::num::NonZeroU64;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use sha2::{Digest, Sha256};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, ColorTargetState, ColorWrites, Device, Error, ErrorFilter, ErrorScopeGuard,
    FragmentState, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PollType, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat, TextureSampleType,
    TextureViewDimension, VertexState,
};

use crate::error::RenderError;

pub(crate) const WGSL_SOURCE: &str = include_str!("voi_linear.wgsl");

pub(crate) fn compute_shader_sha256() -> Arc<str> {
    let digest = Sha256::digest(WGSL_SOURCE.as_bytes());
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    Arc::from(hex)
}

/// Pops a `wgpu::ErrorScopeGuard` and resolves its future synchronously without an external async
/// executor. Callers MUST pop nested error scopes in strict reverse LIFO order.
pub(crate) fn pop_error_scope(device: &Device, guard: ErrorScopeGuard) -> Option<Error> {
    let fut = guard.pop();
    let mut fut = pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(err) = fut.as_mut().poll(&mut cx) {
            return err;
        }
        let _ = device.poll(PollType::Poll);
    }
}

#[derive(Debug)]
pub(crate) struct CachedPipelines {
    pub shader_sha256: Arc<str>,
    pub sint_bgl: BindGroupLayout,
    pub uint_bgl: BindGroupLayout,
    pub sint_pipeline: RenderPipeline,
    pub uint_pipeline: RenderPipeline,
}

impl CachedPipelines {
    pub(crate) fn build(device: &Device) -> Result<Self, RenderError> {
        let shader_sha256 = compute_shader_sha256();

        let oom_guard = device.push_error_scope(ErrorFilter::OutOfMemory);
        let val_guard = device.push_error_scope(ErrorFilter::Validation);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("dicomflux-3d-voi-linear-wgsl"),
            source: ShaderSource::Wgsl(Cow::Borrowed(WGSL_SOURCE)),
        });

        let uniform_entry = BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(64),
            },
            count: None,
        };

        let sint_bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("dicomflux-3d-sint-bgl"),
            entries: &[
                uniform_entry,
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Sint,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let uint_bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("dicomflux-3d-uint-bgl"),
            entries: &[
                uniform_entry,
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Uint,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let sint_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("dicomflux-3d-sint-pipeline-layout"),
            bind_group_layouts: &[Some(&sint_bgl)],
            immediate_size: 0,
        });

        let uint_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("dicomflux-3d-uint-pipeline-layout"),
            bind_group_layouts: &[Some(&uint_bgl)],
            immediate_size: 0,
        });

        let color_targets = [Some(ColorTargetState {
            format: TextureFormat::R8Uint,
            blend: None,
            write_mask: ColorWrites::ALL,
        })];

        let primitive = PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..PrimitiveState::default()
        };

        let sint_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("dicomflux-3d-voi-sint-pipeline"),
            layout: Some(&sint_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive,
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main_sint"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &color_targets,
            }),
            multiview_mask: None,
            cache: None,
        });

        let uint_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("dicomflux-3d-voi-uint-pipeline"),
            layout: Some(&uint_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive,
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main_uint"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &color_targets,
            }),
            multiview_mask: None,
            cache: None,
        });

        // Strict reverse LIFO order: pop Validation (pushed second) BEFORE OutOfMemory (pushed first).
        let val_err = pop_error_scope(device, val_guard);
        let oom_err = pop_error_scope(device, oom_guard);

        if oom_err.is_some() {
            return Err(RenderError::GpuOutOfMemory);
        }
        if val_err.is_some() {
            return Err(RenderError::InternalError(
                "WGSL shader or render pipeline failed GPU validation",
            ));
        }

        Ok(Self {
            shader_sha256,
            sint_bgl,
            uint_bgl,
            sint_pipeline,
            uint_pipeline,
        })
    }
}
