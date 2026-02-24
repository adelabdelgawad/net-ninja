//! Download speed test with progressive ramp-up.
//!
//! Streams large file chunks while tracking throughput.
//! Creates new HTTP client per request to match speedtest-rs methodology.

use futures_util::StreamExt;
use reqwest::Client;
use std::net::IpAddr;
use std::sync::Arc;

use crate::errors::AppResult;

use super::models::{TestServer, ThroughputResult};
use super::throughput::{run_progressive_ramp, ThroughputTracker};
use super::SpeedtestConfig;

/// User-Agent header required by speedtest servers.
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Run download speed test with progressive concurrency ramp-up.
///
/// Downloads `/random{size}x{size}.jpg` from the server using multiple file sizes
/// (matching speedtest-rs methodology) to include HTTP overhead for realistic results.
/// Creates a new HTTP client for each request to match speedtest-rs behavior.
///
/// # Arguments
/// * `server` - Server to test against
/// * `config` - Test configuration
/// * `source_address` - Optional source IP for multi-WAN support
///
/// # Returns
/// Throughput result with best speed achieved and statistics.
pub async fn run_download_test(
    server: &TestServer,
    config: &SpeedtestConfig,
    source_address: Option<IpAddr>,
) -> AppResult<ThroughputResult> {
    // Build URLs for all download sizes (matching speedtest-rs methodology)
    let download_urls: Vec<String> = config
        .download_sizes
        .iter()
        .map(|size| server.download_url(&format!("{}x{}", size, size)))
        .collect();

    tracing::info!(
        "[speedtest_progressive::download] Starting download test with {} file sizes",
        download_urls.len()
    );

    let urls = Arc::new(download_urls);
    let timeout = config.request_timeout();
    let connect_timeout = config.connect_timeout();

    let result = run_progressive_ramp(config, "download", move |tracker| {
        let urls = Arc::clone(&urls);
        download_worker_multi_size(urls, tracker, timeout, connect_timeout, source_address)
    })
    .await;

    tracing::info!(
        "[speedtest_progressive::download] Download complete: {:.2} Mbps",
        result.to_mbps()
    );

    Ok(result)
}

/// Download worker that cycles through multiple file sizes (matching speedtest-rs).
/// Creates a new HTTP client for each request to match speedtest-rs behavior
/// (includes TCP connection overhead for realistic results).
async fn download_worker_multi_size(
    urls: Arc<Vec<String>>,
    tracker: Arc<ThroughputTracker>,
    timeout: std::time::Duration,
    connect_timeout: std::time::Duration,
    source_address: Option<IpAddr>,
) {
    let mut shutdown_rx = tracker.shutdown_receiver();
    let mut url_index = 0;

    loop {
        // Check for shutdown before starting new request
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        // Create new client per request (matching speedtest-rs methodology)
        // This includes TCP connection overhead in the measurement
        let client = match create_client(timeout, connect_timeout, source_address) {
            Ok(c) => c,
            Err(e) => {
                tracing::trace!("[speedtest_progressive::download] Failed to create client: {:?}", e);
                tracker.request_failed();
                continue;
            }
        };

        // Cycle through URLs (different file sizes)
        let url = &urls[url_index % urls.len()];
        url_index += 1;

        match download_single(&client, url, &tracker, timeout).await {
            Ok(_) => {
                tracker.request_completed();
            }
            Err(e) => {
                tracing::trace!("[speedtest_progressive::download] Request failed: {:?}", e);
                tracker.request_failed();
            }
        }

        // Check for shutdown after request
        if shutdown_rx.try_recv().is_ok() {
            break;
        }
    }
}

/// Create a new HTTP client (matching speedtest-rs behavior).
fn create_client(
    timeout: std::time::Duration,
    connect_timeout: std::time::Duration,
    source_address: Option<IpAddr>,
) -> Result<Client, reqwest::Error> {
    let mut builder = Client::builder()
        .timeout(timeout)
        .connect_timeout(connect_timeout)
        .user_agent(USER_AGENT);

    if let Some(addr) = source_address {
        builder = builder.local_address(addr);
    }

    builder.build()
}

/// Download a single file, streaming and counting bytes.
///
/// Note: We don't check HTTP status codes - some speedtest servers return
/// non-2xx status but still send valid data. This matches Python's behavior.
async fn download_single(
    client: &Client,
    url: &str,
    tracker: &ThroughputTracker,
    timeout: std::time::Duration,
) -> Result<u64, reqwest::Error> {
    // Use Connection: close to match speedtest-rs behavior (new TCP connection per request)
    // This includes TCP slow-start overhead in the measurement for more realistic results
    let response = client
        .get(url)
        .header("Connection", "close")
        .timeout(timeout)
        .send()
        .await?;

    // Log status but don't fail - servers may return 500 but still send data
    let status = response.status();
    if !status.is_success() {
        tracing::trace!(
            "[speedtest_progressive::download] HTTP {} (ignoring, reading body anyway)",
            status
        );
    }

    let mut stream = response.bytes_stream();
    let mut total_bytes: u64 = 0;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                let len = chunk.len() as u64;
                total_bytes += len;
                tracker.add_bytes(len);
            }
            Err(e) => {
                // Partial download - still count what we got
                if total_bytes > 0 {
                    tracing::trace!(
                        "[speedtest_progressive::download] Stream error after {} bytes: {:?}",
                        total_bytes,
                        e
                    );
                    // Return success with partial data
                    return Ok(total_bytes);
                }
                return Err(e);
            }
        }
    }

    Ok(total_bytes)
}

/// Run a simple download test without progressive ramp-up.
/// Useful for quick verification or when simple throughput is sufficient.
#[allow(dead_code)]
pub async fn run_simple_download(
    server: &TestServer,
    config: &SpeedtestConfig,
    source_address: Option<IpAddr>,
    duration_secs: f64,
    concurrency: usize,
) -> AppResult<ThroughputResult> {
    use super::throughput::run_single_level;
    use std::time::Duration;

    // Build URLs for all download sizes
    let download_urls: Vec<String> = config
        .download_sizes
        .iter()
        .map(|size| server.download_url(&format!("{}x{}", size, size)))
        .collect();

    let urls = Arc::new(download_urls);
    let timeout = config.request_timeout();
    let connect_timeout = config.connect_timeout();

    let result = run_single_level(
        concurrency,
        Duration::from_secs_f64(duration_secs),
        Duration::from_millis(500),
        move |tracker| {
            let urls = Arc::clone(&urls);
            download_worker_multi_size(urls, tracker, timeout, connect_timeout, source_address)
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
    fn test_download_url_generation_with_custom_url() {
        // With custom URL (like from speedtest.net server list)
        let server = TestServer {
            id: "1".to_string(),
            name: "Test".to_string(),
            sponsor: "ISP".to_string(),
            country: "US".to_string(),
            lat: 0.0,
            lon: 0.0,
            host: "speedtest.example.com:8080".to_string(),
            url: Some("http://speedtest.example.com:8080/speedtest/upload.php".to_string()),
        };

        let url = server.download_url("4000x4000");
        // Ookla protocol appends download file to upload URL
        assert_eq!(
            url,
            "http://speedtest.example.com:8080/speedtest/upload.php/random4000x4000.jpg"
        );
    }

    #[test]
    fn test_download_url_generation_without_custom_url() {
        // Without custom URL (host-based fallback)
        let server = TestServer {
            id: "1".to_string(),
            name: "Test".to_string(),
            sponsor: "ISP".to_string(),
            country: "US".to_string(),
            lat: 0.0,
            lon: 0.0,
            host: "speedtest.example.com:8080".to_string(),
            url: None,
        };

        let url = server.download_url("4000x4000");
        assert_eq!(
            url,
            "http://speedtest.example.com:8080/upload.php/random4000x4000.jpg"
        );
    }
}
