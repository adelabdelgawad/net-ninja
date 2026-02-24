//! SpeedTestClient - High-level API for speed testing
//!
//! Provides a familiar interface that:
//! - Automatically fetches servers from speedtest.net
//! - Detects client location from public IP
//! - Uses progressive concurrency ramp-up internally

use std::net::IpAddr;

use reqwest::Client;

use crate::errors::{AppError, AppResult};

use super::{OoklaSpeedtestResult, SpeedtestConfig, SpeedtestRunner, TestServer};

const SPEEDTEST_CONFIG_URL: &str = "https://www.speedtest.net/speedtest-config.php";
const SPEEDTEST_SERVERS_URL: &str = "https://www.speedtest.net/speedtest-servers-static.php";

/// Result of a speed test with all relevant measurements.
#[derive(Debug, Clone)]
pub struct SpeedTestResult {
    /// Latency to the test server in milliseconds
    pub ping_ms: f64,

    /// Download speed in megabits per second (peak/link capacity)
    pub download_mbps: f64,

    /// Upload speed in megabits per second (peak/link capacity)
    pub upload_mbps: f64,

    /// Sustained download speed in megabits per second (Ookla-compatible trimmed mean)
    /// This typically matches what you'd see on speedtest.net
    pub sustained_download_mbps: f64,

    /// Sustained upload speed in megabits per second (Ookla-compatible trimmed mean)
    /// This typically matches what you'd see on speedtest.net
    pub sustained_upload_mbps: f64,

    /// Public IP address detected during the test
    pub public_ip: String,

    /// Name of the server used for testing (sponsor)
    pub server_name: String,

    /// Location of the server (city, country)
    pub server_location: String,

    /// Number of download requests completed
    pub download_requests_completed: u32,

    /// Number of download requests failed
    pub download_requests_failed: u32,

    /// Number of upload requests completed
    pub upload_requests_completed: u32,

    /// Number of upload requests failed
    pub upload_requests_failed: u32,
}

/// Client configuration from speedtest.net
#[derive(Debug, Clone, Default)]
struct ClientConfig {
    pub lat: f64,
    pub lon: f64,
    pub public_ip: Option<String>,
}

/// Main SpeedTest client with progressive ramp-up algorithm.
///
/// # Example
///
/// ```rust,no_run
/// use net_ninja::clients::SpeedTestClient;
///
/// #[tokio::main]
/// async fn main() {
///     let mut client = SpeedTestClient::new().unwrap();
///     let result = client.run().await.unwrap();
///     println!("Download: {:.2} Mbps", result.download_mbps);
///     println!("Upload: {:.2} Mbps", result.upload_mbps);
///     println!("Ping: {:.2} ms", result.ping_ms);
/// }
/// ```
pub struct SpeedTestClient {
    http_client: Client,
    source_address: Option<IpAddr>,
    config: SpeedtestConfig,
}

impl SpeedTestClient {
    /// Create a new SpeedTestClient with default settings.
    pub fn new() -> AppResult<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(AppError::HttpClient)?;

        Ok(Self {
            http_client,
            source_address: None,
            config: SpeedtestConfig::default(),
        })
    }

    /// Create a new SpeedTestClient bound to a specific source IP address.
    ///
    /// This is useful for multi-WAN setups where you want to test a specific connection.
    pub fn with_source_address(source_address: IpAddr) -> AppResult<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .local_address(source_address)
            .build()
            .map_err(AppError::HttpClient)?;

        Ok(Self {
            http_client,
            source_address: Some(source_address),
            config: SpeedtestConfig::default(),
        })
    }

    /// Run the complete speed test sequence.
    ///
    /// 1. Fetches client configuration (location, public IP) from speedtest.net
    /// 2. Fetches server list from speedtest.net
    /// 3. Selects best server based on distance and latency
    /// 4. Runs progressive download test
    /// 5. Runs progressive upload test
    pub async fn run(&mut self) -> AppResult<SpeedTestResult> {
        tracing::info!("[speedtest] Starting speed test sequence");

        // Phase 1: Get client configuration
        tracing::info!("[speedtest] Phase 1/3: Getting config from speedtest.net");
        let client_config = self.fetch_client_config().await?;
        tracing::info!(
            "[speedtest] Client location: {:.4}, {:.4}, IP: {:?}",
            client_config.lat,
            client_config.lon,
            client_config.public_ip
        );

        // Phase 2: Fetch servers
        tracing::info!("[speedtest] Phase 2/3: Fetching server list");
        let servers = self.fetch_servers().await?;
        tracing::info!("[speedtest] Found {} servers", servers.len());

        if servers.is_empty() {
            return Err(AppError::Internal("No speedtest servers available".to_string()));
        }

        // Phase 3: Run the progressive speedtest
        tracing::info!("[speedtest] Phase 3/3: Running progressive speedtest");
        let runner = self.create_runner(client_config.lat, client_config.lon)?;
        let result = runner.run(servers).await?;

        tracing::info!(
            "[speedtest] Complete: {:.2} Mbps down ({:.2} sustained), {:.2} Mbps up ({:.2} sustained), {:.2}ms ping",
            result.download_mbps,
            result.sustained_download_mbps,
            result.upload_mbps,
            result.sustained_upload_mbps,
            result.ping_ms
        );

        Ok(self.build_result(result, client_config))
    }

    /// Fetch client configuration from speedtest.net
    async fn fetch_client_config(&self) -> AppResult<ClientConfig> {
        let response = self
            .http_client
            .get(SPEEDTEST_CONFIG_URL)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch config: {}", e)))?;

        let text = response
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read config: {}", e)))?;

        // Parse client section from XML
        let client_start = text.find("<client ").ok_or_else(|| {
            AppError::Internal("Invalid config: missing client section".to_string())
        })?;
        let client_end = text[client_start..].find("/>").ok_or_else(|| {
            AppError::Internal("Invalid config: malformed client section".to_string())
        })? + client_start;
        let client_section = &text[client_start..=client_end + 1];

        let ip = extract_attr(client_section, "ip");
        let lat: f64 = extract_attr(client_section, "lat")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let lon: f64 = extract_attr(client_section, "lon")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        Ok(ClientConfig {
            lat,
            lon,
            public_ip: ip,
        })
    }

    /// Fetch servers from speedtest.net
    async fn fetch_servers(&self) -> AppResult<Vec<TestServer>> {
        let response = self
            .http_client
            .get(SPEEDTEST_SERVERS_URL)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch servers: {}", e)))?;

        let text = response
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read servers: {}", e)))?;

        let mut servers = Vec::new();

        for line in text.lines() {
            if line.contains("<server ") {
                if let Some(server) = parse_server_xml(line) {
                    servers.push(server);
                }
            }
        }

        Ok(servers)
    }

    /// Create the internal SpeedtestRunner
    fn create_runner(&self, lat: f64, lon: f64) -> AppResult<SpeedtestRunner> {
        match self.source_address {
            Some(addr) => SpeedtestRunner::with_source_address(addr, lat, lon, self.config.clone()),
            None => SpeedtestRunner::with_config(lat, lon, self.config.clone()),
        }
    }

    /// Build the final result struct
    fn build_result(&self, result: OoklaSpeedtestResult, config: ClientConfig) -> SpeedTestResult {
        SpeedTestResult {
            ping_ms: result.ping_ms,
            download_mbps: result.download_mbps,
            upload_mbps: result.upload_mbps,
            sustained_download_mbps: result.sustained_download_mbps,
            sustained_upload_mbps: result.sustained_upload_mbps,
            public_ip: config.public_ip.unwrap_or_default(),
            server_name: result.server_sponsor,
            server_location: format!("{}, {}", result.server_name, result.server_location),
            download_requests_completed: result.download_requests_completed,
            download_requests_failed: result.download_requests_failed,
            upload_requests_completed: result.upload_requests_completed,
            upload_requests_failed: result.upload_requests_failed,
        }
    }
}

impl Default for SpeedTestClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default SpeedTestClient")
    }
}

/// Extract an attribute value from XML
fn extract_attr(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = xml.find(&pattern)? + pattern.len();
    let end = xml[start..].find('"')? + start;
    Some(xml[start..end].to_string())
}

/// Parse a server XML element
fn parse_server_xml(xml: &str) -> Option<TestServer> {
    let url = extract_attr(xml, "url")?;
    let lat: f64 = extract_attr(xml, "lat")?.parse().ok()?;
    let lon: f64 = extract_attr(xml, "lon")?.parse().ok()?;
    let name = extract_attr(xml, "name")?;
    let country = extract_attr(xml, "country")?;
    let sponsor = extract_attr(xml, "sponsor")?;
    let id = extract_attr(xml, "id")?;
    let host = extract_attr(xml, "host")?;

    Some(TestServer {
        id,
        name,
        sponsor,
        country,
        lat,
        lon,
        host,
        url: Some(url),
    })
}
