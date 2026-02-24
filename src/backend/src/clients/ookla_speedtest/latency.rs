//! Trimmed median latency measurement.
//!
//! Measures latency using multiple HTTP probes, discards outliers,
//! and returns the median of the remaining values.

use reqwest::Client;
use std::time::Instant;

use super::models::TestServer;
use super::SpeedtestConfig;

/// Measure latency to a server using trimmed median algorithm.
///
/// Performs `probe_count` HTTP GET requests to the server's latency endpoint,
/// discards the top and bottom `trim_percent` of results, and returns the
/// median of the remaining values.
///
/// # Arguments
/// * `client` - HTTP client to use for requests
/// * `server` - Server to measure latency to
/// * `config` - Configuration with probe_count and trim_percent
///
/// # Returns
/// Median latency in milliseconds, or None if all probes failed.
pub async fn measure_trimmed_latency(
    client: &Client,
    server: &TestServer,
    config: &SpeedtestConfig,
) -> Option<f64> {
    measure_trimmed_latency_with_params(
        client,
        server,
        config.latency_probe_count,
        config.latency_trim_percent,
    )
    .await
}

/// Measure latency with explicit parameters.
///
/// # Arguments
/// * `client` - HTTP client to use
/// * `server` - Server to test
/// * `probe_count` - Number of probes to send (default: 7)
/// * `trim_percent` - Percentage to trim from each end (default: 0.20)
pub async fn measure_trimmed_latency_with_params(
    client: &Client,
    server: &TestServer,
    probe_count: usize,
    trim_percent: f64,
) -> Option<f64> {
    let url = server.latency_url();
    let mut latencies: Vec<f64> = Vec::with_capacity(probe_count);

    tracing::debug!(
        "[speedtest_progressive::latency] Measuring latency to {} ({} probes)",
        server.name,
        probe_count
    );

    for i in 0..probe_count {
        match measure_single_latency(client, &url).await {
            Some(latency_ms) => {
                tracing::trace!(
                    "[speedtest_progressive::latency] Probe {}/{}: {:.2}ms",
                    i + 1,
                    probe_count,
                    latency_ms
                );
                latencies.push(latency_ms);
            }
            None => {
                tracing::trace!(
                    "[speedtest_progressive::latency] Probe {}/{}: failed",
                    i + 1,
                    probe_count
                );
            }
        }
    }

    if latencies.is_empty() {
        tracing::warn!(
            "[speedtest_progressive::latency] All {} probes to {} failed",
            probe_count,
            server.name
        );
        return None;
    }

    // Sort for trimming and median calculation
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate trim count
    let trim_count = ((latencies.len() as f64 * trim_percent) / 2.0).floor() as usize;

    // Ensure we have at least one value after trimming
    if latencies.len() <= trim_count * 2 {
        // Not enough values to trim, just return median
        let median = calculate_median(&latencies);
        tracing::debug!(
            "[speedtest_progressive::latency] {} latency: {:.2}ms (no trim, {} samples)",
            server.name,
            median,
            latencies.len()
        );
        return Some(median);
    }

    // Trim outliers
    let trimmed: Vec<f64> = latencies[trim_count..latencies.len() - trim_count].to_vec();

    let median = calculate_median(&trimmed);
    tracing::debug!(
        "[speedtest_progressive::latency] {} latency: {:.2}ms (trimmed {} of {} samples)",
        server.name,
        median,
        trim_count * 2,
        latencies.len()
    );

    Some(median)
}

/// Measure a single latency probe.
///
/// Note: We measure round-trip time regardless of HTTP status code.
/// Many speedtest servers return 404 or 500 for the latency endpoint but
/// still work correctly for actual downloads. This matches Ookla's behavior.
async fn measure_single_latency(client: &Client, url: &str) -> Option<f64> {
    let start = Instant::now();

    tracing::trace!("[speedtest_progressive::latency] Probing: {}", url);

    match client.get(url).send().await {
        Ok(response) => {
            // Measure round-trip time regardless of HTTP status
            // Many servers return 404/500 for latency but work for downloads
            let latency = start.elapsed().as_secs_f64() * 1000.0;
            let status = response.status();

            if !status.is_success() {
                tracing::trace!(
                    "[speedtest_progressive::latency] HTTP {} (ignoring, measuring connectivity): {:.2}ms",
                    status,
                    latency
                );
            } else {
                tracing::trace!(
                    "[speedtest_progressive::latency] HTTP {} - {:.2}ms",
                    status,
                    latency
                );
            }

            Some(latency)
        }
        Err(e) => {
            tracing::trace!("[speedtest_progressive::latency] Request failed: {:?}", e);
            None
        }
    }
}

/// Calculate median of a sorted slice.
fn calculate_median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_median_odd() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_median(&values), 3.0);
    }

    #[test]
    fn test_calculate_median_even() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(calculate_median(&values), 2.5);
    }

    #[test]
    fn test_calculate_median_single() {
        let values = vec![42.0];
        assert_eq!(calculate_median(&values), 42.0);
    }

    #[test]
    fn test_calculate_median_empty() {
        let values: Vec<f64> = vec![];
        assert_eq!(calculate_median(&values), 0.0);
    }
}
