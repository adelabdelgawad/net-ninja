use std::net::{IpAddr, ToSocketAddrs};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use crate::errors::{AppError, AppResult};

pub struct NetworkDiagnostics;

impl NetworkDiagnostics {
    /// Test DNS resolution for a host
    pub async fn test_dns_resolution(host: &str) -> AppResult<Vec<IpAddr>> {
        tracing::info!("[diagnostics] Testing DNS resolution for {}", host);
        let start = Instant::now();

        match tokio::task::spawn_blocking({
            let host = host.to_string();
            move || (host.as_str(), 443).to_socket_addrs()
        })
        .await
        {
            Ok(Ok(addrs)) => {
                let ips: Vec<IpAddr> = addrs.map(|s| s.ip()).collect();
                let elapsed = start.elapsed();
                tracing::info!("[diagnostics] DNS resolved to {:?} in {:?}", ips, elapsed);
                Ok(ips)
            }
            Ok(Err(e)) => {
                tracing::error!("[diagnostics] DNS resolution failed: {:?}", e);
                Err(AppError::DnsFailure {
                    host: host.to_string(),
                    message: e.to_string(),
                })
            }
            Err(e) => {
                tracing::error!("[diagnostics] DNS task panicked: {:?}", e);
                Err(AppError::DnsFailure {
                    host: host.to_string(),
                    message: format!("Task panic: {}", e),
                })
            }
        }
    }

    /// Test TCP connectivity to a host:port
    pub async fn test_tcp_connection(host: &str, port: u16, timeout_secs: u64) -> AppResult<Duration> {
        tracing::info!("[diagnostics] Testing TCP connection to {}:{} (timeout: {}s)", host, port, timeout_secs);

        let addr = format!("{}:{}", host, port);
        let timeout = Duration::from_secs(timeout_secs);
        let start = Instant::now();

        match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
            Ok(Ok(_stream)) => {
                let elapsed = start.elapsed();
                tracing::info!("[diagnostics] TCP connection successful in {:?}", elapsed);
                Ok(elapsed)
            }
            Ok(Err(e)) => {
                tracing::error!("[diagnostics] TCP connection failed: {:?}", e);
                if e.kind() == std::io::ErrorKind::ConnectionRefused {
                    Err(AppError::ConnectionRefused {
                        host: host.to_string(),
                        port,
                    })
                } else {
                    Err(AppError::Internal(format!("TCP error: {}", e)))
                }
            }
            Err(_) => {
                tracing::error!("[diagnostics] TCP connection timeout after {}s", timeout_secs);
                Err(AppError::ConnectionTimeout {
                    host: host.to_string(),
                    timeout_secs,
                })
            }
        }
    }

    /// Check system proxy environment variables
    pub fn check_proxy_settings() -> Option<String> {
        tracing::info!("[diagnostics] Checking proxy environment variables");

        for var in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy"] {
            if let Ok(proxy) = std::env::var(var) {
                tracing::info!("[diagnostics] Found proxy: {}={}", var, proxy);
                return Some(proxy);
            }
        }

        tracing::info!("[diagnostics] No proxy environment variables set");
        None
    }

    /// Full diagnostic check before speed test
    pub async fn run_diagnostics(host: &str) -> AppResult<DiagnosticReport> {
        tracing::info!("[diagnostics] Running full diagnostics for {}", host);

        let mut report = DiagnosticReport {
            host: host.to_string(),
            dns_resolved: false,
            dns_ips: Vec::new(),
            tcp_reachable: false,
            tcp_latency_ms: None,
            proxy_detected: None,
            errors: Vec::new(),
        };

        // Check proxy
        report.proxy_detected = Self::check_proxy_settings();

        // Test DNS
        match Self::test_dns_resolution(host).await {
            Ok(ips) => {
                report.dns_resolved = true;
                report.dns_ips = ips;
            }
            Err(e) => {
                report.errors.push(format!("DNS: {}", e));
                return Ok(report); // Stop here if DNS fails
            }
        }

        // Test TCP
        match Self::test_tcp_connection(host, 443, 15).await {
            Ok(latency) => {
                report.tcp_reachable = true;
                report.tcp_latency_ms = Some(latency.as_millis() as u64);
            }
            Err(e) => {
                report.errors.push(format!("TCP: {}", e));
            }
        }

        tracing::info!("[diagnostics] Diagnostics complete: {:?}", report);
        Ok(report)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiagnosticReport {
    pub host: String,
    pub dns_resolved: bool,
    pub dns_ips: Vec<IpAddr>,
    pub tcp_reachable: bool,
    pub tcp_latency_ms: Option<u64>,
    pub proxy_detected: Option<String>,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_resolution() {
        // Test DNS resolution for a known host
        let result = NetworkDiagnostics::test_dns_resolution("google.com").await;
        assert!(result.is_ok(), "DNS resolution should succeed for google.com");
        let ips = result.unwrap();
        assert!(!ips.is_empty(), "Should resolve at least one IP");
    }

    #[tokio::test]
    async fn test_tcp_connection() {
        // Test TCP connection to a known host
        let result = NetworkDiagnostics::test_tcp_connection("google.com", 443, 10).await;
        assert!(result.is_ok(), "TCP connection should succeed to google.com:443");
    }

    #[tokio::test]
    async fn test_full_diagnostics() {
        // Test full diagnostics for speedtest.net
        let result = NetworkDiagnostics::run_diagnostics("www.speedtest.net").await;
        assert!(result.is_ok(), "Full diagnostics should complete");

        let report = result.unwrap();
        println!("Diagnostic Report:");
        println!("  Host: {}", report.host);
        println!("  DNS Resolved: {}", report.dns_resolved);
        println!("  DNS IPs: {:?}", report.dns_ips);
        println!("  TCP Reachable: {}", report.tcp_reachable);
        println!("  TCP Latency: {:?} ms", report.tcp_latency_ms);
        println!("  Proxy Detected: {:?}", report.proxy_detected);
        println!("  Errors: {:?}", report.errors);
    }
}
