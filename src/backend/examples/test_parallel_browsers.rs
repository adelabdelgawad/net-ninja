//! Test parallel browser instances to diagnose concurrency issues
//!
//! Run with: cargo run --example test_parallel_browsers
//!
//! This test launches multiple browser instances simultaneously to verify
//! that the WebDriver client handles parallel execution correctly.

use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::stream::{self, StreamExt};
use tokio::sync::Semaphore;

use net_ninja::clients::WebDriverClient;
use net_ninja::config::Settings;
use net_ninja::errors::AppResult;

/// Test result for a single browser instance
#[derive(Debug)]
struct BrowserTestResult {
    instance_id: usize,
    success: bool,
    duration_ms: u64,
    error: Option<String>,
    url_visited: Option<String>,
}

/// Test a single browser instance
async fn test_single_browser(
    instance_id: usize,
    settings: &Settings,
) -> BrowserTestResult {
    let start = Instant::now();

    println!("[Browser {}] Starting...", instance_id);

    // Create browser
    let driver = match WebDriverClient::new_headless(&settings.webdriver).await {
        Ok(d) => {
            println!("[Browser {}] Created successfully", instance_id);
            d
        }
        Err(e) => {
            let error_msg = format!("Failed to create browser: {}", e);
            println!("[Browser {}] ERROR: {}", instance_id, error_msg);
            return BrowserTestResult {
                instance_id,
                success: false,
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(error_msg),
                url_visited: None,
            };
        }
    };

    // Navigate to a simple page
    let test_url = "https://example.com";
    println!("[Browser {}] Navigating to {}...", instance_id, test_url);

    if let Err(e) = driver.navigate(test_url).await {
        let error_msg = format!("Navigation failed: {}", e);
        println!("[Browser {}] ERROR: {}", instance_id, error_msg);

        // Still try to close
        let _ = driver.quit().await;

        return BrowserTestResult {
            instance_id,
            success: false,
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some(error_msg),
            url_visited: None,
        };
    }

    // Get title to verify page loaded
    let title = match driver.get_title().await {
        Ok(t) => {
            println!("[Browser {}] Page title: {}", instance_id, t);
            Some(t)
        }
        Err(e) => {
            println!("[Browser {}] Warning: Could not get title: {}", instance_id, e);
            None
        }
    };

    // Get current URL
    let current_url = driver.get_current_url().await.ok();

    // Close browser
    println!("[Browser {}] Closing...", instance_id);
    if let Err(e) = driver.quit().await {
        let error_msg = format!("Failed to close browser: {}", e);
        println!("[Browser {}] Warning: {}", instance_id, error_msg);
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    println!("[Browser {}] Completed in {}ms", instance_id, duration_ms);

    BrowserTestResult {
        instance_id,
        success: true,
        duration_ms,
        error: None,
        url_visited: current_url,
    }
}

/// Run parallel browser tests
async fn run_parallel_test(
    num_browsers: usize,
    concurrency: usize,
    settings: &Settings,
) -> Vec<BrowserTestResult> {
    println!("\n=== Starting parallel test: {} browsers, {} concurrent ===\n",
        num_browsers, concurrency);

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let settings = Arc::new(settings.clone());

    let results: Vec<BrowserTestResult> = stream::iter(0..num_browsers)
        .map(|i| {
            let semaphore = semaphore.clone();
            let settings = settings.clone();
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                test_single_browser(i, &settings).await
            }
        })
        .buffer_unordered(num_browsers)
        .collect()
        .await;

    results
}

fn print_results(results: &[BrowserTestResult]) {
    println!("\n=== Results Summary ===\n");

    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    println!("Total: {} browsers", results.len());
    println!("Successful: {}", successful);
    println!("Failed: {}", failed);

    if failed > 0 {
        println!("\nFailed instances:");
        for r in results.iter().filter(|r| !r.success) {
            println!("  - Browser {}: {}",
                r.instance_id,
                r.error.as_deref().unwrap_or("Unknown error"));
        }
    }

    // Calculate timing stats
    let durations: Vec<u64> = results.iter().map(|r| r.duration_ms).collect();
    let avg = durations.iter().sum::<u64>() as f64 / durations.len() as f64;
    let min = durations.iter().min().unwrap_or(&0);
    let max = durations.iter().max().unwrap_or(&0);

    println!("\nTiming (ms):");
    println!("  Min: {}", min);
    println!("  Max: {}", max);
    println!("  Avg: {:.1}", avg);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,net_ninja=debug,chaser_oxide=debug")
        .init();

    println!("=== Parallel Browser Test ===\n");

    // Load settings
    let settings = Settings::load()?;
    println!("Settings loaded (headless: {})\n", settings.webdriver.headless);

    // Test 1: Sequential (baseline)
    println!("\n### Test 1: Sequential (2 browsers, concurrency=1) ###");
    let results = run_parallel_test(2, 1, &settings).await;
    print_results(&results);

    // Give system time to clean up
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test 2: Parallel with current concurrency setting
    println!("\n### Test 2: Parallel ({} browsers, concurrency={}) ###",
        4, settings.quota_check.concurrency);
    let results = run_parallel_test(4, settings.quota_check.concurrency, &settings).await;
    print_results(&results);

    // Give system time to clean up
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test 3: High concurrency stress test
    println!("\n### Test 3: Stress test (4 browsers, concurrency=4) ###");
    let results = run_parallel_test(4, 4, &settings).await;
    print_results(&results);

    println!("\n=== All tests completed ===");

    Ok(())
}
