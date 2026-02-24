//! Ookla-style Speedtest Client with Progressive Ramp-up
//!
//! A speedtest implementation inspired by Ookla's algorithm with:
//! - Progressive concurrency ramp-up (1→2→4→8)
//! - Trimmed median latency measurement
//! - Congestion-aware plateau detection
//! - Windowed throughput averaging
//! - Automatic server discovery from speedtest.net
//!
//! # Simple Usage
//!
//! ```rust,no_run
//! use net_ninja::clients::SpeedTestClient;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = SpeedTestClient::new().unwrap();
//!     let result = client.run().await.unwrap();
//!     println!("Download: {:.2} Mbps", result.download_mbps);
//!     println!("Upload: {:.2} Mbps", result.upload_mbps);
//!     println!("Ping: {:.2} ms", result.ping_ms);
//! }
//! ```
//!
//! # With Source IP Binding (Multi-WAN)
//!
//! ```rust,no_run
//! use net_ninja::clients::SpeedTestClient;
//! use std::net::IpAddr;
//!
//! let source_ip: IpAddr = "192.168.1.100".parse().unwrap();
//! let mut client = SpeedTestClient::with_source_address(source_ip).unwrap();
//! let result = client.run().await.unwrap();
//! ```

mod client;
mod config;
mod download;
mod latency;
mod models;
mod server_selector;
mod throughput;
mod upload;

// Primary exports - simple API for most users
pub use client::SpeedTestClient;

// Advanced exports - for custom configurations
pub use config::SpeedtestConfig;
pub use models::{OoklaSpeedtestResult, TestServer};

use std::net::IpAddr;

use reqwest::Client;

use crate::errors::{AppError, AppResult};

use download::run_download_test;
use server_selector::select_best_server;
use upload::run_upload_test;

/// User-Agent header required by speedtest servers.
/// Without this, many servers return HTTP 500 with empty body.
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// The main speedtest runner.
///
/// Orchestrates the complete speedtest flow:
/// 1. Server selection (distance + latency based)
/// 2. Latency measurement (trimmed median)
/// 3. Download test (progressive ramp-up)
/// 4. Upload test (progressive ramp-up)
pub struct SpeedtestRunner {
    client: Client,
    client_lat: f64,
    client_lon: f64,
    config: SpeedtestConfig,
    /// Source address for multi-WAN support (None = system default)
    source_address: Option<IpAddr>,
}

impl SpeedtestRunner {
    /// Create a new speedtest runner with default configuration.
    ///
    /// # Arguments
    /// * `client_lat` - Client latitude for server distance calculation
    /// * `client_lon` - Client longitude for server distance calculation
    pub fn new(client_lat: f64, client_lon: f64) -> AppResult<Self> {
        Self::with_config(client_lat, client_lon, SpeedtestConfig::default())
    }

    /// Create a runner with custom configuration.
    ///
    /// # Arguments
    /// * `client_lat` - Client latitude
    /// * `client_lon` - Client longitude
    /// * `config` - Custom speedtest configuration
    pub fn with_config(client_lat: f64, client_lon: f64, config: SpeedtestConfig) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(config.request_timeout())
            .connect_timeout(config.connect_timeout())
            .user_agent(USER_AGENT)
            .build()
            .map_err(AppError::HttpClient)?;

        Ok(Self {
            client,
            client_lat,
            client_lon,
            config,
            source_address: None,
        })
    }

    /// Create a runner bound to a specific source IP address.
    ///
    /// This is essential for multi-WAN setups where you want to test
    /// through a specific network interface.
    ///
    /// # Arguments
    /// * `source_address` - Local IP address to bind outgoing connections to
    /// * `client_lat` - Client latitude
    /// * `client_lon` - Client longitude
    /// * `config` - Custom speedtest configuration
    pub fn with_source_address(
        source_address: IpAddr,
        client_lat: f64,
        client_lon: f64,
        config: SpeedtestConfig,
    ) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(config.request_timeout())
            .connect_timeout(config.connect_timeout())
            .local_address(source_address)
            .user_agent(USER_AGENT)
            .build()
            .map_err(AppError::HttpClient)?;

        tracing::debug!(
            "[speedtest_progressive] Created client bound to source IP: {}",
            source_address
        );

        Ok(Self {
            client,
            client_lat,
            client_lon,
            config,
            source_address: Some(source_address),
        })
    }

    /// Run the complete speedtest.
    ///
    /// # Arguments
    /// * `servers` - List of servers to choose from
    ///
    /// # Returns
    /// Complete speedtest result including ping, download, upload, and server info.
    ///
    /// # Errors
    /// Returns error if no servers available or all phases fail.
    pub async fn run(&self, servers: Vec<TestServer>) -> AppResult<OoklaSpeedtestResult> {
        tracing::info!(
            "[speedtest_progressive] Starting speedtest with {} available servers",
            servers.len()
        );

        // Phase 1: Select best server
        tracing::info!("[speedtest_progressive] Phase 1/4: Selecting server");
        let selected = select_best_server(
            &self.client,
            servers,
            self.client_lat,
            self.client_lon,
            &self.config,
        )
        .await?;

        let ping_ms = selected.latency_ms.unwrap_or(0.0);

        tracing::info!(
            "[speedtest_progressive] Selected server: {} ({}) - {:.1}km, {:.2}ms",
            selected.server.name,
            selected.server.sponsor,
            selected.distance_km,
            ping_ms
        );

        // Phase 2: Download test
        tracing::info!("[speedtest_progressive] Phase 2/4: Download test");
        let download_result = run_download_test(&selected.server, &self.config, self.source_address).await;

        let (download_mbps, sustained_download_mbps, download_concurrency, download_completed, download_failed) =
            match download_result {
                Ok(result) => (
                    result.to_mbps(),
                    result.to_sustained_mbps(),
                    result.best_concurrency,
                    result.requests_completed,
                    result.requests_failed,
                ),
                Err(e) => {
                    tracing::warn!("[speedtest_progressive] Download test failed: {:?}", e);
                    (0.0, 0.0, 0, 0, 0)
                }
            };

        // Phase 3: Upload test
        tracing::info!("[speedtest_progressive] Phase 3/4: Upload test");
        let upload_result = run_upload_test(&self.client, &selected.server, &self.config).await;

        let (upload_mbps, sustained_upload_mbps, upload_concurrency, upload_completed, upload_failed) =
            match upload_result {
                Ok(result) => (
                    result.to_mbps(),
                    result.to_sustained_mbps(),
                    result.best_concurrency,
                    result.requests_completed,
                    result.requests_failed,
                ),
                Err(e) => {
                    tracing::warn!("[speedtest_progressive] Upload test failed: {:?}", e);
                    (0.0, 0.0, 0, 0, 0)
                }
            };

        // Phase 4: Compile results
        tracing::info!("[speedtest_progressive] Phase 4/4: Compiling results");

        let result = OoklaSpeedtestResult {
            ping_ms,
            download_mbps,
            upload_mbps,
            sustained_download_mbps,
            sustained_upload_mbps,
            server_name: selected.server.name,
            server_location: selected.server.country,
            server_sponsor: selected.server.sponsor,
            server_id: selected.server.id,
            server_distance_km: selected.distance_km,
            download_concurrency,
            upload_concurrency,
            download_requests_completed: download_completed,
            download_requests_failed: download_failed,
            upload_requests_completed: upload_completed,
            upload_requests_failed: upload_failed,
        };

        // Validate that at least one direction succeeded
        if !result.is_successful() {
            return Err(AppError::Internal(
                "Both download and upload tests failed".to_string(),
            ));
        }

        tracing::info!(
            "[speedtest_progressive] Complete: {:.2} Mbps down ({:.2} sustained), {:.2} Mbps up ({:.2} sustained), {:.2}ms ping",
            result.download_mbps,
            result.sustained_download_mbps,
            result.upload_mbps,
            result.sustained_upload_mbps,
            result.ping_ms
        );

        Ok(result)
    }

    /// Run download test only.
    pub async fn run_download_only(
        &self,
        server: &TestServer,
    ) -> AppResult<models::ThroughputResult> {
        run_download_test(server, &self.config, self.source_address).await
    }

    /// Run upload test only.
    pub async fn run_upload_only(&self, server: &TestServer) -> AppResult<models::ThroughputResult> {
        run_upload_test(&self.client, server, &self.config).await
    }

    /// Measure latency to a specific server.
    pub async fn measure_latency(&self, server: &TestServer) -> Option<f64> {
        latency::measure_trimmed_latency(&self.client, server, &self.config).await
    }

    /// Get the current configuration.
    pub fn config(&self) -> &SpeedtestConfig {
        &self.config
    }

    /// Get client location.
    pub fn client_location(&self) -> (f64, f64) {
        (self.client_lat, self.client_lon)
    }
}

/// Builder for creating SpeedtestRunner with fluent API.
pub struct SpeedtestRunnerBuilder {
    client_lat: f64,
    client_lon: f64,
    config: SpeedtestConfig,
    source_address: Option<IpAddr>,
}

impl SpeedtestRunnerBuilder {
    /// Create a new builder with required location parameters.
    pub fn new(client_lat: f64, client_lon: f64) -> Self {
        Self {
            client_lat,
            client_lon,
            config: SpeedtestConfig::default(),
            source_address: None,
        }
    }

    /// Set custom configuration.
    pub fn config(mut self, config: SpeedtestConfig) -> Self {
        self.config = config;
        self
    }

    /// Set source IP address for multi-WAN testing.
    pub fn source_address(mut self, addr: IpAddr) -> Self {
        self.source_address = Some(addr);
        self
    }

    /// Set maximum concurrency level.
    pub fn max_concurrency(mut self, max: usize) -> Self {
        self.config.max_concurrency = max;
        self
    }

    /// Set number of latency probes.
    pub fn latency_probes(mut self, count: usize) -> Self {
        self.config.latency_probe_count = count;
        self
    }

    /// Set throughput gain threshold for plateau detection.
    pub fn throughput_threshold(mut self, threshold: f64) -> Self {
        self.config.throughput_gain_threshold = threshold;
        self
    }

    /// Build the SpeedtestRunner.
    pub fn build(self) -> AppResult<SpeedtestRunner> {
        match self.source_address {
            Some(addr) => {
                SpeedtestRunner::with_source_address(addr, self.client_lat, self.client_lon, self.config)
            }
            None => SpeedtestRunner::with_config(self.client_lat, self.client_lon, self.config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let runner = SpeedtestRunnerBuilder::new(40.7, -74.0).build().unwrap();

        assert_eq!(runner.client_location(), (40.7, -74.0));
        // Default is fixed concurrency=4 (matching speedtest-rs methodology)
        assert_eq!(runner.config().max_concurrency, 4);
    }

    #[test]
    fn test_builder_custom() {
        let runner = SpeedtestRunnerBuilder::new(40.7, -74.0)
            .max_concurrency(16)
            .latency_probes(10)
            .build()
            .unwrap();

        assert_eq!(runner.config().max_concurrency, 16);
        assert_eq!(runner.config().latency_probe_count, 10);
    }

    #[test]
    fn test_result_is_successful() {
        let mut result = OoklaSpeedtestResult {
            ping_ms: 10.0,
            download_mbps: 100.0,
            upload_mbps: 50.0,
            sustained_download_mbps: 90.0,
            sustained_upload_mbps: 45.0,
            server_name: "Test".to_string(),
            server_location: "US".to_string(),
            server_sponsor: "ISP".to_string(),
            server_id: "1".to_string(),
            server_distance_km: 10.0,
            download_concurrency: 4,
            upload_concurrency: 4,
            download_requests_completed: 10,
            download_requests_failed: 0,
            upload_requests_completed: 10,
            upload_requests_failed: 0,
        };

        assert!(result.is_successful());

        result.download_mbps = 0.0;
        assert!(result.is_successful()); // Upload still works

        result.upload_mbps = 0.0;
        assert!(!result.is_successful()); // Both failed
    }
}
