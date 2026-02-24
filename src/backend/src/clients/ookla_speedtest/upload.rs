//! Upload speed test with progressive ramp-up.
//!
//! Posts large data chunks while tracking throughput.

use reqwest::Client;
use std::sync::Arc;

use crate::errors::AppResult;

use super::models::{TestServer, ThroughputResult};
use super::throughput::{run_progressive_ramp, ThroughputTracker};
use super::SpeedtestConfig;

/// Run upload speed test with progressive concurrency ramp-up.
///
/// Uploads data chunks to the server's upload endpoint,
/// measuring throughput across increasing concurrency levels.
///
/// # Arguments
/// * `client` - HTTP client configured with source IP binding
/// * `server` - Server to test against
/// * `config` - Test configuration
///
/// # Returns
/// Throughput result with best speed achieved and statistics.
pub async fn run_upload_test(
    client: &Client,
    server: &TestServer,
    config: &SpeedtestConfig,
) -> AppResult<ThroughputResult> {
    let upload_url = server.upload_url();

    tracing::info!(
        "[speedtest_progressive::upload] Starting upload test to {}",
        upload_url
    );

    let client = client.clone();
    let url = upload_url.clone();
    let chunk_size = config.upload_chunk_size;
    let timeout = config.request_timeout();

    let result = run_progressive_ramp(config, "upload", move |tracker| {
        let client = client.clone();
        let url = url.clone();
        upload_worker(client, url, tracker, chunk_size, timeout)
    })
    .await;

    tracing::info!(
        "[speedtest_progressive::upload] Upload complete: {:.2} Mbps",
        result.to_mbps()
    );

    Ok(result)
}

/// Single upload worker that posts chunks until shutdown.
async fn upload_worker(
    client: Client,
    url: String,
    tracker: Arc<ThroughputTracker>,
    chunk_size: usize,
    timeout: std::time::Duration,
) {
    let mut shutdown_rx = tracker.shutdown_receiver();

    // Pre-generate upload data (random-ish content)
    let upload_data = generate_upload_data(chunk_size);

    loop {
        // Check for shutdown before starting new request
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        match upload_single(&client, &url, &upload_data, &tracker, timeout).await {
            Ok(_) => {
                tracker.request_completed();
            }
            Err(e) => {
                tracing::trace!("[speedtest_progressive::upload] Request failed: {:?}", e);
                tracker.request_failed();
            }
        }

        // Check for shutdown after request
        if shutdown_rx.try_recv().is_ok() {
            break;
        }
    }
}

/// Upload a single chunk of data.
///
/// Note: We don't strictly check HTTP status codes - some speedtest servers
/// return non-2xx status but still accept the upload. This matches Python's behavior.
async fn upload_single(
    client: &Client,
    url: &str,
    data: &[u8],
    tracker: &ThroughputTracker,
    timeout: std::time::Duration,
) -> Result<(), reqwest::Error> {
    let data_len = data.len() as u64;

    // Ookla-style upload uses form encoding
    let result = client
        .post(url)
        .timeout(timeout)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(data.to_vec())
        .send()
        .await;

    match result {
        Ok(response) => {
            // Count bytes AFTER the request completes so only successfully
            // transmitted data is measured.  send().await blocks until the
            // full body has been written and the response headers arrive,
            // so by this point the data has crossed the wire.
            tracker.add_bytes(data_len);

            let status = response.status();
            if !status.is_success() {
                tracing::trace!(
                    "[speedtest_progressive::upload] HTTP {} (ignoring, bytes counted)",
                    status
                );
            }
            Ok(())
        }
        Err(e) => {
            // Request failed — don't count any bytes since the data may
            // not have been fully transmitted.
            tracing::trace!(
                "[speedtest_progressive::upload] Request failed, no bytes counted: {:?}",
                e
            );
            Err(e)
        }
    }
}

/// Generate upload data of specified size.
///
/// Creates pseudo-random data that compresses poorly (to prevent
/// server-side compression from affecting results).
fn generate_upload_data(size: usize) -> Vec<u8> {
    // Use a simple pattern that doesn't compress well
    // Format: "content0={random data}"
    let prefix = b"content0=";
    let data_size = size.saturating_sub(prefix.len());

    let mut data = Vec::with_capacity(size);
    data.extend_from_slice(prefix);

    // Generate pseudo-random bytes (not cryptographically secure, but fast)
    // Using a simple LCG-like pattern
    let mut seed: u64 = 0x123456789ABCDEF0;
    for _ in 0..data_size {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        data.push((seed >> 56) as u8);
    }

    data
}

/// Run a simple upload test without progressive ramp-up.
#[allow(dead_code)]
pub async fn run_simple_upload(
    client: &Client,
    server: &TestServer,
    config: &SpeedtestConfig,
    duration_secs: f64,
    concurrency: usize,
) -> AppResult<ThroughputResult> {
    use super::throughput::run_single_level;
    use std::time::Duration;

    let upload_url = server.upload_url();
    let client = client.clone();
    let url = upload_url;
    let chunk_size = config.upload_chunk_size;
    let timeout = config.request_timeout();

    let result = run_single_level(
        concurrency,
        Duration::from_secs_f64(duration_secs),
        Duration::from_millis(500),
        move |tracker| {
            let client = client.clone();
            let url = url.clone();
            upload_worker(client, url, tracker, chunk_size, timeout)
        },
    )
    .await;

    Ok(super::models::ThroughputResult {
        throughput_bps: result.throughput_bps,
        sustained_throughput_bps: result.throughput_bps, // Simple test doesn't sample
        best_concurrency: concurrency,
        requests_completed: result.requests_completed,
        requests_failed: result.requests_failed,
        levels: vec![result],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_upload_data_size() {
        let data = generate_upload_data(1000);
        assert_eq!(data.len(), 1000);
    }

    #[test]
    fn test_generate_upload_data_prefix() {
        let data = generate_upload_data(100);
        assert!(data.starts_with(b"content0="));
    }

    #[test]
    fn test_generate_upload_data_unique() {
        // Verify data is pseudo-random (not all same byte)
        let data = generate_upload_data(1000);
        let unique_bytes: std::collections::HashSet<u8> = data.iter().cloned().collect();
        // Should have many unique byte values
        assert!(unique_bytes.len() > 50);
    }

    #[test]
    fn test_upload_url_generation() {
        let server = super::super::models::TestServer {
            id: "1".to_string(),
            name: "Test".to_string(),
            sponsor: "ISP".to_string(),
            country: "US".to_string(),
            lat: 0.0,
            lon: 0.0,
            host: "speedtest.example.com:8080".to_string(),
            url: None,
        };

        let url = server.upload_url();
        assert_eq!(url, "http://speedtest.example.com:8080/upload.php");
    }
}
