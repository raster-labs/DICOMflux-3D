use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use crate::error::{LimitPoolKind, RenderError};

const MIB: usize = 1024 * 1024;

/// Governing byte, dimension, and concurrency ceilings enforced by `ResourceTracker`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Maximum retained CPU backing capacity across live `CheckedFrame` instances (`64 MiB` default).
    pub retained_source_bytes: usize,
    /// Maximum active GPU upload staging bytes (`32 MiB` default).
    pub gpu_staging_upload_bytes: usize,
    /// Maximum active GPU resident source and target texture bytes (`64 MiB` default).
    pub gpu_resident_bytes: usize,
    /// Maximum active GPU readback staging bytes (`32 MiB` default).
    pub gpu_staging_readback_bytes: usize,
    /// Maximum active CPU output pixel buffer bytes (`32 MiB` default).
    pub cpu_output_bytes: usize,
    /// Maximum regular 3D volume byte extent admitted by `admit_volume` (`512 MiB` default).
    pub max_volume_bytes: usize,
    /// Maximum 2D frame width or height (`8192` default).
    pub max_frame_dim: u32,
    /// Maximum 3D volume column, row, or slice count (`2048` default).
    pub max_volume_dim: u32,
    /// Maximum concurrent CPU or GPU rendering jobs in flight (`4` default).
    pub max_in_flight_jobs: usize,
}

impl ResourceLimits {
    /// Returns the canonical `FS-01` bounded resource limits preset.
    #[must_use]
    pub const fn default_limits() -> Self {
        Self {
            retained_source_bytes: 64 * MIB,
            gpu_staging_upload_bytes: 32 * MIB,
            gpu_resident_bytes: 64 * MIB,
            gpu_staging_readback_bytes: 32 * MIB,
            cpu_output_bytes: 32 * MIB,
            max_volume_bytes: 512 * MIB,
            max_frame_dim: 8192,
            max_volume_dim: 2048,
            max_in_flight_jobs: 4,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::default_limits()
    }
}

/// Monotonically increasing correlation identifier stamped into each `RenderReceipt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationId(pub u64);

impl fmt::Display for OperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Point-in-time active and high-water-mark usage telemetry across `ResourceTracker` pools.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolUsageSnapshot {
    /// Currently retained CPU source capacity bytes.
    pub retained_source_bytes: usize,
    /// High-water mark of retained CPU source capacity bytes.
    pub peak_retained_source_bytes: usize,
    /// Currently reserved GPU upload staging bytes.
    pub gpu_staging_upload_bytes: usize,
    /// High-water mark of reserved GPU upload staging bytes.
    pub peak_gpu_staging_upload_bytes: usize,
    /// Currently reserved GPU resident texture bytes.
    pub gpu_resident_bytes: usize,
    /// High-water mark of reserved GPU resident texture bytes.
    pub peak_gpu_resident_bytes: usize,
    /// Currently reserved GPU readback staging bytes.
    pub gpu_staging_readback_bytes: usize,
    /// High-water mark of reserved GPU readback staging bytes.
    pub peak_gpu_staging_readback_bytes: usize,
    /// Currently reserved CPU output pixel buffer bytes.
    pub cpu_output_bytes: usize,
    /// High-water mark of reserved CPU output pixel buffer bytes.
    pub peak_cpu_output_bytes: usize,
    /// Currently active in-flight rendering jobs.
    pub in_flight_jobs: usize,
    /// High-water mark of concurrent in-flight rendering jobs.
    pub peak_in_flight_jobs: usize,
}

#[derive(Debug, Default)]
struct AtomicPoolCounter {
    active: AtomicUsize,
    peak: AtomicUsize,
}

impl AtomicPoolCounter {
    fn reserve(
        &self,
        pool: LimitPoolKind,
        requested: usize,
        limit: usize,
    ) -> Result<(), RenderError> {
        let mut current = self.active.load(Ordering::Acquire);
        loop {
            let available = limit.saturating_sub(current);
            let Some(next) = current.checked_add(requested) else {
                return Err(RenderError::ResourceLimitExceeded {
                    pool,
                    requested,
                    available,
                    limit,
                });
            };
            if next > limit {
                return Err(RenderError::ResourceLimitExceeded {
                    pool,
                    requested,
                    available,
                    limit,
                });
            }
            match self.active.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.peak.fetch_max(next, Ordering::AcqRel);
                    return Ok(());
                }
                Err(actual) => current = actual,
            }
        }
    }

    fn release(&self, amount: usize) {
        if amount > 0 {
            self.active.fetch_sub(amount, Ordering::Release);
        }
    }

    fn load_pair(&self) -> (usize, usize) {
        let active = self.active.load(Ordering::Acquire);
        let peak = self.peak.load(Ordering::Acquire).max(active);
        (active, peak)
    }
}

#[derive(Debug)]
struct ResourceTrackerInner {
    limits: ResourceLimits,
    retained_source: AtomicPoolCounter,
    gpu_staging_upload: AtomicPoolCounter,
    gpu_resident: AtomicPoolCounter,
    gpu_staging_readback: AtomicPoolCounter,
    cpu_output: AtomicPoolCounter,
    in_flight_jobs: AtomicPoolCounter,
    next_op_id: AtomicU64,
}

impl ResourceTrackerInner {
    fn counter_and_limit(&self, pool: LimitPoolKind) -> Option<(&AtomicPoolCounter, usize)> {
        match pool {
            LimitPoolKind::RetainedSource => {
                Some((&self.retained_source, self.limits.retained_source_bytes))
            }
            LimitPoolKind::GpuStagingUpload => Some((
                &self.gpu_staging_upload,
                self.limits.gpu_staging_upload_bytes,
            )),
            LimitPoolKind::GpuResident => {
                Some((&self.gpu_resident, self.limits.gpu_resident_bytes))
            }
            LimitPoolKind::GpuStagingReadback => Some((
                &self.gpu_staging_readback,
                self.limits.gpu_staging_readback_bytes,
            )),
            LimitPoolKind::CpuOutput => Some((&self.cpu_output, self.limits.cpu_output_bytes)),
            LimitPoolKind::InFlightJobs => {
                Some((&self.in_flight_jobs, self.limits.max_in_flight_jobs))
            }
            LimitPoolKind::VolumeBytes
            | LimitPoolKind::FrameDimension
            | LimitPoolKind::VolumeDimension => None,
        }
    }
}

/// Thread-safe quota tracker owning immutable `ResourceLimits` and live/peak atomic pool counters.
#[derive(Debug, Clone)]
pub struct ResourceTracker {
    inner: Arc<ResourceTrackerInner>,
}

impl ResourceTracker {
    /// Creates a new `ResourceTracker` governed by `limits`.
    #[must_use]
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            inner: Arc::new(ResourceTrackerInner {
                limits,
                retained_source: AtomicPoolCounter::default(),
                gpu_staging_upload: AtomicPoolCounter::default(),
                gpu_resident: AtomicPoolCounter::default(),
                gpu_staging_readback: AtomicPoolCounter::default(),
                cpu_output: AtomicPoolCounter::default(),
                in_flight_jobs: AtomicPoolCounter::default(),
                next_op_id: AtomicU64::new(1),
            }),
        }
    }

    /// Creates a new `ResourceTracker` governed by `ResourceLimits::default_limits()`.
    #[must_use]
    pub fn with_default_limits() -> Self {
        Self::new(ResourceLimits::default_limits())
    }

    /// Returns the immutable `ResourceLimits` governing this tracker.
    #[must_use]
    pub fn limits(&self) -> &ResourceLimits {
        &self.inner.limits
    }

    /// Captures a snapshot of active and peak resource usage across all tracked pools.
    #[must_use]
    pub fn snapshot(&self) -> PoolUsageSnapshot {
        let (retained_source_bytes, peak_retained_source_bytes) =
            self.inner.retained_source.load_pair();
        let (gpu_staging_upload_bytes, peak_gpu_staging_upload_bytes) =
            self.inner.gpu_staging_upload.load_pair();
        let (gpu_resident_bytes, peak_gpu_resident_bytes) = self.inner.gpu_resident.load_pair();
        let (gpu_staging_readback_bytes, peak_gpu_staging_readback_bytes) =
            self.inner.gpu_staging_readback.load_pair();
        let (cpu_output_bytes, peak_cpu_output_bytes) = self.inner.cpu_output.load_pair();
        let (in_flight_jobs, peak_in_flight_jobs) = self.inner.in_flight_jobs.load_pair();

        PoolUsageSnapshot {
            retained_source_bytes,
            peak_retained_source_bytes,
            gpu_staging_upload_bytes,
            peak_gpu_staging_upload_bytes,
            gpu_resident_bytes,
            peak_gpu_resident_bytes,
            gpu_staging_readback_bytes,
            peak_gpu_staging_readback_bytes,
            cpu_output_bytes,
            peak_cpu_output_bytes,
            in_flight_jobs,
            peak_in_flight_jobs,
        }
    }

    pub(crate) fn next_operation_id(&self) -> OperationId {
        OperationId(self.inner.next_op_id.fetch_add(1, Ordering::AcqRel))
    }

    pub(crate) fn reserve(
        &self,
        pool: LimitPoolKind,
        requested: usize,
    ) -> Result<BudgetGuard, RenderError> {
        if let Some((counter, limit)) = self.inner.counter_and_limit(pool) {
            counter.reserve(pool, requested, limit)?;
            return Ok(BudgetGuard {
                inner: Arc::clone(&self.inner),
                pool,
                amount: requested,
            });
        }

        let limit = match pool {
            LimitPoolKind::VolumeBytes => self.inner.limits.max_volume_bytes,
            LimitPoolKind::FrameDimension => self.inner.limits.max_frame_dim as usize,
            LimitPoolKind::VolumeDimension => self.inner.limits.max_volume_dim as usize,
            LimitPoolKind::RetainedSource
            | LimitPoolKind::GpuStagingUpload
            | LimitPoolKind::GpuResident
            | LimitPoolKind::GpuStagingReadback
            | LimitPoolKind::CpuOutput
            | LimitPoolKind::InFlightJobs => unreachable!("handled by counter_and_limit"),
        };

        if requested > limit {
            return Err(RenderError::ResourceLimitExceeded {
                pool,
                requested,
                available: limit,
                limit,
            });
        }

        Ok(BudgetGuard {
            inner: Arc::clone(&self.inner),
            pool,
            amount: requested,
        })
    }

    pub(crate) fn reserve_job(&self) -> Result<BudgetGuard, RenderError> {
        self.reserve(LimitPoolKind::InFlightJobs, 1)
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::with_default_limits()
    }
}

/// Internal RAII reservation guard that automatically releases live pool quota on drop.
#[derive(Debug)]
pub(crate) struct BudgetGuard {
    inner: Arc<ResourceTrackerInner>,
    pool: LimitPoolKind,
    amount: usize,
}

impl BudgetGuard {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn amount(&self) -> usize {
        self.amount
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn pool(&self) -> LimitPoolKind {
        self.pool
    }
}

impl Drop for BudgetGuard {
    fn drop(&mut self) {
        if let Some((counter, _)) = self.inner.counter_and_limit(self.pool) {
            counter.release(self.amount);
        }
    }
}

/// Cooperative thread-safe cancellation flag observed by CPU and GPU rendering paths.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates an uncancelled `CancellationToken`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Signals cooperative cancellation to all clones of this token.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Returns `true` if `cancel()` has been invoked on this token or any of its clones.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::{CancellationToken, OperationId, ResourceLimits, ResourceTracker};
    use crate::error::{LimitPoolKind, RenderError};

    #[test]
    fn default_limits_and_operation_ids_are_monotonic() {
        let tracker = ResourceTracker::with_default_limits();
        assert_eq!(tracker.limits(), &ResourceLimits::default());
        assert_eq!(tracker.next_operation_id(), OperationId(1));
        assert_eq!(tracker.next_operation_id(), OperationId(2));
        assert_eq!(tracker.next_operation_id(), OperationId(3));
    }

    #[test]
    fn exact_boundary_reservation_peak_tracking_and_drop_release() {
        let mut limits = ResourceLimits::default_limits();
        limits.retained_source_bytes = 1024;
        limits.cpu_output_bytes = 512;
        limits.max_in_flight_jobs = 2;
        limits.max_frame_dim = 64;
        limits.max_volume_dim = 32;
        limits.max_volume_bytes = 4096;

        let tracker = ResourceTracker::new(limits);

        let g1 = tracker
            .reserve(LimitPoolKind::RetainedSource, 600)
            .expect("600 <= 1024");
        assert_eq!(g1.amount(), 600);
        assert_eq!(g1.pool(), LimitPoolKind::RetainedSource);

        let g2 = tracker
            .reserve(LimitPoolKind::RetainedSource, 424)
            .expect("600 + 424 == 1024 exact boundary");

        let over = tracker
            .reserve(LimitPoolKind::RetainedSource, 1)
            .unwrap_err();
        assert_eq!(
            over,
            RenderError::ResourceLimitExceeded {
                pool: LimitPoolKind::RetainedSource,
                requested: 1,
                available: 0,
                limit: 1024,
            }
        );

        let overflow = tracker
            .reserve(LimitPoolKind::RetainedSource, usize::MAX)
            .unwrap_err();
        assert_eq!(
            overflow,
            RenderError::ResourceLimitExceeded {
                pool: LimitPoolKind::RetainedSource,
                requested: usize::MAX,
                available: 0,
                limit: 1024,
            }
        );

        let snap = tracker.snapshot();
        assert_eq!(snap.retained_source_bytes, 1024);
        assert_eq!(snap.peak_retained_source_bytes, 1024);

        drop(g1);
        assert_eq!(tracker.snapshot().retained_source_bytes, 424);
        assert_eq!(tracker.snapshot().peak_retained_source_bytes, 1024);

        drop(g2);
        assert_eq!(tracker.snapshot().retained_source_bytes, 0);
        assert_eq!(tracker.snapshot().peak_retained_source_bytes, 1024);

        let j1 = tracker.reserve_job().expect("job 1");
        let j2 = tracker.reserve_job().expect("job 2");
        assert!(matches!(
            tracker.reserve_job(),
            Err(RenderError::ResourceLimitExceeded {
                pool: LimitPoolKind::InFlightJobs,
                ..
            })
        ));
        drop((j1, j2));
        assert_eq!(tracker.snapshot().in_flight_jobs, 0);
        assert_eq!(tracker.snapshot().peak_in_flight_jobs, 2);

        assert!(tracker.reserve(LimitPoolKind::FrameDimension, 64).is_ok());
        assert!(matches!(
            tracker.reserve(LimitPoolKind::FrameDimension, 65),
            Err(RenderError::ResourceLimitExceeded {
                pool: LimitPoolKind::FrameDimension,
                requested: 65,
                available: 64,
                limit: 64,
            })
        ));
        assert!(tracker.reserve(LimitPoolKind::VolumeDimension, 32).is_ok());
        assert!(tracker.reserve(LimitPoolKind::VolumeBytes, 4096).is_ok());
        assert!(tracker.reserve(LimitPoolKind::VolumeBytes, 4097).is_err());
    }

    #[test]
    fn cancellation_token_propagates_across_clones() {
        let token = CancellationToken::new();
        let clone = token.clone();
        assert!(!token.is_cancelled());
        assert!(!clone.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
        assert!(clone.is_cancelled());
    }
}
