//! Integration test for parallel WebDriver instances
//!
//! Run with: cargo test --test integration_tests webdriver_parallel

use std::time::Instant;
use tokio::sync::Semaphore;
use std::sync::Arc;
use futures_util::stream::{self, StreamExt};

use net_ninja::clients::WebDriverClient;
use net_ninja::config::WebDriverSettings;

/// Test that multiple browsers can launch in parallel
#[tokio::test]
#[ignore] // Requires Chrome to be installed
async fn test_parallel_browser_launch() {
    // Initialize logging for debugging
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info,net_ninja=debug")
        .try_init();

    let settings = WebDriverSettings {
        chrome_path: None,
        headless: true,
        auto_install: true,
    };

    let num_browsers = 3;
    let concurrency = 2;

    println!("\n=== Testing {} browsers with concurrency={} ===\n", num_browsers, concurrency);

    let start = Instant::now();
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let results: Vec<Result<(), String>> = stream::iter(0..num_browsers)
        .map(|i| {
            let semaphore = semaphore.clone();
            let settings = settings.clone();
            async move {
                let _permit = semaphore.acquire().await.unwrap();

                let browser_start = Instant::now();
                println!("[Browser {}] Starting...", i);

                // Create browser
                let driver = match WebDriverClient::new_headless(&settings).await {
                    Ok(d) => {
                        let elapsed = browser_start.elapsed();
                        println!("[Browser {}] Created in {:?}", i, elapsed);
                        d
                    }
                    Err(e) => {
                        println!("[Browser {}] Failed: {}", i, e);
                        return Err(format!("Browser {} failed: {}", i, e));
                    }
                };

                // Navigate to test page
                if let Err(e) = driver.navigate("https://example.com").await {
                    println!("[Browser {}] Navigation failed: {}", i, e);
                    let _ = driver.quit().await;
                    return Err(format!("Browser {} navigation failed: {}", i, e));
                }

                // Get title to verify page loaded
                match driver.get_title().await {
                    Ok(title) => println!("[Browser {}] Title: {}", i, title),
                    Err(e) => println!("[Browser {}] Warning: Could not get title: {}", i, e),
                }

                // Close browser
                if let Err(e) = driver.quit().await {
                    println!("[Browser {}] Warning: Failed to close: {}", i, e);
                }

                let total_elapsed = browser_start.elapsed();
                println!("[Browser {}] Completed in {:?}", i, total_elapsed);

                Ok(())
            }
        })
        .buffer_unordered(num_browsers)
        .collect()
        .await;

    let total_elapsed = start.elapsed();
    println!("\n=== All browsers completed in {:?} ===\n", total_elapsed);

    // Verify all succeeded
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.len() - successful;

    println!("Results: {} successful, {} failed", successful, failed);

    assert!(
        failed == 0,
        "Some browsers failed: {:?}",
        results.iter().filter(|r| r.is_err()).collect::<Vec<_>>()
    );

    // Verify parallel execution was faster than sequential
    // With concurrency=2, 3 browsers should take ~1.5x the time of 1 browser
    // Not 3x (which would be sequential)
    // Allow generous margin since CI environments are slow
    let max_expected = std::time::Duration::from_secs(20); // Very generous for CI

    assert!(
        total_elapsed < max_expected,
        "Parallel execution took too long: {:?} (expected < {:?}). This suggests browsers launched sequentially!",
        total_elapsed,
        max_expected
    );

    println!("\n✓ Parallel browser test passed!");
}

/// Test that browsers with unique profiles don't interfere
#[tokio::test]
#[ignore] // Requires Chrome to be installed
async fn test_browser_isolation() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info,net_ninja=debug")
        .try_init();

    let settings = WebDriverSettings {
        chrome_path: None,
        headless: true,
        auto_install: true,
    };

    println!("\n=== Testing browser isolation ===\n");

    // Launch 2 browsers simultaneously
    let (result1, result2) = tokio::join!(
        async {
            let driver = WebDriverClient::new_headless(&settings).await?;
            driver.navigate("https://example.com").await?;
            let title = driver.get_title().await?;
            driver.quit().await?;
            Ok::<String, Box<dyn std::error::Error>>(title)
        },
        async {
            let driver = WebDriverClient::new_headless(&settings).await?;
            driver.navigate("https://www.iana.org/domains/reserved").await?;
            let title = driver.get_title().await?;
            driver.quit().await?;
            Ok::<String, Box<dyn std::error::Error>>(title)
        }
    );

    // Both should succeed
    assert!(result1.is_ok(), "Browser 1 failed: {:?}", result1);
    assert!(result2.is_ok(), "Browser 2 failed: {:?}", result2);

    // They should have different titles (proving isolation)
    let title1 = result1.unwrap();
    let title2 = result2.unwrap();

    println!("Browser 1 title: {}", title1);
    println!("Browser 2 title: {}", title2);

    assert_ne!(
        title1, title2,
        "Browsers should have navigated to different pages"
    );

    println!("\n✓ Browser isolation test passed!");
}

/// Benchmark: measure actual parallelization speedup
#[tokio::test]
#[ignore] // Requires Chrome to be installed
async fn test_parallel_speedup() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info,net_ninja=debug")
        .try_init();

    let settings = WebDriverSettings {
        chrome_path: None,
        headless: true,
        auto_install: true,
    };

    println!("\n=== Parallel Speedup Benchmark ===\n");

    // Test 1: Sequential (concurrency=1)
    println!("Running 2 browsers sequentially...");
    let sequential_start = Instant::now();
    for i in 0..2 {
        let driver = WebDriverClient::new_headless(&settings).await.unwrap();
        driver.navigate("https://example.com").await.unwrap();
        driver.quit().await.unwrap();
        println!("  Sequential browser {} completed", i);
    }
    let sequential_time = sequential_start.elapsed();
    println!("Sequential time: {:?}\n", sequential_time);

    // Test 2: Parallel (concurrency=2)
    println!("Running 2 browsers in parallel...");
    let parallel_start = Instant::now();
    let (_r1, _r2) = tokio::join!(
        async {
            let driver = WebDriverClient::new_headless(&settings).await.unwrap();
            driver.navigate("https://example.com").await.unwrap();
            driver.quit().await.unwrap();
        },
        async {
            let driver = WebDriverClient::new_headless(&settings).await.unwrap();
            driver.navigate("https://example.com").await.unwrap();
            driver.quit().await.unwrap();
        }
    );
    let parallel_time = parallel_start.elapsed();
    println!("Parallel time: {:?}\n", parallel_time);

    // Calculate speedup
    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);

    // Parallel should be at least 1.3x faster (conservative, should be closer to 2x)
    // This accounts for overhead and system variability
    assert!(
        speedup >= 1.3,
        "Parallel execution should be faster! Speedup: {:.2}x (expected >= 1.3x). \
         This suggests browsers are launching sequentially due to mutex contention.",
        speedup
    );

    println!("\n✓ Parallel speedup test passed!");
}
