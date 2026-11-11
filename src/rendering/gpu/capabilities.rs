// Copyright (c) 2026 Raster Images
// SPDX-License-Identifier: MIT

use std::fmt;
use std::future::Future as _;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use crate::error::RenderError;
use crate::presentation::{GpuBackendKind, QualificationTier, host_cpu_qualification_tier};

/// Explicit set of native desktop GPU APIs permitted during adapter probing and selection.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllowedBackends {
    pub metal: bool,
    pub vulkan: bool,
    pub dx12: bool,
}

impl AllowedBackends {
    /// Permits all desktop backends supported by `FS-01` (`Metal`, `Vulkan`, and `Dx12`).
    #[must_use]
    pub const fn all_supported() -> Self {
        Self {
            metal: true,
            vulkan: true,
            dx12: true,
        }
    }

    /// Disallows every GPU backend, deterministically forcing [`RenderError::NoSuitableAdapter`].
    #[must_use]
    pub const fn none() -> Self {
        Self {
            metal: false,
            vulkan: false,
            dx12: false,
        }
    }

    #[must_use]
    pub const fn new(metal: bool, vulkan: bool, dx12: bool) -> Self {
        Self {
            metal,
            vulkan,
            dx12,
        }
    }

    pub(crate) fn to_wgpu_backends(self) -> wgpu::Backends {
        let mut backends = wgpu::Backends::empty();
        if self.metal {
            backends |= wgpu::Backends::METAL;
        }
        if self.vulkan {
            backends |= wgpu::Backends::VULKAN;
        }
        if self.dx12 {
            backends |= wgpu::Backends::DX12;
        }
        backends
    }

    pub(crate) const fn allows(self, backend: GpuBackendKind) -> bool {
        match backend {
            GpuBackendKind::Metal => self.metal,
            GpuBackendKind::Vulkan => self.vulkan,
            GpuBackendKind::Dx12 => self.dx12,
            GpuBackendKind::Other => false,
        }
    }
}

impl Default for AllowedBackends {
    fn default() -> Self {
        Self::all_supported()
    }
}

/// Hardware or software device classification reported by an enumerated GPU adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdapterDeviceType {
    DiscreteGpu,
    IntegratedGpu,
    VirtualGpu,
    CpuSoftware,
    Other,
}

/// Explicit policy governing GPU adapter filtering and ranking.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdapterSelectionPolicy {
    pub allowed_backends: AllowedBackends,
    pub allow_software_adapter: bool,
    pub prefer_discrete_gpu: bool,
}

impl AdapterSelectionPolicy {
    #[must_use]
    pub const fn new(
        allowed_backends: AllowedBackends,
        allow_software_adapter: bool,
        prefer_discrete_gpu: bool,
    ) -> Self {
        Self {
            allowed_backends,
            allow_software_adapter,
            prefer_discrete_gpu,
        }
    }
}

impl Default for AdapterSelectionPolicy {
    fn default() -> Self {
        Self {
            allowed_backends: AllowedBackends::all_supported(),
            allow_software_adapter: false,
            prefer_discrete_gpu: true,
        }
    }
}

/// Privacy-redacted capability summary for an admitted GPU adapter.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GpuCapabilities {
    pub backend: GpuBackendKind,
    pub device_type: AdapterDeviceType,
    pub qualification_tier: QualificationTier,
    pub max_texture_dimension_2d: u32,
    pub max_buffer_size: u64,
    pub supports_16bit_grayscale_offscreen: bool,
}

impl fmt::Debug for GpuCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GpuCapabilities")
            .field("backend", &self.backend)
            .field("device_type", &self.device_type)
            .field("qualification_tier", &self.qualification_tier)
            .field("max_texture_dimension_2d", &self.max_texture_dimension_2d)
            .field("max_buffer_size", &self.max_buffer_size)
            .field(
                "supports_16bit_grayscale_offscreen",
                &self.supports_16bit_grayscale_offscreen,
            )
            .field("driver_details", &"<redacted>")
            .finish()
    }
}

/// Profiles explicitly excluded from `FS-01` along with their typed rejection or deferral reason.
pub(crate) const EXCLUDED_FS01_PROFILES: &[&str] = &[
    "multiframe_inputs (frame_count > 1): UnsupportedPixelFormat",
    "non_16bit_sample_containers (u8, i32, f32): UnsupportedPixelFormat",
    "color_and_palette_photometrics: UnsupportedPresentation",
    "voi_linear_exact_and_sigmoid: UnsupportedPresentation",
    "nonlinear_modality_and_voi_lut_sequences: UnsupportedPresentation",
    "gsps_and_gsdf_calibrated_display: UnsupportedPresentation",
    "mpr_mip_dvr_3d_volume_rendering: deferred_beyond_fs01",
    "segmentation_overlays_and_surface_meshes: deferred_beyond_fs01",
    "window_surfaces_and_browser_webgpu: deferred_beyond_fs01",
];

/// Pre-work host and GPU capability report returned by [`probe_capabilities`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityReport {
    pub cpu_reference_available: bool,
    pub host_qualification_tier: QualificationTier,
    pub gpu_adapters: Vec<GpuCapabilities>,
    pub excluded_profiles: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RawAdapterProfile {
    pub backend: GpuBackendKind,
    pub device_type: AdapterDeviceType,
    pub host_os: &'static str,
    pub host_arch: &'static str,
    pub max_texture_dimension_2d: u32,
    pub max_buffer_size: u64,
    pub supports_r16_sint_sampled_copy_dst: bool,
    pub supports_r16_uint_sampled_copy_dst: bool,
    pub supports_r8_uint_render_copy_src: bool,
}

/// Pure, deterministic evaluator mapping an adapter profile and host target triple to
/// [`GpuCapabilities`] or a typed [`RenderError`].
pub(crate) fn evaluate_adapter_profile(
    profile: &RawAdapterProfile,
    policy: &AdapterSelectionPolicy,
) -> Result<GpuCapabilities, RenderError> {
    if !policy.allowed_backends.allows(profile.backend) {
        return Err(RenderError::NoSuitableAdapter);
    }

    if profile.device_type == AdapterDeviceType::CpuSoftware && !policy.allow_software_adapter {
        return Err(RenderError::NoSuitableAdapter);
    }

    if !profile.supports_r16_sint_sampled_copy_dst {
        return Err(RenderError::UnsupportedGpuCapability(
            "adapter missing R16Sint TEXTURE_BINDING | COPY_DST support",
        ));
    }

    if !profile.supports_r16_uint_sampled_copy_dst {
        return Err(RenderError::UnsupportedGpuCapability(
            "adapter missing R16Uint TEXTURE_BINDING | COPY_DST support",
        ));
    }

    if !profile.supports_r8_uint_render_copy_src {
        return Err(RenderError::UnsupportedGpuCapability(
            "adapter missing R8Uint RENDER_ATTACHMENT | COPY_SRC support",
        ));
    }

    if profile.max_texture_dimension_2d == 0 || profile.max_buffer_size < 256 {
        return Err(RenderError::UnsupportedGpuCapability(
            "adapter limits below minimum FS-01 offscreen requirements",
        ));
    }

    let qualification_tier = if profile.device_type == AdapterDeviceType::CpuSoftware {
        QualificationTier::SoftwareAdapter
    } else if (profile.host_os == "windows"
        && profile.host_arch == "x86_64"
        && matches!(
            profile.backend,
            GpuBackendKind::Dx12 | GpuBackendKind::Vulkan
        ))
        || (profile.host_os == "linux"
            && profile.host_arch == "x86_64"
            && profile.backend == GpuBackendKind::Vulkan)
    {
        QualificationTier::PrimaryTargetCandidate
    } else {
        QualificationTier::BoundedDevelopmentHost
    };

    Ok(GpuCapabilities {
        backend: profile.backend,
        device_type: profile.device_type,
        qualification_tier,
        max_texture_dimension_2d: profile.max_texture_dimension_2d,
        max_buffer_size: profile.max_buffer_size,
        supports_16bit_grayscale_offscreen: true,
    })
}

pub(crate) const fn adapter_rank_key(
    caps: &GpuCapabilities,
    policy: &AdapterSelectionPolicy,
) -> (u8, u8) {
    let device_rank = match (policy.prefer_discrete_gpu, caps.device_type) {
        (true, AdapterDeviceType::DiscreteGpu) => 0,
        (true, AdapterDeviceType::IntegratedGpu) => 1,
        (false, AdapterDeviceType::IntegratedGpu) => 0,
        (false, AdapterDeviceType::DiscreteGpu) => 1,
        (_, AdapterDeviceType::VirtualGpu) => 2,
        (_, AdapterDeviceType::Other) => 3,
        (_, AdapterDeviceType::CpuSoftware) => 4,
    };
    let backend_rank = match caps.backend {
        GpuBackendKind::Dx12 => 0,
        GpuBackendKind::Vulkan => 1,
        GpuBackendKind::Metal => 2,
        GpuBackendKind::Other => 3,
    };
    (device_rank, backend_rank)
}

fn map_wgpu_backend(backend: wgpu::Backend) -> GpuBackendKind {
    match backend {
        wgpu::Backend::Metal => GpuBackendKind::Metal,
        wgpu::Backend::Vulkan => GpuBackendKind::Vulkan,
        wgpu::Backend::Dx12 => GpuBackendKind::Dx12,
        _ => GpuBackendKind::Other,
    }
}

fn map_wgpu_device_type(device_type: wgpu::DeviceType) -> AdapterDeviceType {
    match device_type {
        wgpu::DeviceType::DiscreteGpu => AdapterDeviceType::DiscreteGpu,
        wgpu::DeviceType::IntegratedGpu => AdapterDeviceType::IntegratedGpu,
        wgpu::DeviceType::VirtualGpu => AdapterDeviceType::VirtualGpu,
        wgpu::DeviceType::Cpu => AdapterDeviceType::CpuSoftware,
        wgpu::DeviceType::Other => AdapterDeviceType::Other,
    }
}

pub(crate) fn extract_raw_adapter_profile(adapter: &wgpu::Adapter) -> RawAdapterProfile {
    let info = adapter.get_info();
    let limits = adapter.limits();

    let src_usages = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
    let dst_usages = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC;

    let r16_sint = adapter
        .get_texture_format_features(wgpu::TextureFormat::R16Sint)
        .allowed_usages
        .contains(src_usages);
    let r16_uint = adapter
        .get_texture_format_features(wgpu::TextureFormat::R16Uint)
        .allowed_usages
        .contains(src_usages);
    let r8_uint = adapter
        .get_texture_format_features(wgpu::TextureFormat::R8Uint)
        .allowed_usages
        .contains(dst_usages);

    RawAdapterProfile {
        backend: map_wgpu_backend(info.backend),
        device_type: map_wgpu_device_type(info.device_type),
        host_os: std::env::consts::OS,
        host_arch: std::env::consts::ARCH,
        max_texture_dimension_2d: limits.max_texture_dimension_2d,
        max_buffer_size: limits.max_buffer_size,
        supports_r16_sint_sampled_copy_dst: r16_sint,
        supports_r16_uint_sampled_copy_dst: r16_uint,
        supports_r8_uint_render_copy_src: r8_uint,
    }
}

pub(crate) fn enumerate_raw_wgpu_adapters(policy: &AdapterSelectionPolicy) -> Vec<wgpu::Adapter> {
    let requested = policy.allowed_backends.to_wgpu_backends();
    let available = requested & wgpu::Instance::enabled_backend_features();
    if available.is_empty() {
        return Vec::new();
    }

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: available,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });

    let fut = instance.enumerate_adapters(available);
    let mut fut = pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(adapters) => adapters,
        Poll::Pending => Vec::new(),
    }
}

/// Probes host CPU and GPU offscreen presentation capabilities without panicking when zero GPU
/// adapters match `policy`.
#[must_use]
pub fn probe_capabilities(policy: &AdapterSelectionPolicy) -> CapabilityReport {
    let raw_adapters = enumerate_raw_wgpu_adapters(policy);
    let mut admitted: Vec<GpuCapabilities> = raw_adapters
        .iter()
        .filter_map(|adapter| {
            let raw = extract_raw_adapter_profile(adapter);
            evaluate_adapter_profile(&raw, policy).ok()
        })
        .collect();

    admitted.sort_by_key(|caps| adapter_rank_key(caps, policy));

    CapabilityReport {
        cpu_reference_available: true,
        host_qualification_tier: host_cpu_qualification_tier(),
        gpu_adapters: admitted,
        excluded_profiles: EXCLUDED_FS01_PROFILES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_profile() -> RawAdapterProfile {
        RawAdapterProfile {
            backend: GpuBackendKind::Vulkan,
            device_type: AdapterDeviceType::DiscreteGpu,
            host_os: "linux",
            host_arch: "x86_64",
            max_texture_dimension_2d: 16_384,
            max_buffer_size: 256 * 1024 * 1024,
            supports_r16_sint_sampled_copy_dst: true,
            supports_r16_uint_sampled_copy_dst: true,
            supports_r8_uint_render_copy_src: true,
        }
    }

    #[test]
    fn windows_and_linux_x86_64_hardware_stamp_primary_target_candidate() {
        let policy = AdapterSelectionPolicy::default();

        let linux_vk = baseline_profile();
        let caps_linux = evaluate_adapter_profile(&linux_vk, &policy).expect("linux vulkan valid");
        assert_eq!(
            caps_linux.qualification_tier,
            QualificationTier::PrimaryTargetCandidate
        );

        let win_dx12 = RawAdapterProfile {
            backend: GpuBackendKind::Dx12,
            host_os: "windows",
            ..baseline_profile()
        };
        let caps_dx12 = evaluate_adapter_profile(&win_dx12, &policy).expect("win dx12 valid");
        assert_eq!(
            caps_dx12.qualification_tier,
            QualificationTier::PrimaryTargetCandidate
        );

        let win_vk = RawAdapterProfile {
            backend: GpuBackendKind::Vulkan,
            host_os: "windows",
            ..baseline_profile()
        };
        let caps_win_vk = evaluate_adapter_profile(&win_vk, &policy).expect("win vulkan valid");
        assert_eq!(
            caps_win_vk.qualification_tier,
            QualificationTier::PrimaryTargetCandidate
        );
    }

    #[test]
    fn macos_metal_and_arm64_targets_stamp_bounded_development_host() {
        let policy = AdapterSelectionPolicy::default();

        let mac_metal = RawAdapterProfile {
            backend: GpuBackendKind::Metal,
            device_type: AdapterDeviceType::IntegratedGpu,
            host_os: "macos",
            host_arch: "aarch64",
            ..baseline_profile()
        };
        let caps_mac = evaluate_adapter_profile(&mac_metal, &policy).expect("mac metal valid");
        assert_eq!(
            caps_mac.qualification_tier,
            QualificationTier::BoundedDevelopmentHost
        );

        let linux_arm64 = RawAdapterProfile {
            backend: GpuBackendKind::Vulkan,
            host_os: "linux",
            host_arch: "aarch64",
            ..baseline_profile()
        };
        let caps_arm = evaluate_adapter_profile(&linux_arm64, &policy).expect("linux arm64 valid");
        assert_eq!(
            caps_arm.qualification_tier,
            QualificationTier::BoundedDevelopmentHost
        );

        let win_arm64 = RawAdapterProfile {
            backend: GpuBackendKind::Dx12,
            host_os: "windows",
            host_arch: "aarch64",
            ..baseline_profile()
        };
        let caps_win_arm = evaluate_adapter_profile(&win_arm64, &policy).expect("win arm64 valid");
        assert_eq!(
            caps_win_arm.qualification_tier,
            QualificationTier::BoundedDevelopmentHost
        );
    }

    #[test]
    fn software_adapter_obeys_policy_and_stamps_software_tier() {
        let sw_profile = RawAdapterProfile {
            device_type: AdapterDeviceType::CpuSoftware,
            ..baseline_profile()
        };

        let strict_policy = AdapterSelectionPolicy::default();
        assert_eq!(
            evaluate_adapter_profile(&sw_profile, &strict_policy),
            Err(RenderError::NoSuitableAdapter)
        );

        let sw_policy = AdapterSelectionPolicy::new(AllowedBackends::all_supported(), true, true);
        let caps = evaluate_adapter_profile(&sw_profile, &sw_policy).expect("software admitted");
        assert_eq!(caps.qualification_tier, QualificationTier::SoftwareAdapter);
    }

    #[test]
    fn disallowed_backends_and_missing_formats_fail_closed() {
        let profile = baseline_profile();
        let no_backends = AdapterSelectionPolicy::new(AllowedBackends::none(), true, true);
        assert_eq!(
            evaluate_adapter_profile(&profile, &no_backends),
            Err(RenderError::NoSuitableAdapter)
        );

        let policy = AdapterSelectionPolicy::default();

        let no_sint = RawAdapterProfile {
            supports_r16_sint_sampled_copy_dst: false,
            ..profile
        };
        assert!(matches!(
            evaluate_adapter_profile(&no_sint, &policy),
            Err(RenderError::UnsupportedGpuCapability(_))
        ));

        let no_uint = RawAdapterProfile {
            supports_r16_uint_sampled_copy_dst: false,
            ..profile
        };
        assert!(matches!(
            evaluate_adapter_profile(&no_uint, &policy),
            Err(RenderError::UnsupportedGpuCapability(_))
        ));

        let no_r8 = RawAdapterProfile {
            supports_r8_uint_render_copy_src: false,
            ..profile
        };
        assert!(matches!(
            evaluate_adapter_profile(&no_r8, &policy),
            Err(RenderError::UnsupportedGpuCapability(_))
        ));

        let tiny_buf = RawAdapterProfile {
            max_buffer_size: 128,
            ..profile
        };
        assert!(matches!(
            evaluate_adapter_profile(&tiny_buf, &policy),
            Err(RenderError::UnsupportedGpuCapability(_))
        ));
    }

    #[test]
    fn discrete_vs_integrated_ranking_follows_preference() {
        let disc = GpuCapabilities {
            backend: GpuBackendKind::Vulkan,
            device_type: AdapterDeviceType::DiscreteGpu,
            qualification_tier: QualificationTier::PrimaryTargetCandidate,
            max_texture_dimension_2d: 8192,
            max_buffer_size: 1024,
            supports_16bit_grayscale_offscreen: true,
        };
        let integ = GpuCapabilities {
            device_type: AdapterDeviceType::IntegratedGpu,
            ..disc
        };

        let prefer_disc =
            AdapterSelectionPolicy::new(AllowedBackends::all_supported(), false, true);
        assert!(adapter_rank_key(&disc, &prefer_disc) < adapter_rank_key(&integ, &prefer_disc));

        let prefer_integ =
            AdapterSelectionPolicy::new(AllowedBackends::all_supported(), false, false);
        assert!(adapter_rank_key(&integ, &prefer_integ) < adapter_rank_key(&disc, &prefer_integ));

        let dbg = format!("{disc:?}");
        assert!(dbg.contains("<redacted>"));
    }
}
