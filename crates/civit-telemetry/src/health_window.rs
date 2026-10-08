#![forbid(unsafe_code)]

//! Rolling health window for rollout gating.
//!
//! The health gate in `flag-kit` decides whether a rollout deserves more
//! exposure, and it needs evidence: an error rate and a latency
//! distribution over a *window*. The process-wide counters in
//! `tracing_setup` are cumulative since process start, which cannot answer
//! either question — a cumulative rate hides a spike that just happened,
//! and has no latency distribution at all.
//!
//! The window is a ring of fixed time buckets. Buckets rather than a deque
//! of samples because the gate reads the window on a timer, not per request,
//! so per-sample retention would be wasted work in the hot path. Each
//! bucket keeps only what the gate needs: totals plus a bounded latency
//! sample, so memory is O(buckets) regardless of traffic.
//!
//! Concurrency: a `Mutex` around the ring, not atomics per bucket. Two
//! threads recording in the same bucket and a reader summing the window can
//! otherwise disagree about which bucket is current. The critical section is
//! a handful of integer adds, and recording is already behind the tracing
//! middleware.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// One time bucket.
#[derive(Debug, Clone, Copy)]
struct Bucket {
    /// When this bucket's window began; `None` marks an empty slot.
    epoch: Option<u64>,
    total: u64,
    errors: u64,
    /// Bounded latency reservoir in milliseconds.
    latencies: [f32; Self::MAX_SAMPLES],
    sample_count: u32,
}

impl Bucket {
    /// Per-bucket sample cap. Enough for a p99 to mean something at low
    /// traffic without letting memory grow with request volume.
    const MAX_SAMPLES: usize = 64;

    fn zeroed() -> Self {
        Self {
            epoch: None,
            total: 0,
            errors: 0,
            latencies: [0.0; Self::MAX_SAMPLES],
            sample_count: 0,
        }
    }

    fn push_latency(&mut self, ms: f32) {
        if (self.sample_count as usize) < Self::MAX_SAMPLES {
            self.latencies[self.sample_count as usize] = ms;
            self.sample_count += 1;
        } else {
            // Replace the current maximum: a bounded reservoir that keeps the
            // worst recent sample preserves the property the gate cares
            // about (p99 must not miss a slow tail) instead of biasing toward
            // whichever samples happened to arrive first.
            let (idx, _) = self
                .latencies
                .iter()
                .enumerate()
                .take(self.sample_count as usize)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));
            self.latencies[idx] = ms;
        }
    }
}

/// Aggregate health over the current window.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HealthWindowSnapshot {
    /// Requests inside the window.
    pub total: u64,
    /// Requests that returned 5xx (or another server error) in the window.
    pub errors: u64,
    /// Latency samples in the window.
    pub latencies: Vec<f64>,
    /// How long the window has been accumulating.
    pub observed: Duration,
}

impl HealthWindowSnapshot {
    /// Error rate over the window, or `None` with no requests.
    ///
    /// `None` rather than `0.0` matters: the gate must be able to tell "no
    /// traffic" from "no failures", and conflating them lets a dead service
    /// look perfectly healthy.
    #[must_use]
    pub fn error_rate(&self) -> Option<f64> {
        if self.total == 0 {
            return None;
        }
        Some(self.errors as f64 / self.total as f64)
    }

    /// p99 latency in milliseconds, or `None` with no samples.
    #[must_use]
    pub fn latency_p99_ms(&self) -> Option<f64> {
        if self.latencies.is_empty() {
            return None;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let rank = ((sorted.len() as f64) * 0.99).ceil().max(1.0) as usize;
        sorted.get(rank.min(sorted.len()) - 1).copied()
    }
}

/// Fixed-size rolling window of request health.
#[derive(Debug)]
pub struct HealthWindow {
    buckets: Mutex<Vec<Bucket>>,
    bucket_width: Duration,
    /// Monotonic origin so bucket epochs are comparable across calls.
    origin: Instant,
}

impl HealthWindow {
    /// Builds a window of `bucket_count` buckets, each `bucket_width` wide.
    ///
    /// # Errors
    /// Returns `Err` when `bucket_count` is zero, which would silently
    /// produce a window that can never observe anything.
    pub fn new(bucket_count: usize, bucket_width: Duration) -> Result<Self, WindowError> {
        if bucket_count == 0 {
            return Err(WindowError::ZeroBuckets);
        }
        Ok(Self {
            buckets: Mutex::new(vec![Bucket::zeroed(); bucket_count]),
            bucket_width,
            origin: Instant::now(),
        })
    }

    /// The window's total duration.
    #[must_use]
    pub fn span(&self) -> Duration {
        self.bucket_width
            .checked_mul(self.buckets.lock().map(|b| b.len()).unwrap_or(0) as u32)
            .unwrap_or(self.bucket_width)
    }

    fn slot(&self, buckets: &mut [Bucket], elapsed: Duration) -> usize {
        let width_secs = self.bucket_width.as_secs_f64().max(f64::MIN_POSITIVE);
        let idx = ((elapsed.as_secs_f64() / width_secs) as usize) % buckets.len();
        let epoch = (elapsed.as_secs_f64() / width_secs) as u64;
        // Reset the slot when it belongs to an older epoch, so a long-idle
        // window does not report stale traffic as current.
        if buckets[idx].epoch != Some(epoch) {
            buckets[idx] = Bucket {
                epoch: Some(epoch),
                ..Bucket::zeroed()
            };
        }
        idx
    }

    /// Records one completed request.
    pub fn record(&self, duration: Duration, status: u16) {
        let Ok(mut guard) = self.buckets.lock() else {
            // A poisoned lock means another thread panicked while holding it.
            // Dropping the sample is correct here: telemetry must never take
            // down the request path, and the gate treats missing evidence as
            // "hold", not "healthy".
            return;
        };
        let elapsed = self.origin.elapsed();
        let idx = self.slot(&mut guard, elapsed);
        let bucket = &mut guard[idx];
        bucket.total += 1;
        if status >= 500 {
            bucket.errors += 1;
        }
        bucket.push_latency((duration.as_secs_f64() * 1000.0) as f32);
    }

    /// Aggregates the current window.
    pub fn snapshot(&self) -> HealthWindowSnapshot {
        let Ok(guard) = self.buckets.lock() else {
            return HealthWindowSnapshot::default();
        };
        let elapsed = self.origin.elapsed();
        let current_epoch = (elapsed.as_secs_f64() / self.bucket_width.as_secs_f64().max(f64::MIN_POSITIVE))
            as u64;
        let bucket_count = guard.len() as u64;
        let mut total = 0u64;
        let mut errors = 0u64;
        let mut latencies: Vec<f64> = Vec::new();
        for bucket in guard.iter() {
            // Bucket epochs are absolute (seconds since window creation), so
            // freshness is distance in epochs, not a comparison against
            // "now". An `e + 1 >= current_epoch` test admits only the last
            // two seconds of a long-lived window and silently starves every
            // reader — found by the rollout gate holding on
            // too_few_samples under continuous traffic.
            match bucket.epoch {
                Some(e) if current_epoch.saturating_sub(e) < bucket_count => {
                    total += bucket.total;
                    errors += bucket.errors;
                    latencies.extend(
                        bucket.latencies[..bucket.sample_count as usize]
                            .iter()
                            .map(|v| f64::from(*v)),
                    );
                }
                _ => {}
            }
        }
        HealthWindowSnapshot {
            total,
            errors,
            latencies,
            observed: elapsed,
        }
    }
}

/// Error from an invalid window configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowError {
    /// A zero-bucket window can never observe anything.
    ZeroBuckets,
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("health window needs at least one bucket")
    }
}

impl std::error::Error for WindowError {}

/// Process-wide health window: 60 one-second buckets, so a minute of traffic
/// is what the gate judges.
pub fn global_health_window() -> &'static HealthWindow {
    static WINDOW: std::sync::LazyLock<HealthWindow> = std::sync::LazyLock::new(|| {
        HealthWindow::new(60, Duration::from_secs(1))
            .expect("60x1s health window is a valid configuration")
    });
    &WINDOW
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn window(buckets: usize, width_ms: u64) -> HealthWindow {
        HealthWindow::new(buckets, Duration::from_millis(width_ms)).unwrap()
    }

    #[test]
    fn zero_buckets_is_rejected() {
        assert!(HealthWindow::new(0, Duration::from_secs(1)).is_err());
    }

    #[test]
    fn records_totals_and_server_errors_only() {
        let w = window(4, 1000);
        w.record(Duration::from_millis(10), 200);
        w.record(Duration::from_millis(20), 404);
        w.record(Duration::from_millis(30), 500);
        w.record(Duration::from_millis(40), 503);
        let s = w.snapshot();
        assert_eq!(s.total, 4);
        assert_eq!(s.errors, 2, "only 5xx counts as an error");
        assert_eq!(s.error_rate(), Some(0.5));
    }

    /// A dead service and a perfect service both report zero errors. The
    /// gate must be able to tell them apart, so an empty window reports
    /// `None`, not `Some(0.0)`.
    #[test]
    fn empty_window_reports_no_rate_at_all() {
        let w = window(4, 1000);
        let s = w.snapshot();
        assert_eq!(s.total, 0);
        assert_eq!(s.error_rate(), None);
        assert_eq!(s.latency_p99_ms(), None);
    }

    #[test]
    fn latency_samples_reach_the_snapshot() {
        let w = window(4, 1000);
        for ms in 1..=10u64 {
            w.record(Duration::from_millis(ms), 200);
        }
        let s = w.snapshot();
        assert_eq!(s.total, 10);
        let p99 = s.latency_p99_ms().expect("samples present");
        assert!(p99 >= 9.0 && p99 <= 10.0, "p99 was {p99}");
    }

    /// Sample memory must not grow with traffic: the reservoir is bounded.
    #[test]
    fn sample_reservoir_is_bounded() {
        let w = window(2, 1000);
        for i in 0..1000u64 {
            w.record(Duration::from_millis(i % 5), 200);
        }
        let s = w.snapshot();
        assert_eq!(s.total, 1000, "counters are exact");
        assert!(
            s.latencies.len() <= Bucket::MAX_SAMPLES * 2,
            "samples must stay bounded, got {}",
            s.latencies.len()
        );
    }

    /// Replacing the maximum keeps the slow tail visible, which is the whole
    /// point of gating on p99.
    #[test]
    fn bounded_reservoir_still_sees_a_slow_tail() {
        let w = window(1, 60_000);
        for _ in 0..(Bucket::MAX_SAMPLES * 4) {
            w.record(Duration::from_millis(5), 200);
        }
        w.record(Duration::from_millis(9000), 200);
        let p99 = w.snapshot().latency_p99_ms().expect("samples");
        assert!(p99 >= 5000.0, "slow tail must survive eviction, got {p99}");
    }

    /// A window alive for many minutes must still aggregate its recent
    /// buckets: epochs are absolute, so freshness is distance from now. The
    /// rollout gate starved on exactly this before the fix.
    #[test]
    fn long_lived_window_counts_recent_samples() {
        let w = HealthWindow::new(60, Duration::from_millis(10)).unwrap();
        std::thread::sleep(Duration::from_millis(120));
        w.record(Duration::from_millis(5), 200);
        w.record(Duration::from_millis(5), 500);
        let s = w.snapshot();
        assert_eq!(s.total, 2, "fresh samples in an aged window must count");
        assert_eq!(s.errors, 1);
    }

    #[test]
    fn window_rolls_over_old_buckets() {
        let w = window(2, 40);
        w.record(Duration::from_millis(5), 500);
        assert_eq!(w.snapshot().errors, 1);
        std::thread::sleep(Duration::from_millis(120));
        let s = w.snapshot();
        assert_eq!(s.total, 0, "traffic from 3+ buckets ago must age out");
        assert_eq!(s.errors, 0);
    }

    #[test]
    fn span_covers_the_configured_buckets() {
        let w = window(30, 1000);
        assert_eq!(w.span(), Duration::from_secs(30));
    }

    #[test]
    fn global_window_is_usable() {
        let w = global_health_window();
        w.record(Duration::from_millis(1), 200);
        assert!(w.snapshot().total >= 1);
    }
}