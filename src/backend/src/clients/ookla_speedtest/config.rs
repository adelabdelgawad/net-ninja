//! Configuration for the Ookla-style progressive speedtest.
//!
//! All tunable parameters for the progressive ramp-up speedtest algorithm.

use std::time::Duration;

/// Configuration for the progressive speedtest runner.
///
/// All values have sensible defaults based on Ookla's behavior.
#[derive(Debug, Clone)]
pub struct SpeedtestConfig {
    /// Number of closest servers to latency-test (default: 5)
    pub closest_server_count: usize,

    /// Number of latency probes per server (default: 7)
    pub latency_probe_count: usize,

    /// Percentage of extreme latency values to trim (default: 0.20 = 20%)
    pub latency_trim_percent: f64,

    /// Starting concurrency level (default: 1)
    pub initial_concurrency: usize,

    /// Maximum concurrency level for ramp-up (default: 8)
    pub max_concurrency: usize,

    /// Duration to run each ramp level in seconds (default: 2.0)
    pub ramp_level_duration_secs: f64,

    /// Minimum throughput gain to continue ramping up (default: 0.05 = 5%)
    pub throughput_gain_threshold: f64,

    /// Warmup duration before measuring throughput (default: 1.0 seconds)
    pub warmup_duration_secs: f64,

    /// HTTP request timeout in seconds (default: 30)
    pub request_timeout_secs: u64,

    /// TCP connection timeout in seconds (default: 10)
    pub connect_timeout_secs: u64,

    /// Maximum distance to servers in kilometers (default: 1000)
    pub max_server_distance_km: f64,

    /// Chunk size for upload in bytes (default: 256KB)
    /// Smaller chunks complete faster, allowing more samples during measurement.
    pub upload_chunk_size: usize,

    /// Size parameters for download test (default: matches speedtest-rs)
    /// Uses multiple sizes to include HTTP overhead in measurement for realistic results.
    pub download_sizes: Vec<u32>,

    // === Sampling configuration for Ookla-compatible sustained throughput ===

    /// Sample rate in Hz for throughput measurement (default: 30)
    /// Ookla samples at ~30 times per second.
    pub sample_rate_hz: u32,

    /// Percentage of slowest samples to discard (default: 0.30 = 30%)
    /// Ookla discards the slowest 30% to remove TCP slow-start and congestion dips.
    pub trim_low_percent: f64,

    /// Percentage of fastest samples to discard (default: 0.10 = 10%)
    /// Ookla discards the fastest 10% to remove burst/peak traffic.
    pub trim_high_percent: f64,
}

impl Default for SpeedtestConfig {
    fn default() -> Self {
        Self {
            closest_server_count: 5,
            latency_probe_count: 7,
            latency_trim_percent: 0.20,
            // Match speedtest-rs: use fixed concurrency, not progressive ramp-up
            initial_concurrency: 4,
            max_concurrency: 4, // Same as initial = no ramp-up, fixed concurrency
            // Match speedtest-rs: 10 second test duration
            ramp_level_duration_secs: 10.0,
            throughput_gain_threshold: 0.05,
            // No warmup - match speedtest-rs (count all bytes from start)
            warmup_duration_secs: 0.0,
            request_timeout_secs: 30,
            connect_timeout_secs: 10,
            max_server_distance_km: 1000.0,
            upload_chunk_size: 256 * 1024, // 256 KB
            // Match speedtest-rs: multiple sizes to include HTTP overhead for realistic results
            download_sizes: vec![350, 500, 750, 1000, 1500, 2000, 2500, 3000, 3500, 4000],
            // Ookla-compatible sampling defaults
            sample_rate_hz: 30,
            trim_low_percent: 0.30,  // Discard slowest 30%
            trim_high_percent: 0.10, // Discard fastest 10%
        }
    }
}

impl SpeedtestConfig {
    /// Create a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get request timeout as Duration.
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    /// Get connection timeout as Duration.
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    /// Get ramp level duration as Duration.
    pub fn ramp_level_duration(&self) -> Duration {
        Duration::from_secs_f64(self.ramp_level_duration_secs)
    }

    /// Get warmup duration as Duration.
    pub fn warmup_duration(&self) -> Duration {
        Duration::from_secs_f64(self.warmup_duration_secs)
    }

    /// Get the concurrency levels for progressive ramp-up.
    /// Returns levels from initial to max (e.g., [1, 2, 4, 8]).
    pub fn concurrency_levels(&self) -> Vec<usize> {
        let mut levels = Vec::new();
        let mut level = self.initial_concurrency;
        while level <= self.max_concurrency {
            levels.push(level);
            level *= 2;
        }
        // Ensure we include max if it wasn't a power of 2 multiple
        if levels.last().map(|&l| l != self.max_concurrency).unwrap_or(true)
            && self.max_concurrency > self.initial_concurrency
        {
            levels.push(self.max_concurrency);
        }
        levels
    }

    /// Get the sample interval as Duration (1 / sample_rate_hz).
    pub fn sample_interval(&self) -> Duration {
        Duration::from_micros(1_000_000 / self.sample_rate_hz as u64)
    }

    /// Calculate trimmed mean from a vector of samples.
    ///
    /// Discards the slowest `trim_low_percent` and fastest `trim_high_percent`
    /// of samples, then returns the mean of the remaining samples.
    ///
    /// This matches Ookla's methodology for sustained throughput measurement.
    pub fn trimmed_mean(&self, samples: &[f64]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted.len();
        let trim_low = (n as f64 * self.trim_low_percent) as usize;
        let trim_high = (n as f64 * (1.0 - self.trim_high_percent)) as usize;

        // Ensure we have at least one sample after trimming
        let start = trim_low.min(n.saturating_sub(1));
        let end = trim_high.max(start + 1).min(n);

        let trimmed = &sorted[start..end];
        if trimmed.is_empty() {
            // Fallback: return median if trimming removed everything
            sorted[n / 2]
        } else {
            trimmed.iter().sum::<f64>() / trimmed.len() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SpeedtestConfig::default();
        assert_eq!(config.closest_server_count, 5);
        assert_eq!(config.latency_probe_count, 7);
        // Fixed concurrency (no ramp-up) matching speedtest-rs behavior
        assert_eq!(config.max_concurrency, 4);
        assert_eq!(config.initial_concurrency, 4);
        // Sampling defaults
        assert_eq!(config.sample_rate_hz, 30);
        assert!((config.trim_low_percent - 0.30).abs() < 0.001);
        assert!((config.trim_high_percent - 0.10).abs() < 0.001);
    }

    #[test]
    fn test_concurrency_levels() {
        let config = SpeedtestConfig::default();
        let levels = config.concurrency_levels();
        // Default is fixed concurrency (no ramp-up)
        assert_eq!(levels, vec![4]);
    }

    #[test]
    fn test_concurrency_levels_custom() {
        let config = SpeedtestConfig {
            initial_concurrency: 2,
            max_concurrency: 16,
            ..Default::default()
        };
        let levels = config.concurrency_levels();
        assert_eq!(levels, vec![2, 4, 8, 16]);
    }

    #[test]
    fn test_duration_conversions() {
        let config = SpeedtestConfig::default();
        assert_eq!(config.request_timeout(), Duration::from_secs(30));
        assert_eq!(config.connect_timeout(), Duration::from_secs(10));
        assert_eq!(config.ramp_level_duration(), Duration::from_secs(10));
    }

    #[test]
    fn test_sample_interval() {
        let config = SpeedtestConfig::default();
        // 30 Hz = ~33.33ms interval
        let interval = config.sample_interval();
        assert_eq!(interval.as_micros(), 33333);
    }

    #[test]
    fn test_trimmed_mean_basic() {
        let config = SpeedtestConfig::default();
        // With 10 samples: discard slowest 3 (30%) and fastest 1 (10%), keep middle 6
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mean = config.trimmed_mean(&samples);
        // Should average samples 4,5,6,7,8,9 = (4+5+6+7+8+9)/6 = 39/6 = 6.5
        assert!((mean - 6.5).abs() < 0.001);
    }

    #[test]
    fn test_trimmed_mean_empty() {
        let config = SpeedtestConfig::default();
        let samples: Vec<f64> = vec![];
        let mean = config.trimmed_mean(&samples);
        assert_eq!(mean, 0.0);
    }

    #[test]
    fn test_trimmed_mean_single() {
        let config = SpeedtestConfig::default();
        let samples = vec![100.0];
        let mean = config.trimmed_mean(&samples);
        assert_eq!(mean, 100.0);
    }

    #[test]
    fn test_trimmed_mean_removes_outliers() {
        let config = SpeedtestConfig::default();
        // Burst at the end should be trimmed
        let mut samples = vec![50.0; 90];
        samples.push(150.0); // High outlier (should be trimmed)
        samples.extend(vec![10.0; 9]); // Low outliers (should be trimmed)

        let mean = config.trimmed_mean(&samples);
        // After trimming, should be close to 50.0
        assert!((mean - 50.0).abs() < 1.0);
    }
}
