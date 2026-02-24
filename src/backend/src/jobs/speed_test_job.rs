use std::net::IpAddr;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::clients::SpeedTestClient;
use crate::config::Settings;
use crate::db::create_pool;
use crate::errors::AppResult;
use crate::models::{CreateSpeedTestResultRequest, Line};
use crate::services::{LineService, LogService, SpeedTestService};

pub async fn run(_settings: &Settings) -> AppResult<()> {
    let process_id = Uuid::new_v4();

    // Create database pool for this job run
    let pool = create_pool().await?;

    LogService::info(&pool, process_id, "speed_test_job::run", "Starting speed test job").await?;

    // Get all lines
    let lines = LineService::get_all_with_credentials(&pool).await?;

    LogService::info(
        &pool,
        process_id,
        "speed_test_job::run",
        &format!("Found {} lines to test", lines.len()),
    )
    .await?;

    // Run speed tests concurrently for all lines
    let mut handles = Vec::new();

    for line in lines {
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            run_speed_test_for_line(&pool, &line).await
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await {
            Ok(Ok(())) => success_count += 1,
            Ok(Err(e)) => {
                tracing::error!("Speed test failed: {:?}", e);
                error_count += 1;
            }
            Err(e) => {
                tracing::error!("Task panicked: {:?}", e);
                error_count += 1;
            }
        }
    }

    LogService::info(
        &pool,
        process_id,
        "speed_test_job::run",
        &format!(
            "Speed test job completed: {} success, {} errors",
            success_count, error_count
        ),
    )
    .await?;

    Ok(())
}

async fn run_speed_test_for_line(pool: &SqlitePool, line: &Line) -> AppResult<()> {
    let process_id = Uuid::new_v4();

    LogService::info_for_line(
        pool,
        process_id,
        line.id,
        "run_speed_test_for_line",
        &format!(
            "Starting speed test for line: {} (IP: {:?})",
            line.name, line.ip_address
        ),
    )
    .await?;

    // Create SpeedTest client with source address binding if IP is configured
    let mut client: SpeedTestClient = match &line.ip_address {
        Some(ip_str) => {
            match ip_str.parse::<IpAddr>() {
                Ok(ip) => {
                    LogService::info_for_line(
                        pool,
                        process_id,
                        line.id,
                        "run_speed_test_for_line",
                        &format!("Binding speed test to IP: {}", ip),
                    )
                    .await?;
                    SpeedTestClient::with_source_address(ip)?
                }
                Err(e) => {
                    LogService::warning_for_line(
                        pool,
                        process_id,
                        line.id,
                        "run_speed_test_for_line",
                        &format!("Invalid IP address '{}': {}, using default", ip_str, e),
                    )
                    .await?;
                    SpeedTestClient::new()?
                }
            }
        }
        None => {
            LogService::info_for_line(
                pool,
                process_id,
                line.id,
                "run_speed_test_for_line",
                "No IP address configured, using default interface",
            )
            .await?;
            SpeedTestClient::new()?
        }
    };

    // Run the speed test
    match client.run().await {
        Ok(result) => {
            LogService::info_for_line(
                pool,
                process_id,
                line.id,
                "run_speed_test_for_line",
                &format!(
                    "Speed test completed for {}: {:.2} Mbps down, {:.2} Mbps up, {:.2} ms ping",
                    line.name, result.download_mbps, result.upload_mbps, result.ping_ms
                ),
            )
            .await?;

            // Store the result
            let request = CreateSpeedTestResultRequest {
                line_id: line.id,
                process_id,
                download_speed: Some(result.download_mbps),
                upload_speed: Some(result.upload_mbps),
                ping: Some(result.ping_ms),
                server_name: Some(result.server_name),
                server_location: Some(result.server_location),
                public_ip: Some(result.public_ip),
                status: Some("success".to_string()),
                error_message: None,
            };

            SpeedTestService::create(pool, request).await?;

            Ok(())
        }
        Err(e) => {
            LogService::error_for_line(
                pool,
                process_id,
                line.id,
                "run_speed_test_for_line",
                &format!("Speed test failed for {}: {:?}", line.name, e),
            )
            .await?;

            // Store failed result with zeros
            let request = CreateSpeedTestResultRequest {
                line_id: line.id,
                process_id,
                download_speed: Some(0.0),
                upload_speed: Some(0.0),
                ping: Some(0.0),
                server_name: None,
                server_location: None,
                public_ip: None,
                status: Some("failed".to_string()),
                error_message: Some(format!("{:?}", e)),
            };

            SpeedTestService::create(pool, request).await?;

            Err(e)
        }
    }
}
