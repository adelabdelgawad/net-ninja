//! Test to diagnose CDP concurrency issues
//!
//! Run with: RUST_LOG=debug cargo run --example test_cdp_concurrency
//!
//! This test creates browsers and tracks exactly when each operation happens

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Barrier;

use net_ninja::clients::WebDriverClient;
use net_ninja::config::WebDriverSettings;

async fn browser_with_timing(
    id: usize,
    settings: Arc<WebDriverSettings>,
    start_barrier: Arc<Barrier>,
) -> Result<(), Box<dyn std::error::Error>> {
    let global_start = Instant::now();

    // Step 1: Wait for all browsers to be ready to start
    println!("[Browser {}] Waiting at start barrier...", id);
    start_barrier.wait().await;
    println!("[Browser {}] [t={}ms] START: Creating browser", id, global_start.elapsed().as_millis());

    // Step 2: Create browser
    let create_start = Instant::now();
    let driver = WebDriverClient::new_headless(&settings).await?;
    let create_time = create_start.elapsed();
    println!("[Browser {}] [t={}ms] CREATED: Browser created in {:?}",
        id, global_start.elapsed().as_millis(), create_time);

    // Step 3: First navigation
    println!("[Browser {}] [t={}ms] NAV1_START: Navigating to example.com",
        id, global_start.elapsed().as_millis());
    let nav1_start = Instant::now();
    driver.navigate("https://example.com").await?;
    let nav1_time = nav1_start.elapsed();
    println!("[Browser {}] [t={}ms] NAV1_END: First navigation took {:?}",
        id, global_start.elapsed().as_millis(), nav1_time);

    // Step 4: Get title
    println!("[Browser {}] [t={}ms] TITLE_START: Getting page title",
        id, global_start.elapsed().as_millis());
    let title_start = Instant::now();
    let title = driver.get_title().await?;
    let title_time = title_start.elapsed();
    println!("[Browser {}] [t={}ms] TITLE_END: Got title '{}' in {:?}",
        id, global_start.elapsed().as_millis(), title, title_time);

    // Step 5: Second navigation
    println!("[Browser {}] [t={}ms] NAV2_START: Navigating to iana.org",
        id, global_start.elapsed().as_millis());
    let nav2_start = Instant::now();
    driver.navigate("https://www.iana.org/domains/reserved").await?;
    let nav2_time = nav2_start.elapsed();
    println!("[Browser {}] [t={}ms] NAV2_END: Second navigation took {:?}",
        id, global_start.elapsed().as_millis(), nav2_time);

    // Step 6: Close
    println!("[Browser {}] [t={}ms] CLOSE_START: Closing browser",
        id, global_start.elapsed().as_millis());
    let close_start = Instant::now();
    driver.quit().await?;
    let close_time = close_start.elapsed();
    println!("[Browser {}] [t={}ms] CLOSE_END: Closed in {:?}",
        id, global_start.elapsed().as_millis(), close_time);

    let total = global_start.elapsed();
    println!("[Browser {}] [t={}ms] COMPLETE: Total time {:?}\n",
        id, total.as_millis(), total);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info,net_ninja=debug,chaser_oxide=debug")
        .with_target(false)
        .init();

    println!("\n=== CDP Concurrency Diagnostic Test ===\n");

    let settings = Arc::new(WebDriverSettings {
        chrome_path: None,
        headless: true,
        auto_install: true,
    });

    println!("Launching 2 browsers simultaneously...\n");

    // Barrier ensures both browsers start at the same time
    let barrier = Arc::new(Barrier::new(2));
    let overall_start = Instant::now();

    let (r1, r2) = tokio::join!(
        browser_with_timing(1, settings.clone(), barrier.clone()),
        browser_with_timing(2, settings.clone(), barrier.clone())
    );

    let total_time = overall_start.elapsed();

    println!("\n=== Final Results ===\n");
    println!("Browser 1: {:?}", r1);
    println!("Browser 2: {:?}", r2);
    println!("Total time: {:?}", total_time);

    println!("\n=== Diagnostic Analysis ===\n");
    println!("Look at the timestamps above to identify the bottleneck:\n");

    println!("1. BROWSER CREATION (CREATED events):");
    println!("   - If times differ by >1000ms: Launch mutex is still too long");
    println!("   - If times are within ~500ms: Launch parallelism is working\n");

    println!("2. FIRST NAVIGATION (NAV1_START to NAV1_END):");
    println!("   - If Browser 2's NAV1_START is delayed until AFTER Browser 1's NAV1_END:");
    println!("     -> Navigation is SEQUENTIAL (CDP or chaser-oxide bottleneck)");
    println!("   - If both NAV1_START times are within ~1000ms of each other:");
    println!("     -> Navigation is PARALLEL (good!)\n");

    println!("3. OPERATIONS TIMING:");
    println!("   - If Browser 2's operations (TITLE_START, NAV2_START) only happen");
    println!("     AFTER Browser 1 completes: There's serialization in CDP operations");
    println!("   - If operations overlap: CDP concurrency is working\n");

    println!("4. EXPECTED PATTERNS:");
    println!("   PARALLEL (good):");
    println!("     [Browser 1] [t=200ms] NAV1_START");
    println!("     [Browser 2] [t=300ms] NAV1_START  <- Both navigating ~same time");
    println!("     [Browser 1] [t=2500ms] NAV1_END");
    println!("     [Browser 2] [t=2600ms] NAV1_END\n");

    println!("   SEQUENTIAL (bad):");
    println!("     [Browser 1] [t=200ms] NAV1_START");
    println!("     [Browser 1] [t=2500ms] NAV1_END");
    println!("     [Browser 2] [t=2600ms] NAV1_START  <- Waits for Browser 1!");
    println!("     [Browser 2] [t=4900ms] NAV1_END");

    Ok(())
}
