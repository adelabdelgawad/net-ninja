//! Progressive ramp-up throughput measurement.
//!
//! Implements the core algorithm for congestion-aware concurrency scaling:
//! - Start with 1 concurrent worker
//! - Double concurrency at each level (1 → 2 → 4 → 8)
//! - Stop when throughput gain falls below threshold
//! - Return the best throughput achieved
//!
//! Also implements Ookla-compatible sustained throughput measurement:
//! - Samples throughput at 30Hz during measurement
//! - Calculates trimmed mean (discards slowest 30%, fastest 10%)
//! - Reports both peak (link capacity) and sustained (Ookla-compatible) speeds

use std::future::Future;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use super::models::{RampLevelResult, ThroughputResult};
use super::SpeedtestConfig;

/// Shared state for tracking throughput during a test.
#[derive(Debug)]
pub struct ThroughputTracker {
    /// Total bytes transferred
    pub bytes: Arc<AtomicU64>,

    /// Requests completed successfully
    pub requests_completed: Arc<AtomicU32>,

    /// Requests that failed
    pub requests_failed: Arc<AtomicU32>,

    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<()>,

    /// Throughput samples collected during measurement (bytes per second).
    /// Used for Ookla-compatible trimmed mean calculation.
    samples: Arc<Mutex<Vec<f64>>>,

    /// Last byte count, used for delta calculation in sampling.
    last_bytes: Arc<AtomicU64>,
}

impl ThroughputTracker {
    /// Create a new throughput tracker.
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);
        Self {
            bytes: Arc::new(AtomicU64::new(0)),
            requests_completed: Arc::new(AtomicU32::new(0)),
            requests_failed: Arc::new(AtomicU32::new(0)),
            shutdown_tx,
            samples: Arc::new(Mutex::new(Vec::with_capacity(300))), // ~10s at 30Hz
            last_bytes: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Reset counters for a new measurement level.
    pub fn reset(&self) {
        self.bytes.store(0, Ordering::SeqCst);
        self.requests_completed.store(0, Ordering::SeqCst);
        self.requests_failed.store(0, Ordering::SeqCst);
        self.last_bytes.store(0, Ordering::SeqCst);
        if let Ok(mut samples) = self.samples.lock() {
            samples.clear();
        }
    }

    /// Get a shutdown receiver for workers.
    pub fn shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Signal all workers to stop.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Add bytes to the counter.
    pub fn add_bytes(&self, bytes: u64) {
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Increment completed request counter.
    pub fn request_completed(&self) {
        self.requests_completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment failed request counter.
    pub fn request_failed(&self) {
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current byte count.
    pub fn get_bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }

    /// Get completed request count.
    pub fn get_completed(&self) -> u32 {
        self.requests_completed.load(Ordering::Relaxed)
    }

    /// Get failed request count.
    pub fn get_failed(&self) -> u32 {
        self.requests_failed.load(Ordering::Relaxed)
    }

    /// Take a throughput sample.
    ///
    /// Calculates the bytes transferred since the last sample and converts
    /// to bytes per second based on the sample interval.
    pub fn take_sample(&self, sample_interval: Duration) {
        let current_bytes = self.bytes.load(Ordering::Relaxed);
        let last_bytes = self.last_bytes.swap(current_bytes, Ordering::Relaxed);
        let delta_bytes = current_bytes.saturating_sub(last_bytes);

        // Convert to bytes per second
        let bps = delta_bytes as f64 / sample_interval.as_secs_f64();

        if let Ok(mut samples) = self.samples.lock() {
            samples.push(bps);
        }
    }

    /// Get all collected samples.
    pub fn get_samples(&self) -> Vec<f64> {
        self.samples.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

impl Default for ThroughputTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Run progressive ramp-up test with the provided worker function.
///
/// The worker function receives:
/// - A tracker for reporting bytes transferred
/// - A shutdown receiver to know when to stop
///
/// # Type Parameters
/// * `F` - Factory function that creates workers
/// * `Fut` - Future returned by each worker
///
/// # Arguments
/// * `config` - Test configuration
/// * `worker_factory` - Function that creates a worker task
///
/// # Returns
/// Result containing both peak and sustained throughput with statistics.
pub async fn run_progressive_ramp<F, Fut>(
    config: &SpeedtestConfig,
    test_name: &str,
    worker_factory: F,
) -> ThroughputResult
where
    F: Fn(Arc<ThroughputTracker>) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let levels = config.concurrency_levels();
    let mut results: Vec<RampLevelResult> = Vec::with_capacity(levels.len());
    let mut best_throughput: f64 = 0.0;
    let mut best_sustained: f64 = 0.0;
    let mut best_concurrency: usize = 1;
    let mut total_completed: u32 = 0;
    let mut total_failed: u32 = 0;

    tracing::info!(
        "[speedtest_progressive::throughput] Starting {} ramp-up test (levels: {:?})",
        test_name,
        levels
    );

    for concurrency in levels {
        let tracker = Arc::new(ThroughputTracker::new());

        tracing::debug!(
            "[speedtest_progressive::throughput] {} level {} starting",
            test_name,
            concurrency
        );

        // Spawn workers
        let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(concurrency);
        for _ in 0..concurrency {
            let tracker_clone = Arc::clone(&tracker);
            let factory = worker_factory.clone();
            handles.push(tokio::spawn(async move {
                factory(tracker_clone).await;
            }));
        }

        // Warmup period
        tokio::time::sleep(config.warmup_duration()).await;

        // Reset counters after warmup
        tracker.reset();

        // Measurement period with sampling
        let start = Instant::now();
        let measurement_duration = config.ramp_level_duration();
        let sample_interval = config.sample_interval();

        // Spawn sampling task
        let tracker_for_sampling = Arc::clone(&tracker);
        let sampling_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(sample_interval);
            interval.tick().await; // Skip first immediate tick
            loop {
                interval.tick().await;
                tracker_for_sampling.take_sample(sample_interval);
            }
        });

        // Wait for measurement duration
        tokio::time::sleep(measurement_duration).await;
        let elapsed = start.elapsed();

        // Stop sampling
        sampling_handle.abort();

        // Signal workers to stop
        tracker.shutdown();

        // Wait for workers to finish (with timeout)
        let shutdown_timeout = Duration::from_secs(2);
        let _ = tokio::time::timeout(shutdown_timeout, async {
            for handle in handles {
                let _ = handle.await;
            }
        })
        .await;

        // Calculate peak throughput (total bytes / time)
        let bytes = tracker.get_bytes();
        let completed = tracker.get_completed();
        let failed = tracker.get_failed();

        let throughput_bps = bytes as f64 / elapsed.as_secs_f64();
        let throughput_mbps = (throughput_bps * 8.0) / 1_000_000.0;

        // Calculate sustained throughput (trimmed mean of samples)
        let samples = tracker.get_samples();
        let mut sustained_bps = config.trimmed_mean(&samples);

        // Fallback: if sustained is 0 but we have throughput, samples were too bursty
        // This happens with upload where bytes are counted in large bursts per request.
        // Use peak throughput as fallback (with a slight reduction to estimate sustained).
        if sustained_bps <= 0.0 && throughput_bps > 0.0 {
            // Apply a conservative 95% factor as fallback estimate
            sustained_bps = throughput_bps * 0.95;
            tracing::debug!(
                "[speedtest_progressive::throughput] {} sustained calculation failed (bursty samples), using 95% of peak as fallback",
                test_name
            );
        }

        let sustained_mbps = (sustained_bps * 8.0) / 1_000_000.0;

        tracing::info!(
            "[speedtest_progressive::throughput] {} level {}: {:.2} Mbps peak, {:.2} Mbps sustained ({} samples, {} completed, {} failed)",
            test_name,
            concurrency,
            throughput_mbps,
            sustained_mbps,
            samples.len(),
            completed,
            failed
        );

        results.push(RampLevelResult {
            concurrency,
            throughput_bps,
            requests_completed: completed,
            requests_failed: failed,
        });

        total_completed += completed;
        total_failed += failed;

        // Check if we should continue or stop (plateau detection)
        if throughput_bps > best_throughput {
            let gain = if best_throughput > 0.0 {
                (throughput_bps - best_throughput) / best_throughput
            } else {
                1.0 // First level always counts as improvement
            };

            if gain < config.throughput_gain_threshold && concurrency > config.initial_concurrency {
                tracing::info!(
                    "[speedtest_progressive::throughput] {} plateau detected at level {} (gain: {:.1}% < {:.1}%)",
                    test_name,
                    concurrency,
                    gain * 100.0,
                    config.throughput_gain_threshold * 100.0
                );
                // Update best if this level was still better
                if throughput_bps > best_throughput {
                    best_throughput = throughput_bps;
                    best_sustained = sustained_bps;
                    best_concurrency = concurrency;
                }
                break;
            }

            best_throughput = throughput_bps;
            best_sustained = sustained_bps;
            best_concurrency = concurrency;
        } else if concurrency > config.initial_concurrency {
            // Throughput decreased, stop
            tracing::info!(
                "[speedtest_progressive::throughput] {} throughput decreased at level {}, stopping",
                test_name,
                concurrency
            );
            break;
        }
    }

    let result = ThroughputResult {
        throughput_bps: best_throughput,
        sustained_throughput_bps: best_sustained,
        best_concurrency,
        requests_completed: total_completed,
        requests_failed: total_failed,
        levels: results,
    };

    tracing::info!(
        "[speedtest_progressive::throughput] {} complete: {:.2} Mbps peak, {:.2} Mbps sustained at concurrency {}",
        test_name,
        result.to_mbps(),
        result.to_sustained_mbps(),
        best_concurrency
    );

    result
}

/// Result from run_single_level including sustained throughput.
#[derive(Debug, Clone)]
pub struct SingleLevelResult {
    /// Ramp level result with peak throughput
    pub level: RampLevelResult,
    /// Sustained throughput in bytes per second (trimmed mean)
    #[allow(dead_code)]
    pub sustained_throughput_bps: f64,
}

/// Helper to run a single concurrency level (used for testing).
#[allow(dead_code)]
pub async fn run_single_level<F, Fut>(
    concurrency: usize,
    duration: Duration,
    warmup: Duration,
    worker_factory: F,
) -> RampLevelResult
where
    F: Fn(Arc<ThroughputTracker>) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let config = SpeedtestConfig::default();
    let result = run_single_level_with_config(concurrency, duration, warmup, &config, worker_factory).await;
    result.level
}

/// Helper to run a single concurrency level with config (includes sustained throughput).
#[allow(dead_code)]
pub async fn run_single_level_with_config<F, Fut>(
    concurrency: usize,
    duration: Duration,
    warmup: Duration,
    config: &SpeedtestConfig,
    worker_factory: F,
) -> SingleLevelResult
where
    F: Fn(Arc<ThroughputTracker>) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let tracker = Arc::new(ThroughputTracker::new());

    // Spawn workers
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let tracker_clone = Arc::clone(&tracker);
        let factory = worker_factory.clone();
        handles.push(tokio::spawn(async move {
            factory(tracker_clone).await;
        }));
    }

    // Warmup
    tokio::time::sleep(warmup).await;
    tracker.reset();

    // Measurement with sampling
    let start = Instant::now();
    let sample_interval = config.sample_interval();

    // Spawn sampling task
    let tracker_for_sampling = Arc::clone(&tracker);
    let sampling_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(sample_interval);
        interval.tick().await;
        loop {
            interval.tick().await;
            tracker_for_sampling.take_sample(sample_interval);
        }
    });

    tokio::time::sleep(duration).await;
    let elapsed = start.elapsed();

    sampling_handle.abort();
    tracker.shutdown();

    // Wait for workers
    for handle in handles {
        let _ = handle.await;
    }

    let bytes = tracker.get_bytes();
    let throughput_bps = bytes as f64 / elapsed.as_secs_f64();
    let samples = tracker.get_samples();
    let sustained_bps = config.trimmed_mean(&samples);

    SingleLevelResult {
        level: RampLevelResult {
            concurrency,
            throughput_bps,
            requests_completed: tracker.get_completed(),
            requests_failed: tracker.get_failed(),
        },
        sustained_throughput_bps: sustained_bps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_reset() {
        let tracker = ThroughputTracker::new();
        tracker.add_bytes(1000);
        tracker.request_completed();
        tracker.request_failed();

        assert_eq!(tracker.get_bytes(), 1000);
        assert_eq!(tracker.get_completed(), 1);
        assert_eq!(tracker.get_failed(), 1);

        tracker.reset();

        assert_eq!(tracker.get_bytes(), 0);
        assert_eq!(tracker.get_completed(), 0);
        assert_eq!(tracker.get_failed(), 0);
    }

    #[test]
    fn test_tracker_concurrent_access() {
        let tracker = Arc::new(ThroughputTracker::new());

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let t = Arc::clone(&tracker);
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        t.add_bytes(1);
                        t.request_completed();
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(tracker.get_bytes(), 1000);
        assert_eq!(tracker.get_completed(), 1000);
    }
}
