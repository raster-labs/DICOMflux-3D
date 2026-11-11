// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

mod capabilities;
mod pipeline;
mod pitch;
mod uniforms;

use std::fmt;
use std::future::Future as _;
use std::pin::pin;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferDescriptor, BufferUsages,
    Color, CommandEncoderDescriptor, Device, DeviceDescriptor, ErrorFilter, ErrorScopeGuard,
    Extent3d, Features, Limits, LoadOp, MapMode, Operations, Origin3d, PollError, PollType, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TexelCopyBufferInfo,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::error::{LimitPoolKind, RenderError};
use crate::image::{
    CheckedFrame, DecodedFrameLease, FrameDescriptor, PixelPadding, SampleType, SourceIdentity,
};
use crate::presentation::{
    DisplayEncoding, MAX_GPU_VS_CPU_INTERIOR_LSB_TOLERANCE, OutputDescriptor, PresentationPath,
    ROUNDING_POLICY_TIES_EVEN_U8, RenderReceipt, RenderedImage, StageTimings, UploadAccounting,
    VoiFunction, VoiRequest,
};
use crate::resource::{BudgetGuard, CancellationToken, ResourceTracker};

pub use capabilities::{
    AdapterDeviceType, AdapterSelectionPolicy, AllowedBackends, CapabilityReport, GpuCapabilities,
    probe_capabilities,
};

use capabilities::{
    adapter_rank_key, enumerate_raw_wgpu_adapters, evaluate_adapter_profile,
    extract_raw_adapter_profile,
};
use pipeline::{CachedPipelines, pop_error_scope};
use pitch::{
    checked_readback_layout, checked_upload_layout, unpack_readback_to_tight,
    write_frame_to_upload_staging,
};
use uniforms::VoiUniformBlock;

const FAULT_HEALTHY: u8 = 0;
const FAULT_DEVICE_LOST: u8 = 1;

const MAP_PENDING: u8 = 0;
const MAP_OK: u8 = 1;
const MAP_ERR: u8 = 2;

#[derive(Debug)]
struct FaultLatch {
    state: AtomicU8,
}

impl FaultLatch {
    const fn new() -> Self {
        Self {
            state: AtomicU8::new(FAULT_HEALTHY),
        }
    }

    fn trip_device_lost(&self) {
        self.state.store(FAULT_DEVICE_LOST, Ordering::Release);
    }

    fn is_healthy(&self) -> bool {
        self.state.load(Ordering::Acquire) == FAULT_HEALTHY
    }
}

type ResidencyKey = (SourceIdentity, FrameDescriptor);

#[derive(Debug)]
struct ResidentGpuFrame {
    key: ResidencyKey,
    sample_type: SampleType,
    _texture: Texture,
    view: TextureView,
    _budget: Arc<BudgetGuard>,
}

#[derive(Debug)]
struct RetainedInFlightJob {
    done: Arc<AtomicBool>,
    _frame_lease: DecodedFrameLease,
    _source_budget: Arc<BudgetGuard>,
    _resident: Arc<ResidentGpuFrame>,
    _target_texture: Texture,
    _buffers: Vec<Buffer>,
    _guards: Vec<BudgetGuard>,
}

#[derive(Debug, Default)]
struct GpuMutableState {
    resident: Option<Arc<ResidentGpuFrame>>,
    in_flight_ledger: Vec<RetainedInFlightJob>,
}

#[cfg(test)]
#[derive(Default)]
struct FaultInjectionState {
    simulate_oom_once: bool,
    simulate_device_loss_once: bool,
    cancel_after_submit_once: Option<CancellationToken>,
}

/// Quarantined, thread-safe (`Send + Sync`) offscreen GPU execution context owning its single-slot
/// 16-bit texture residency cache, compiled WGSL pipelines, fault latch, and bound
/// [`ResourceTracker`].
pub struct GpuContext {
    device: Device,
    queue: Queue,
    pipelines: CachedPipelines,
    capabilities: GpuCapabilities,
    tracker: ResourceTracker,
    fault_state: Arc<FaultLatch>,
    submission_lock: Mutex<GpuMutableState>,
    #[cfg(test)]
    fault_injection: Mutex<FaultInjectionState>,
}

impl fmt::Debug for GpuContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (has_resident, in_flight_retained) = match self.submission_lock.lock() {
            Ok(guard) => (guard.resident.is_some(), guard.in_flight_ledger.len()),
            Err(_) => (false, 0),
        };
        f.debug_struct("GpuContext")
            .field("capabilities", &self.capabilities)
            .field("healthy", &self.is_healthy())
            .field("has_resident_frame", &has_resident)
            .field("retained_in_flight_jobs", &in_flight_retained)
            .field("device", &"<redacted>")
            .finish()
    }
}

fn clamp_device_limits(adapter_limits: &Limits) -> Limits {
    let mut req = Limits::default();
    req.max_texture_dimension_1d = req
        .max_texture_dimension_1d
        .min(adapter_limits.max_texture_dimension_1d);
    req.max_texture_dimension_2d = adapter_limits.max_texture_dimension_2d;
    req.max_texture_dimension_3d = req
        .max_texture_dimension_3d
        .min(adapter_limits.max_texture_dimension_3d);
    req.max_texture_array_layers = req
        .max_texture_array_layers
        .min(adapter_limits.max_texture_array_layers);
    req.max_bind_groups = req.max_bind_groups.min(adapter_limits.max_bind_groups);
    req.max_bindings_per_bind_group = req
        .max_bindings_per_bind_group
        .min(adapter_limits.max_bindings_per_bind_group);
    req.max_dynamic_uniform_buffers_per_pipeline_layout = req
        .max_dynamic_uniform_buffers_per_pipeline_layout
        .min(adapter_limits.max_dynamic_uniform_buffers_per_pipeline_layout);
    req.max_dynamic_storage_buffers_per_pipeline_layout = req
        .max_dynamic_storage_buffers_per_pipeline_layout
        .min(adapter_limits.max_dynamic_storage_buffers_per_pipeline_layout);
    req.max_sampled_textures_per_shader_stage = req
        .max_sampled_textures_per_shader_stage
        .min(adapter_limits.max_sampled_textures_per_shader_stage);
    req.max_samplers_per_shader_stage = req
        .max_samplers_per_shader_stage
        .min(adapter_limits.max_samplers_per_shader_stage);
    req.max_storage_buffers_per_shader_stage = req
        .max_storage_buffers_per_shader_stage
        .min(adapter_limits.max_storage_buffers_per_shader_stage);
    req.max_storage_textures_per_shader_stage = req
        .max_storage_textures_per_shader_stage
        .min(adapter_limits.max_storage_textures_per_shader_stage);
    req.max_uniform_buffers_per_shader_stage = req
        .max_uniform_buffers_per_shader_stage
        .min(adapter_limits.max_uniform_buffers_per_shader_stage);
    req.max_uniform_buffer_binding_size = req
        .max_uniform_buffer_binding_size
        .min(adapter_limits.max_uniform_buffer_binding_size);
    req.max_storage_buffer_binding_size = req
        .max_storage_buffer_binding_size
        .min(adapter_limits.max_storage_buffer_binding_size);
    req.max_buffer_size = adapter_limits.max_buffer_size;
    req.max_vertex_buffers = req
        .max_vertex_buffers
        .min(adapter_limits.max_vertex_buffers);
    req.max_vertex_attributes = req
        .max_vertex_attributes
        .min(adapter_limits.max_vertex_attributes);
    req.max_vertex_buffer_array_stride = req
        .max_vertex_buffer_array_stride
        .min(adapter_limits.max_vertex_buffer_array_stride);
    req.min_uniform_buffer_offset_alignment = req
        .min_uniform_buffer_offset_alignment
        .max(adapter_limits.min_uniform_buffer_offset_alignment);
    req.min_storage_buffer_offset_alignment = req
        .min_storage_buffer_offset_alignment
        .max(adapter_limits.min_storage_buffer_offset_alignment);
    req.max_color_attachments = req
        .max_color_attachments
        .min(adapter_limits.max_color_attachments);
    req.max_color_attachment_bytes_per_sample = req
        .max_color_attachment_bytes_per_sample
        .min(adapter_limits.max_color_attachment_bytes_per_sample);
    req.max_compute_workgroup_storage_size = req
        .max_compute_workgroup_storage_size
        .min(adapter_limits.max_compute_workgroup_storage_size);
    req.max_compute_invocations_per_workgroup = req
        .max_compute_invocations_per_workgroup
        .min(adapter_limits.max_compute_invocations_per_workgroup);
    req.max_compute_workgroup_size_x = req
        .max_compute_workgroup_size_x
        .min(adapter_limits.max_compute_workgroup_size_x);
    req.max_compute_workgroup_size_y = req
        .max_compute_workgroup_size_y
        .min(adapter_limits.max_compute_workgroup_size_y);
    req.max_compute_workgroup_size_z = req
        .max_compute_workgroup_size_z
        .min(adapter_limits.max_compute_workgroup_size_z);
    req.max_compute_workgroups_per_dimension = req
        .max_compute_workgroups_per_dimension
        .min(adapter_limits.max_compute_workgroups_per_dimension);
    req
}

impl GpuContext {
    /// Initializes an offscreen GPU context on the highest-ranked adapter matching `policy` and
    /// binds `tracker` as its unified quota authority.
    pub fn new(
        policy: &AdapterSelectionPolicy,
        tracker: &ResourceTracker,
    ) -> Result<Self, RenderError> {
        let raw_adapters = enumerate_raw_wgpu_adapters(policy);
        if raw_adapters.is_empty() {
            return Err(RenderError::NoSuitableAdapter);
        }

        let mut candidates = Vec::new();
        let mut first_capability_err: Option<RenderError> = None;

        for adapter in raw_adapters {
            let raw_profile = extract_raw_adapter_profile(&adapter);
            match evaluate_adapter_profile(&raw_profile, policy) {
                Ok(caps) => candidates.push((adapter, caps)),
                Err(RenderError::NoSuitableAdapter) => {}
                Err(cap_err) => {
                    if first_capability_err.is_none() {
                        first_capability_err = Some(cap_err);
                    }
                }
            }
        }

        if candidates.is_empty() {
            return Err(first_capability_err.unwrap_or(RenderError::NoSuitableAdapter));
        }

        candidates.sort_by_key(|(_, caps)| adapter_rank_key(caps, policy));
        let (adapter, mut capabilities) = candidates.remove(0);

        let adapter_limits = adapter.limits();
        let required_limits = clamp_device_limits(&adapter_limits);
        capabilities.max_texture_dimension_2d = required_limits.max_texture_dimension_2d;
        capabilities.max_buffer_size = required_limits.max_buffer_size;

        let desc = DeviceDescriptor {
            label: Some("dicomflux-3d-offscreen-device"),
            required_features: Features::empty(),
            required_limits,
            ..DeviceDescriptor::default()
        };

        let req_fut = adapter.request_device(&desc);
        let mut req_fut = pin!(req_fut);
        let mut cx = Context::from_waker(Waker::noop());
        let (device, queue) = match req_fut.as_mut().poll(&mut cx) {
            Poll::Ready(Ok(pair)) => pair,
            Poll::Ready(Err(_)) | Poll::Pending => {
                return Err(RenderError::NoSuitableAdapter);
            }
        };

        let fault_state = Arc::new(FaultLatch::new());

        let lost_latch = Arc::clone(&fault_state);
        device.set_device_lost_callback(move |_reason, _msg| {
            lost_latch.trip_device_lost();
        });

        let uncaptured_latch = Arc::clone(&fault_state);
        device.on_uncaptured_error(Arc::new(move |_err| {
            uncaptured_latch.trip_device_lost();
        }));

        let pipelines = CachedPipelines::build(&device)?;
        if !fault_state.is_healthy() {
            return Err(RenderError::GpuDeviceLost);
        }

        Ok(Self {
            device,
            queue,
            pipelines,
            capabilities,
            tracker: tracker.clone(),
            fault_state,
            submission_lock: Mutex::new(GpuMutableState::default()),
            #[cfg(test)]
            fault_injection: Mutex::new(FaultInjectionState::default()),
        })
    }

    #[must_use]
    pub const fn capabilities(&self) -> &GpuCapabilities {
        &self.capabilities
    }

    #[must_use]
    pub const fn tracker(&self) -> &ResourceTracker {
        &self.tracker
    }

    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.fault_state.is_healthy()
    }

    /// Steps non-blocking GPU completion polling and reaps any cancelled in-flight submissions
    /// whose hardware execution has completed.
    pub fn poll_reap(&self) {
        let _ = self.device.poll(PollType::Poll);
        if let Ok(mut state) = self.submission_lock.lock() {
            state
                .in_flight_ledger
                .retain(|job| !job.done.load(Ordering::Acquire));
        }
    }

    /// Evicts any cached single-slot resident GPU frame and reaps completed in-flight resources.
    pub fn evict_resident_frame(&self) {
        if let Ok(mut state) = self.submission_lock.lock() {
            state.resident = None;
        }
        self.poll_reap();
    }

    #[cfg(test)]
    fn arm_simulated_oom_once(&self) {
        if let Ok(mut inj) = self.fault_injection.lock() {
            inj.simulate_oom_once = true;
        }
    }

    #[cfg(test)]
    fn arm_simulated_device_loss_once(&self) {
        if let Ok(mut inj) = self.fault_injection.lock() {
            inj.simulate_device_loss_once = true;
        }
    }

    #[cfg(test)]
    fn arm_cancel_after_submit_once(&self, token: CancellationToken) {
        if let Ok(mut inj) = self.fault_injection.lock() {
            inj.cancel_after_submit_once = Some(token);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn retain_cancelled_in_flight(
    ctx: &GpuContext,
    done: Arc<AtomicBool>,
    frame: &CheckedFrame,
    resident: Arc<ResidentGpuFrame>,
    target_texture: Texture,
    uniform_buffer: Buffer,
    readback_buffer: Buffer,
    upload_buffer: Option<Buffer>,
    in_flight_guard: BudgetGuard,
    target_guard: BudgetGuard,
    readback_guard: BudgetGuard,
    upload_guard: Option<BudgetGuard>,
) {
    if done.load(Ordering::Acquire) {
        return;
    }

    let mut buffers = Vec::with_capacity(3);
    buffers.push(uniform_buffer);
    buffers.push(readback_buffer);
    if let Some(up_buf) = upload_buffer {
        buffers.push(up_buf);
    }

    let mut guards = Vec::with_capacity(4);
    guards.push(in_flight_guard);
    guards.push(target_guard);
    guards.push(readback_guard);
    if let Some(up_guard) = upload_guard {
        guards.push(up_guard);
    }

    let job = RetainedInFlightJob {
        done,
        _frame_lease: frame.lease().clone(),
        _source_budget: Arc::clone(frame.source_budget()),
        _resident: resident,
        _target_texture: target_texture,
        _buffers: buffers,
        _guards: guards,
    };

    if let Ok(mut state) = ctx.submission_lock.lock() {
        state.in_flight_ledger.push(job);
        state
            .in_flight_ledger
            .retain(|entry| !entry.done.load(Ordering::Acquire));
    }
}

/// Pops active `Validation` and `OutOfMemory` error scopes in strict reverse LIFO order on any
/// early error exit during command/buffer setup so real GPU `OutOfMemory` or `DeviceLost` faults
/// on `mapped_at_creation` buffers are never masked as `InternalError`.
fn abort_scoped_error(
    ctx: &GpuContext,
    resident: &mut Option<Arc<ResidentGpuFrame>>,
    val_scope: ErrorScopeGuard,
    oom_scope: ErrorScopeGuard,
    fallback_msg: &'static str,
) -> RenderError {
    let _val_err = pop_error_scope(&ctx.device, val_scope);
    let oom_err = pop_error_scope(&ctx.device, oom_scope);
    if !ctx.is_healthy() {
        *resident = None;
        return RenderError::GpuDeviceLost;
    }
    if oom_err.is_some() {
        return RenderError::GpuOutOfMemory;
    }
    RenderError::InternalError(fallback_msg)
}

/// Executes synchronous offscreen 16-bit greyscale presentation on `ctx`, reusing the single-slot
/// resident GPU texture across repeated window/level updates on the same
/// `(SourceIdentity, FrameDescriptor)`.
pub fn render_offscreen(
    ctx: &GpuContext,
    frame: &CheckedFrame,
    request: &VoiRequest,
    cancel: &CancellationToken,
) -> Result<RenderedImage, RenderError> {
    let total_start = Instant::now();
    let validation_start = Instant::now();

    if cancel.is_cancelled() {
        return Err(RenderError::Cancelled);
    }

    if !ctx.is_healthy() {
        ctx.evict_resident_frame();
        return Err(RenderError::GpuDeviceLost);
    }

    let (center, width_voi, slope, intercept, modality_unit) = request.validate()?;

    let width = frame.width();
    let height = frame.height();

    let _dim_w_guard = ctx
        .tracker
        .reserve(LimitPoolKind::FrameDimension, width as usize)?;
    let _dim_h_guard = ctx
        .tracker
        .reserve(LimitPoolKind::FrameDimension, height as usize)?;

    if width > ctx.capabilities.max_texture_dimension_2d
        || height > ctx.capabilities.max_texture_dimension_2d
    {
        return Err(RenderError::UnsupportedGpuCapability(
            "frame dimension exceeds adapter max_texture_dimension_2d",
        ));
    }

    let upload_layout = checked_upload_layout(width, height)?;
    let readback_layout = checked_readback_layout(width, height)?;

    if (upload_layout.total_buffer_bytes as u64) > ctx.capabilities.max_buffer_size
        || (readback_layout.total_buffer_bytes as u64) > ctx.capabilities.max_buffer_size
    {
        return Err(RenderError::UnsupportedGpuCapability(
            "padded staging buffer exceeds adapter max_buffer_size",
        ));
    }

    let out_w = width as usize;
    let out_h = height as usize;
    let pixel_count = out_w
        .checked_mul(out_h)
        .ok_or(RenderError::InvalidDescriptor(
            "output pixel count overflow",
        ))?;
    let source_tex_bytes = pixel_count
        .checked_mul(2)
        .ok_or(RenderError::InvalidDescriptor(
            "source texture byte size overflow",
        ))?;
    let target_tex_bytes = pixel_count;

    let in_flight_guard = ctx.tracker.reserve_job()?;
    let output_guard = ctx.tracker.reserve(LimitPoolKind::CpuOutput, pixel_count)?;
    let readback_guard = ctx.tracker.reserve(
        LimitPoolKind::GpuStagingReadback,
        readback_layout.total_buffer_bytes,
    )?;

    let uniform_block = VoiUniformBlock::solve(
        width,
        height,
        frame.pixel_padding(),
        request,
        center,
        width_voi,
        slope,
        intercept,
    );
    let uniform_bytes = uniform_block.to_le_bytes();
    let validation_dur = validation_start.elapsed();

    let encode_start = Instant::now();
    let (
        submission_index,
        job_done,
        active_resident,
        source_cache_hit,
        target_texture,
        uniform_buffer,
        readback_buffer,
        upload_buffer,
        target_guard,
        upload_guard,
    ) = {
        let mut state = ctx
            .submission_lock
            .lock()
            .map_err(|_| RenderError::InternalError("GPU submission lock poisoned"))?;

        state
            .in_flight_ledger
            .retain(|job| !job.done.load(Ordering::Acquire));

        if cancel.is_cancelled() {
            return Err(RenderError::Cancelled);
        }
        if !ctx.is_healthy() {
            state.resident = None;
            return Err(RenderError::GpuDeviceLost);
        }

        #[cfg(test)]
        let (sim_oom, sim_loss) = {
            let mut inj = ctx
                .fault_injection
                .lock()
                .map_err(|_| RenderError::InternalError("fault injection lock poisoned"))?;
            let oom = inj.simulate_oom_once;
            let loss = inj.simulate_device_loss_once;
            inj.simulate_oom_once = false;
            inj.simulate_device_loss_once = false;
            (oom, loss)
        };
        #[cfg(not(test))]
        let (sim_oom, sim_loss) = (false, false);

        if sim_loss {
            ctx.fault_state.trip_device_lost();
            state.resident = None;
            return Err(RenderError::GpuDeviceLost);
        }

        let key = (frame.identity(), *frame.descriptor());
        let cached_resident = match &state.resident {
            Some(existing) if existing.key == key => Some(Arc::clone(existing)),
            Some(_) => {
                // Retire-before-replace: release mismatched resident slot BEFORE reserving new GpuResident bytes.
                state.resident = None;
                None
            }
            None => None,
        };

        let target_guard = ctx
            .tracker
            .reserve(LimitPoolKind::GpuResident, target_tex_bytes)?;

        let (source_resident_guard, upload_guard) = if cached_resident.is_none() {
            let res_guard = Arc::new(
                ctx.tracker
                    .reserve(LimitPoolKind::GpuResident, source_tex_bytes)?,
            );
            let up_guard = ctx.tracker.reserve(
                LimitPoolKind::GpuStagingUpload,
                upload_layout.total_buffer_bytes,
            )?;
            (Some(res_guard), Some(up_guard))
        } else {
            (None, None)
        };

        let oom_scope = ctx.device.push_error_scope(ErrorFilter::OutOfMemory);
        let val_scope = ctx.device.push_error_scope(ErrorFilter::Validation);

        let mut encoder = ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("dicomflux-3d-offscreen-encoder"),
            });

        let (candidate_resident, source_cache_hit, upload_buffer) =
            if let Some(hit) = cached_resident {
                (hit, true, None)
            } else {
                let up_buffer = ctx.device.create_buffer(&BufferDescriptor {
                    label: Some("dicomflux-3d-upload-staging"),
                    size: upload_layout.total_buffer_bytes as u64,
                    usage: BufferUsages::COPY_SRC,
                    mapped_at_creation: true,
                });
                {
                    let Ok(mut view) = up_buffer.get_mapped_range_mut(..) else {
                        drop(encoder);
                        return Err(abort_scoped_error(
                            ctx,
                            &mut state.resident,
                            val_scope,
                            oom_scope,
                            "failed to map upload staging buffer",
                        ));
                    };
                    write_frame_to_upload_staging(frame, &upload_layout, &mut view);
                }
                up_buffer.unmap();

                let src_format = match frame.sample_type() {
                    SampleType::I16 => TextureFormat::R16Sint,
                    SampleType::U16 => TextureFormat::R16Uint,
                    _ => {
                        drop(encoder);
                        return Err(abort_scoped_error(
                            ctx,
                            &mut state.resident,
                            val_scope,
                            oom_scope,
                            "unexpected sample type admitted to GPU renderer",
                        ));
                    }
                };

                let src_texture = ctx.device.create_texture(&TextureDescriptor {
                    label: Some("dicomflux-3d-source-r16"),
                    size: Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: src_format,
                    usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let src_view = src_texture.create_view(&TextureViewDescriptor::default());

                encoder.copy_buffer_to_texture(
                    TexelCopyBufferInfo {
                        buffer: &up_buffer,
                        layout: TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(upload_layout.padded_bytes_per_row as u32),
                            rows_per_image: Some(height),
                        },
                    },
                    TexelCopyTextureInfo {
                        texture: &src_texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );

                let Some(res_budget) = source_resident_guard else {
                    drop(encoder);
                    return Err(abort_scoped_error(
                        ctx,
                        &mut state.resident,
                        val_scope,
                        oom_scope,
                        "missing source resident budget guard on cache miss",
                    ));
                };
                let candidate = Arc::new(ResidentGpuFrame {
                    key,
                    sample_type: frame.sample_type(),
                    _texture: src_texture,
                    view: src_view,
                    _budget: res_budget,
                });
                (candidate, false, Some(up_buffer))
            };

        let uniform_buffer = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("dicomflux-3d-voi-uniforms"),
            size: 64,
            usage: BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });
        {
            let Ok(mut uview) = uniform_buffer.get_mapped_range_mut(..) else {
                drop(encoder);
                return Err(abort_scoped_error(
                    ctx,
                    &mut state.resident,
                    val_scope,
                    oom_scope,
                    "failed to map VOI uniform buffer",
                ));
            };
            uview.copy_from_slice(&uniform_bytes);
        }
        uniform_buffer.unmap();

        let target_texture = ctx.device.create_texture(&TextureDescriptor {
            label: Some("dicomflux-3d-target-r8uint"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Uint,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target_texture.create_view(&TextureViewDescriptor::default());

        let readback_buffer = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("dicomflux-3d-readback-staging"),
            size: readback_layout.total_buffer_bytes as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (bind_group, pipeline) = match candidate_resident.sample_type {
            SampleType::I16 => {
                let bg = ctx.device.create_bind_group(&BindGroupDescriptor {
                    label: Some("dicomflux-3d-sint-bg"),
                    layout: &ctx.pipelines.sint_bgl,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: uniform_buffer.as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&candidate_resident.view),
                        },
                    ],
                });
                (bg, &ctx.pipelines.sint_pipeline)
            }
            SampleType::U16 => {
                let bg = ctx.device.create_bind_group(&BindGroupDescriptor {
                    label: Some("dicomflux-3d-uint-bg"),
                    layout: &ctx.pipelines.uint_bgl,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: uniform_buffer.as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::TextureView(&candidate_resident.view),
                        },
                    ],
                });
                (bg, &ctx.pipelines.uint_pipeline)
            }
            _ => {
                drop(encoder);
                return Err(abort_scoped_error(
                    ctx,
                    &mut state.resident,
                    val_scope,
                    oom_scope,
                    "invalid resident sample type in GPU pass",
                ));
            }
        };

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("dicomflux-3d-voi-pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            rpass.set_pipeline(pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: &target_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &readback_buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(readback_layout.padded_bytes_per_row as u32),
                    rows_per_image: Some(height),
                },
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        if sim_oom {
            drop(encoder);
            let _ = pop_error_scope(&ctx.device, val_scope);
            let _ = pop_error_scope(&ctx.device, oom_scope);
            return Err(RenderError::GpuOutOfMemory);
        }

        let submission_index = ctx.queue.submit(std::iter::once(encoder.finish()));
        let job_done = Arc::new(AtomicBool::new(false));
        let job_done_cb = Arc::clone(&job_done);
        ctx.queue.on_submitted_work_done(move || {
            job_done_cb.store(true, Ordering::Release);
        });

        // Strict reverse LIFO order: pop Validation (pushed second) BEFORE OutOfMemory (pushed first).
        let val_err = pop_error_scope(&ctx.device, val_scope);
        let oom_err = pop_error_scope(&ctx.device, oom_scope);

        if !ctx.is_healthy() {
            state.resident = None;
            return Err(RenderError::GpuDeviceLost);
        }
        if oom_err.is_some() {
            return Err(RenderError::GpuOutOfMemory);
        }
        if val_err.is_some() {
            return Err(RenderError::InternalError(
                "GPU command encoding or submission failed validation",
            ));
        }

        if !source_cache_hit {
            state.resident = Some(Arc::clone(&candidate_resident));
        }

        (
            submission_index,
            job_done,
            candidate_resident,
            source_cache_hit,
            target_texture,
            uniform_buffer,
            readback_buffer,
            upload_buffer,
            target_guard,
            upload_guard,
        )
    };

    let encode_dur = encode_start.elapsed();
    let wait_start = Instant::now();

    #[cfg(test)]
    {
        if let Ok(mut inj) = ctx.fault_injection.lock()
            && let Some(token) = inj.cancel_after_submit_once.take()
        {
            token.cancel();
        }
    }

    let map_state = Arc::new(AtomicU8::new(MAP_PENDING));
    let map_state_cb = Arc::clone(&map_state);
    readback_buffer
        .slice(..)
        .map_async(MapMode::Read, move |res| {
            let status = match res {
                Ok(()) => MAP_OK,
                Err(_) => MAP_ERR,
            };
            map_state_cb.store(status, Ordering::Release);
        });

    loop {
        if cancel.is_cancelled() {
            retain_cancelled_in_flight(
                ctx,
                job_done,
                frame,
                active_resident,
                target_texture,
                uniform_buffer,
                readback_buffer,
                upload_buffer,
                in_flight_guard,
                target_guard,
                readback_guard,
                upload_guard,
            );
            return Err(RenderError::Cancelled);
        }

        if !ctx.is_healthy() {
            ctx.evict_resident_frame();
            return Err(RenderError::GpuDeviceLost);
        }

        match map_state.load(Ordering::Acquire) {
            MAP_OK => break,
            MAP_ERR => {
                if !ctx.is_healthy() {
                    ctx.evict_resident_frame();
                    return Err(RenderError::GpuDeviceLost);
                }
                return Err(RenderError::InternalError(
                    "GPU readback staging buffer mapping failed",
                ));
            }
            _ => {}
        }

        match ctx.device.poll(PollType::Wait {
            submission_index: Some(submission_index.clone()),
            timeout: Some(Duration::from_millis(10)),
        }) {
            Ok(_) | Err(PollError::Timeout) => {}
            Err(_) => {
                if !ctx.is_healthy() {
                    ctx.evict_resident_frame();
                    return Err(RenderError::GpuDeviceLost);
                }
                return Err(RenderError::InternalError(
                    "GPU device wait poll failed unexpectedly",
                ));
            }
        }
    }

    let mut pixels = Vec::with_capacity(pixel_count);
    {
        let mapped = readback_buffer
            .slice(..)
            .get_mapped_range()
            .map_err(|_| RenderError::InternalError("failed to inspect mapped readback slice"))?;
        unpack_readback_to_tight(&mapped, &readback_layout, &mut pixels)?;
    }
    readback_buffer.unmap();

    let staging_upload_reserved = if source_cache_hit {
        0
    } else {
        upload_layout.total_buffer_bytes
    };
    let source_bytes_uploaded = if source_cache_hit {
        0
    } else {
        source_tex_bytes
    };

    drop(upload_buffer);
    drop(upload_guard);
    drop(uniform_buffer);
    drop(target_texture);
    drop(target_guard);
    drop(readback_buffer);
    drop(readback_guard);
    drop(in_flight_guard);

    let wait_dur = wait_start.elapsed();
    let total_dur = total_start.elapsed();

    let output = OutputDescriptor {
        width,
        height,
        row_stride_bytes: out_w,
        bits_per_pixel: 8,
        polarity: request.polarity,
        display_encoding: DisplayEncoding::UncalibratedSrgbU8,
    };

    let receipt = RenderReceipt {
        operation_id: ctx.tracker.next_operation_id(),
        source_identity: frame.identity(),
        path: PresentationPath::GpuOffscreen,
        backend: Some(ctx.capabilities.backend),
        qualification_tier: ctx.capabilities.qualification_tier,
        shader_sha256: Some(Arc::clone(&ctx.pipelines.shader_sha256)),
        output,
        voi_function: VoiFunction::Linear,
        applied_center: center,
        applied_width: width_voi,
        applied_slope: slope,
        applied_intercept: intercept,
        modality_unit,
        polarity: request.polarity,
        padding_applied: !matches!(frame.pixel_padding(), PixelPadding::None),
        padding_display_code: request.padding_display_code,
        rounding_policy: ROUNDING_POLICY_TIES_EVEN_U8,
        max_gpu_vs_cpu_interior_lsb_tolerance: MAX_GPU_VS_CPU_INTERIOR_LSB_TOLERANCE,
        upload: UploadAccounting {
            source_cache_hit,
            source_bytes_uploaded,
            staging_upload_bytes_reserved: staging_upload_reserved,
            staging_readback_bytes_reserved: readback_layout.total_buffer_bytes,
        },
        timings: StageTimings {
            validation: validation_dur,
            host_encode_and_submit: encode_dur,
            gpu_wait_and_unpack: wait_dur,
            total: total_dur,
        },
    };

    Ok(RenderedImage::new(pixels, output_guard, output, receipt))
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use super::*;
    use crate::image::{BitPrecision, admit_frame};

    fn make_test_frame(tracker: &ResourceTracker, generation: u64) -> CheckedFrame {
        let width = 32u32;
        let height = 32u32;
        let words: Vec<i16> = (0..(width * height) as i16)
            .map(|i| (i % 1024) - 512)
            .collect();
        let identity = SourceIdentity {
            owner_id: NonZeroU64::new(77).expect("nonzero"),
            generation,
        };
        let lease = DecodedFrameLease::from_vec_i16(words, identity).expect("valid i16 lease");
        let desc = FrameDescriptor::new(
            width,
            height,
            (width as usize) * 2,
            SampleType::I16,
            BitPrecision::new(16, 12, 11),
        );
        admit_frame(desc, lease, tracker).expect("valid frame")
    }

    #[test]
    fn simulated_oom_discards_uncommitted_resident_and_releases_quotas() {
        let tracker = ResourceTracker::with_default_limits();
        let policy = AdapterSelectionPolicy::new(AllowedBackends::all_supported(), true, true);
        let Ok(ctx) = GpuContext::new(&policy, &tracker) else {
            return;
        };

        let frame = make_test_frame(&tracker, 1);
        let req = VoiRequest::linear(0.0, 400.0);
        let cancel = CancellationToken::new();

        ctx.arm_simulated_oom_once();
        let err = render_offscreen(&ctx, &frame, &req, &cancel).expect_err("simulated OOM");
        assert_eq!(err, RenderError::GpuOutOfMemory);
        assert!(ctx.is_healthy());

        let snap = tracker.snapshot();
        assert_eq!(snap.gpu_resident_bytes, 0);
        assert_eq!(snap.gpu_staging_upload_bytes, 0);
        assert_eq!(snap.gpu_staging_readback_bytes, 0);
        assert_eq!(snap.cpu_output_bytes, 0);
        assert_eq!(snap.in_flight_jobs, 0);

        // Next call succeeds as a cold upload because OOM never committed residency.
        let rendered = render_offscreen(&ctx, &frame, &req, &cancel).expect("recovery succeeds");
        assert!(!rendered.receipt().upload.source_cache_hit);
    }

    #[test]
    fn simulated_device_loss_latches_context_and_clears_residency() {
        let tracker = ResourceTracker::with_default_limits();
        let policy = AdapterSelectionPolicy::new(AllowedBackends::all_supported(), true, true);
        let Ok(ctx) = GpuContext::new(&policy, &tracker) else {
            return;
        };

        let frame = make_test_frame(&tracker, 2);
        let req = VoiRequest::linear(50.0, 200.0);
        let cancel = CancellationToken::new();

        let _warm = render_offscreen(&ctx, &frame, &req, &cancel).expect("initial warm render");
        assert!(tracker.snapshot().gpu_resident_bytes > 0);

        ctx.arm_simulated_device_loss_once();
        let err = render_offscreen(&ctx, &frame, &req, &cancel).expect_err("simulated device loss");
        assert_eq!(err, RenderError::GpuDeviceLost);
        assert!(!ctx.is_healthy());
        assert_eq!(tracker.snapshot().gpu_resident_bytes, 0);
        assert_eq!(tracker.snapshot().in_flight_jobs, 0);

        // Terminal latch refuses subsequent calls deterministically.
        let err_after = render_offscreen(&ctx, &frame, &req, &cancel).expect_err("remains lost");
        assert_eq!(err_after, RenderError::GpuDeviceLost);
    }

    #[test]
    fn cancel_after_submit_retains_in_flight_resources_until_poll_reap() {
        let tracker = ResourceTracker::with_default_limits();
        let policy = AdapterSelectionPolicy::new(AllowedBackends::all_supported(), true, true);
        let Ok(ctx) = GpuContext::new(&policy, &tracker) else {
            return;
        };

        let frame = make_test_frame(&tracker, 3);
        let req = VoiRequest::linear(10.0, 300.0);
        let cancel = CancellationToken::new();

        ctx.arm_cancel_after_submit_once(cancel.clone());
        let err = render_offscreen(&ctx, &frame, &req, &cancel)
            .expect_err("post-submit cancel must return Cancelled");
        assert_eq!(err, RenderError::Cancelled);
        assert_eq!(tracker.snapshot().cpu_output_bytes, 0);

        // Drop caller's CheckedFrame handle while GPU work may still be retiring.
        drop(frame);

        let deadline = Instant::now() + Duration::from_secs(3);
        while tracker.snapshot().in_flight_jobs > 0 && Instant::now() < deadline {
            ctx.poll_reap();
            std::thread::sleep(Duration::from_millis(1));
        }

        ctx.evict_resident_frame();
        let snap = tracker.snapshot();
        assert_eq!(snap.in_flight_jobs, 0);
        assert_eq!(snap.retained_source_bytes, 0);
        assert_eq!(snap.gpu_staging_upload_bytes, 0);
        assert_eq!(snap.gpu_resident_bytes, 0);
        assert_eq!(snap.gpu_staging_readback_bytes, 0);
        assert_eq!(snap.cpu_output_bytes, 0);
    }

    #[test]
    fn abort_scoped_error_pops_lifo_and_classifies_device_loss_or_fallback() {
        let tracker = ResourceTracker::with_default_limits();
        let policy = AdapterSelectionPolicy::new(AllowedBackends::all_supported(), true, true);
        let Ok(ctx) = GpuContext::new(&policy, &tracker) else {
            return;
        };

        let frame = make_test_frame(&tracker, 4);
        let req = VoiRequest::linear(0.0, 200.0);
        let cancel = CancellationToken::new();
        let _warm = render_offscreen(&ctx, &frame, &req, &cancel).expect("warm render");

        // Healthy device with clean scopes returns the fallback InternalError without evicting.
        {
            let mut state = ctx.submission_lock.lock().expect("lock");
            let oom_scope = ctx.device.push_error_scope(ErrorFilter::OutOfMemory);
            let val_scope = ctx.device.push_error_scope(ErrorFilter::Validation);
            let err = abort_scoped_error(
                &ctx,
                &mut state.resident,
                val_scope,
                oom_scope,
                "fallback-msg",
            );
            assert_eq!(err, RenderError::InternalError("fallback-msg"));
            assert!(state.resident.is_some());
        }

        // Latched device loss pops scopes in LIFO order, clears residency, and returns GpuDeviceLost.
        {
            ctx.fault_state.trip_device_lost();
            let mut state = ctx.submission_lock.lock().expect("lock");
            let oom_scope = ctx.device.push_error_scope(ErrorFilter::OutOfMemory);
            let val_scope = ctx.device.push_error_scope(ErrorFilter::Validation);
            let err = abort_scoped_error(
                &ctx,
                &mut state.resident,
                val_scope,
                oom_scope,
                "fallback-msg",
            );
            assert_eq!(err, RenderError::GpuDeviceLost);
            assert!(state.resident.is_none());
        }
        assert_eq!(tracker.snapshot().gpu_resident_bytes, 0);
    }
}
