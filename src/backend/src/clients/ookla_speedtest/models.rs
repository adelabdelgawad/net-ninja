//! Data models for the Ookla-style progressive speedtest.
//!
//! Contains input server configuration and output result structures.

use serde::{Deserialize, Serialize};

/// A speedtest server to test against.
///
/// This is the input format - servers are preconfigured rather than scraped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestServer {
    /// Unique server identifier
    pub id: String,

    /// Server name/description
    pub name: String,

    /// Server sponsor (ISP or organization)
    pub sponsor: String,

    /// Server country
    pub country: String,

    /// Server latitude
    pub lat: f64,

    /// Server longitude
    pub lon: f64,

    /// Server host (e.g., "speedtest.example.com:8080")
    pub host: String,

    /// Optional direct URL override (if not using standard Ookla paths)
    pub url: Option<String>,
}

impl TestServer {
    /// Create a new test server.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        sponsor: impl Into<String>,
        country: impl Into<String>,
        lat: f64,
        lon: f64,
        host: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            sponsor: sponsor.into(),
            country: country.into(),
            lat,
            lon,
            host: host.into(),
            url: None,
        }
    }

    /// Get the base URL for this server.
    pub fn base_url(&self) -> String {
        if let Some(ref url) = self.url {
            // Strip trailing path component if present
            url.rfind('/')
                .map(|i| url[..=i].to_string())
                .unwrap_or_else(|| format!("{}/", url))
        } else {
            format!("http://{}/", self.host)
        }
    }

    /// Get the latency test URL.
    ///
    /// Uses the upload URL with `?latency` query parameter, matching Ookla's protocol.
    /// Note: Many servers return 404/500 for this endpoint but still work for downloads.
    pub fn latency_url(&self) -> String {
        if let Some(ref url) = self.url {
            format!("{}?latency", url)
        } else {
            format!("{}upload.php?latency", self.base_url())
        }
    }

    /// Get the download test URL with specified size.
    ///
    /// Ookla's protocol appends the download file to the upload URL, not the base URL.
    /// e.g., upload.php/random4000x4000.jpg
    pub fn download_url(&self, size: &str) -> String {
        if let Some(ref url) = self.url {
            format!("{}/random{}.jpg", url.trim_end_matches('/'), size)
        } else {
            format!("{}upload.php/random{}.jpg", self.base_url(), size)
        }
    }

    /// Get the upload test URL.
    pub fn upload_url(&self) -> String {
        format!("{}upload.php", self.base_url())
    }
}

/// Internal server with computed metadata.
#[derive(Debug, Clone)]
pub struct RankedServer {
    /// The original server configuration
    pub server: TestServer,

    /// Distance from client in kilometers
    pub distance_km: f64,

    /// Measured latency in milliseconds (None if not yet measured)
    pub latency_ms: Option<f64>,
}

/// Result of a single ramp level measurement.
#[derive(Debug, Clone)]
pub struct RampLevelResult {
    /// Concurrency level tested
    pub concurrency: usize,

    /// Measured throughput in bytes per second
    pub throughput_bps: f64,

    /// Number of successful requests
    pub requests_completed: u32,

    /// Number of failed requests
    pub requests_failed: u32,
}

/// Result of the progressive throughput test (download or upload).
#[derive(Debug, Clone)]
pub struct ThroughputResult {
    /// Best throughput achieved in bytes per second (link capacity / peak speed)
    pub throughput_bps: f64,

    /// Sustained throughput in bytes per second (Ookla-compatible trimmed mean)
    /// This discards the slowest 30% and fastest 10% of samples.
    pub sustained_throughput_bps: f64,

    /// Concurrency level that achieved best throughput
    pub best_concurrency: usize,

    /// Total requests completed across all levels
    pub requests_completed: u32,

    /// Total requests failed across all levels
    pub requests_failed: u32,

    /// Results from each ramp level
    pub levels: Vec<RampLevelResult>,
}

impl ThroughputResult {
    /// Convert peak throughput to megabits per second.
    ///
    /// Uses decimal units (1,000,000 bits = 1 Mbps) as per networking industry standard.
    /// This matches speedtest.net and ISP speed ratings.
    ///
    /// This returns the **peak/link capacity** speed (total bytes / time).
    pub fn to_mbps(&self) -> f64 {
        (self.throughput_bps * 8.0) / 1_000_000.0
    }

    /// Convert sustained throughput to megabits per second.
    ///
    /// This returns the **Ookla-compatible** speed (trimmed mean of samples).
    /// The sustained speed typically matches what you'd see on speedtest.net.
    pub fn to_sustained_mbps(&self) -> f64 {
        (self.sustained_throughput_bps * 8.0) / 1_000_000.0
    }
}

/// Final result of a complete speedtest run.
#[derive(Debug, Clone, Serialize)]
pub struct OoklaSpeedtestResult {
    /// Latency to the selected server in milliseconds
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

    /// Selected server name
    pub server_name: String,

    /// Selected server location (country)
    pub server_location: String,

    /// Selected server sponsor
    pub server_sponsor: String,

    /// Server ID
    pub server_id: String,

    /// Distance to server in kilometers
    pub server_distance_km: f64,

    /// Concurrency level used for download
    pub download_concurrency: usize,

    /// Concurrency level used for upload
    pub upload_concurrency: usize,

    /// Total download requests completed
    pub download_requests_completed: u32,

    /// Total download requests failed
    pub download_requests_failed: u32,

    /// Total upload requests completed
    pub upload_requests_completed: u32,

    /// Total upload requests failed
    pub upload_requests_failed: u32,
}

impl OoklaSpeedtestResult {
    /// Check if the test was successful (at least one direction measured).
    pub fn is_successful(&self) -> bool {
        self.download_mbps > 0.0 || self.upload_mbps > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_urls_without_custom_url() {
        // When no custom URL is provided, uses host-based URLs
        let server = TestServer::new(
            "1234",
            "Test Server",
            "Test ISP",
            "US",
            40.7128,
            -74.0060,
            "speedtest.example.com:8080",
        );

        assert_eq!(server.base_url(), "http://speedtest.example.com:8080/");
        // Latency uses upload.php?latency when no custom URL
        assert_eq!(
            server.latency_url(),
            "http://speedtest.example.com:8080/upload.php?latency"
        );
        // Download appends to upload URL
        assert_eq!(
            server.download_url("4000x4000"),
            "http://speedtest.example.com:8080/upload.php/random4000x4000.jpg"
        );
        assert_eq!(
            server.upload_url(),
            "http://speedtest.example.com:8080/upload.php"
        );
    }

    #[test]
    fn test_server_urls_with_custom_url() {
        // When custom URL is provided (like from speedtest.net server list)
        let mut server = TestServer::new(
            "1234",
            "Test Server",
            "Test ISP",
            "US",
            40.7128,
            -74.0060,
            "speedtest.example.com:8080",
        );
        server.url = Some("http://speedtest.example.com:8080/speedtest/upload.php".to_string());

        assert_eq!(server.base_url(), "http://speedtest.example.com:8080/speedtest/");
        // Latency uses full URL + ?latency
        assert_eq!(
            server.latency_url(),
            "http://speedtest.example.com:8080/speedtest/upload.php?latency"
        );
        // Download appends to upload URL
        assert_eq!(
            server.download_url("4000x4000"),
            "http://speedtest.example.com:8080/speedtest/upload.php/random4000x4000.jpg"
        );
        assert_eq!(
            server.upload_url(),
            "http://speedtest.example.com:8080/speedtest/upload.php"
        );
    }

    #[test]
    fn test_server_base_url_stripping() {
        let mut server = TestServer::new("1", "Test", "ISP", "US", 0.0, 0.0, "host:80");
        server.url = Some("https://custom.example.com/speedtest/upload.php".to_string());

        // base_url strips trailing file to get directory
        assert_eq!(server.base_url(), "https://custom.example.com/speedtest/");
    }

    #[test]
    fn test_throughput_to_mbps() {
        let result = ThroughputResult {
            throughput_bps: 10_000_000.0, // 10 MB/s (bytes per second)
            sustained_throughput_bps: 9_000_000.0, // 9 MB/s sustained
            best_concurrency: 4,
            requests_completed: 10,
            requests_failed: 0,
            levels: vec![],
        };

        // 10 MB/s * 8 = 80 Mbps (decimal, networking standard)
        let mbps = result.to_mbps();
        assert!((mbps - 80.0).abs() < 0.01);

        // 9 MB/s * 8 = 72 Mbps sustained
        let sustained_mbps = result.to_sustained_mbps();
        assert!((sustained_mbps - 72.0).abs() < 0.01);
    }
}
