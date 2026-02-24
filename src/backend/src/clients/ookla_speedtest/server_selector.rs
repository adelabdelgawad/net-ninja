//! Server selection based on distance and latency.
//!
//! Selects the best server by:
//! 1. Filtering servers within max distance
//! 2. Sorting by distance to find closest N servers
//! 3. Measuring latency to each
//! 4. Selecting the one with lowest latency

use reqwest::Client;

use crate::errors::AppResult;

use super::latency::measure_trimmed_latency;
use super::models::{RankedServer, TestServer};
use super::SpeedtestConfig;

/// Select the best server from the provided list.
///
/// # Algorithm
/// 1. Calculate distance from client to each server
/// 2. Filter out servers beyond `max_server_distance_km`
/// 3. Select the `closest_server_count` nearest servers
/// 4. Measure latency to each using trimmed median
/// 5. Return the server with lowest latency
///
/// # Arguments
/// * `client` - HTTP client for latency probes
/// * `servers` - List of available servers
/// * `client_lat` - Client latitude
/// * `client_lon` - Client longitude
/// * `config` - Configuration parameters
///
/// # Returns
/// The best server with its distance and latency, or an error if no servers available.
pub async fn select_best_server(
    client: &Client,
    servers: Vec<TestServer>,
    client_lat: f64,
    client_lon: f64,
    config: &SpeedtestConfig,
) -> AppResult<RankedServer> {
    if servers.is_empty() {
        return Err(crate::errors::AppError::Validation(
            "No servers provided for selection".to_string(),
        ));
    }

    tracing::info!(
        "[speedtest_progressive::server_selector] Selecting from {} servers (client location: {:.4}, {:.4})",
        servers.len(),
        client_lat,
        client_lon
    );

    // Calculate distances and filter
    let mut ranked: Vec<RankedServer> = servers
        .into_iter()
        .map(|server| {
            let distance = haversine_distance(client_lat, client_lon, server.lat, server.lon);
            RankedServer {
                server,
                distance_km: distance,
                latency_ms: None,
            }
        })
        .filter(|s| s.distance_km <= config.max_server_distance_km)
        .collect();

    if ranked.is_empty() {
        return Err(crate::errors::AppError::Validation(format!(
            "No servers within {} km of client location",
            config.max_server_distance_km
        )));
    }

    // Sort by distance
    ranked.sort_by(|a, b| {
        a.distance_km
            .partial_cmp(&b.distance_km)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take closest N servers
    let candidates: Vec<RankedServer> = ranked
        .into_iter()
        .take(config.closest_server_count)
        .collect();

    tracing::debug!(
        "[speedtest_progressive::server_selector] Testing latency to {} closest servers",
        candidates.len()
    );

    // Measure latency to each candidate
    let mut measured: Vec<RankedServer> = Vec::with_capacity(candidates.len());
    for mut candidate in candidates {
        let latency = measure_trimmed_latency(client, &candidate.server, config).await;
        candidate.latency_ms = latency;

        if latency.is_some() {
            tracing::debug!(
                "[speedtest_progressive::server_selector] {} ({}) - {:.1} km, {:.2} ms",
                candidate.server.name,
                candidate.server.sponsor,
                candidate.distance_km,
                latency.unwrap()
            );
            measured.push(candidate);
        } else {
            tracing::warn!(
                "[speedtest_progressive::server_selector] {} unreachable, skipping",
                candidate.server.name
            );
        }
    }

    if measured.is_empty() {
        return Err(crate::errors::AppError::Internal(
            "All candidate servers unreachable".to_string(),
        ));
    }

    // Sort by latency and select best
    measured.sort_by(|a, b| {
        a.latency_ms
            .unwrap_or(f64::MAX)
            .partial_cmp(&b.latency_ms.unwrap_or(f64::MAX))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let best = measured.into_iter().next().unwrap();

    tracing::info!(
        "[speedtest_progressive::server_selector] Selected: {} ({}) - {:.1} km, {:.2} ms",
        best.server.name,
        best.server.sponsor,
        best.distance_km,
        best.latency_ms.unwrap_or(0.0)
    );

    Ok(best)
}

/// Calculate distance between two points using Haversine formula.
///
/// Returns distance in kilometers.
pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_KM * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_same_point() {
        let distance = haversine_distance(40.7128, -74.0060, 40.7128, -74.0060);
        assert!(distance.abs() < 0.001);
    }

    #[test]
    fn test_haversine_nyc_to_la() {
        // NYC to LA is approximately 3940 km
        let distance = haversine_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!((distance - 3940.0).abs() < 50.0); // Within 50km tolerance
    }

    #[test]
    fn test_haversine_london_to_paris() {
        // London to Paris is approximately 343 km
        let distance = haversine_distance(51.5074, -0.1278, 48.8566, 2.3522);
        assert!((distance - 343.0).abs() < 20.0); // Within 20km tolerance
    }

    #[test]
    fn test_haversine_antipodes() {
        // Approximately half Earth's circumference
        let distance = haversine_distance(0.0, 0.0, 0.0, 180.0);
        assert!((distance - 20015.0).abs() < 100.0);
    }
}
