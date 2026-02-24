//! Quick timing test to verify parallel browser fix
//!
//! Run with: cargo run --example test_browser_timing
//!
//! This is a minimal test that just measures browser launch timing
//! without requiring database setup or full quota check logic.

use std::time::Instant;

use net_ninja::clients::WebDriverClient;
use net_ninja::config::WebDriverSettings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,net_ninja=info")
        .init();

    println!("\n=== Browser Timing Test ===\n");
    println!("This test verifies that parallel browser launching works correctly.\n");

    let settings = WebDriverSettings {
        chrome_path: None,
        headless: true,
        auto_install: true,
    };

    // Test 1: Single browser (baseline)
    println!("Test 1: Single browser");
    println!("{}", "-".repeat(50));
    let start = Instant::now();
    {
        let driver = WebDriverClient::new_headless(&settings).await?;
        println!("Browser created in {:?}", start.elapsed());
        driver.quit().await?;
        println!("Browser closed in {:?}\n", start.elapsed());
    }
    let single_time = start.elapsed();

    // Wait a bit between tests
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Test 2: Two browsers in parallel
    println!("Test 2: Two browsers in parallel");
    println!("{}", "-".repeat(50));
    let start = Instant::now();
    let (r1, r2) = tokio::join!(
        async {
            let start = Instant::now();
            println!("[Browser 1] Starting...");
            match WebDriverClient::new_headless(&settings).await {
                Ok(driver) => {
                    println!("[Browser 1] Created in {:?}", start.elapsed());
                    let _ = driver.quit().await;
                    println!("[Browser 1] Closed in {:?}", start.elapsed());
                    Ok(())
                }
                Err(e) => Err(format!("Browser 1 failed: {}", e))
            }
        },
        async {
            let start = Instant::now();
            println!("[Browser 2] Starting...");
            match WebDriverClient::new_headless(&settings).await {
                Ok(driver) => {
                    println!("[Browser 2] Created in {:?}", start.elapsed());
                    let _ = driver.quit().await;
                    println!("[Browser 2] Closed in {:?}", start.elapsed());
                    Ok(())
                }
                Err(e) => Err(format!("Browser 2 failed: {}", e))
            }
        }
    );

    if let Err(e) = r1 {
        return Err(e.into());
    }
    if let Err(e) = r2 {
        return Err(e.into());
    }

    let parallel_time = start.elapsed();
    println!("Both browsers completed in {:?}\n", parallel_time);

    // Analysis
    println!("=== Results ===");
    println!("{}", "-".repeat(50));
    println!("Single browser time:  {:?}", single_time);
    println!("Parallel time:        {:?}", parallel_time);

    let speedup = single_time.as_secs_f64() * 2.0 / parallel_time.as_secs_f64();
    println!("Theoretical max:      {:?} (2x single)", single_time);
    println!("Actual speedup:       {:.2}x", speedup);

    println!("\n=== Analysis ===");
    if speedup >= 1.5 {
        println!("✓ PASS: Browsers launched in parallel!");
        println!("  Speedup of {:.2}x indicates concurrent execution.", speedup);
        println!("  The fix is working correctly.");
    } else if speedup >= 1.2 {
        println!("⚠ MARGINAL: Some parallelism detected ({:.2}x)", speedup);
        println!("  Expected >= 1.5x. There may still be contention.");
        println!("  Check system resources or increase timing measurements.");
    } else {
        println!("✗ FAIL: Browsers launched sequentially!");
        println!("  Speedup of {:.2}x indicates minimal parallelism.", speedup);
        println!("  The mutex may still be causing contention.");
        println!("\n  Expected behavior:");
        println!("    - Before fix: ~1.0x (sequential)");
        println!("    - After fix:  ~1.5-1.9x (parallel)");
    }

    println!("\nNote: Speedup < 2.0x is normal due to:");
    println!("  - Mutex serialization (500ms per browser)");
    println!("  - Process spawn overhead");
    println!("  - System scheduling variability");
    println!("  - Shared system resources (CPU, memory, disk)");

    println!("\n=== Test Complete ===\n");

    Ok(())
}
