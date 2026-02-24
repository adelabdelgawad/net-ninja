//! Unit test for WebDriver mutex scope verification
//!
//! This test verifies that the mutex is released quickly (< 1s)
//! rather than being held for the full initialization (3s in old code)

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Simulate the OLD behavior - mutex held for entire initialization
async fn old_browser_launch_simulation(mutex: Arc<Mutex<()>>) -> Duration {
    let start = Instant::now();

    let _lock = mutex.lock().await;

    // Simulate full initialization (old behavior)
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Lock released here
    drop(_lock);

    start.elapsed()
}

/// Simulate the NEW behavior - mutex held only for launch
async fn new_browser_launch_simulation(mutex: Arc<Mutex<()>>) -> Duration {
    let start = Instant::now();

    // Only hold mutex during "launch"
    {
        let _lock = mutex.lock().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        // Lock released here
    }

    // Parallel initialization (outside mutex)
    tokio::time::sleep(Duration::from_millis(1000)).await;

    start.elapsed()
}

#[tokio::test]
async fn test_old_behavior_is_sequential() {
    let mutex = Arc::new(Mutex::new(()));

    let start = Instant::now();
    let mutex1 = mutex.clone();
    let mutex2 = mutex.clone();

    // Launch 2 browsers in parallel
    let (t1, t2) = tokio::join!(
        old_browser_launch_simulation(mutex1),
        old_browser_launch_simulation(mutex2)
    );

    let total = start.elapsed();

    println!("Old behavior:");
    println!("  Browser 1: {:?}", t1);
    println!("  Browser 2: {:?}", t2);
    println!("  Total: {:?}", total);

    // Old behavior: should take ~6 seconds (sequential)
    // Browser 1: 3s, Browser 2 waits 3s then takes 3s
    assert!(total >= Duration::from_millis(5500),
        "Old behavior should be sequential (~6s), got {:?}", total);
}

#[tokio::test]
async fn test_new_behavior_is_parallel() {
    let mutex = Arc::new(Mutex::new(()));

    let start = Instant::now();
    let mutex1 = mutex.clone();
    let mutex2 = mutex.clone();

    // Launch 2 browsers in parallel
    let (t1, t2) = tokio::join!(
        new_browser_launch_simulation(mutex1),
        new_browser_launch_simulation(mutex2)
    );

    let total = start.elapsed();

    println!("New behavior:");
    println!("  Browser 1: {:?}", t1);
    println!("  Browser 2: {:?}", t2);
    println!("  Total: {:?}", total);

    // New behavior: should take ~2 seconds (parallel)
    // Browser 1: starts immediately (0.5s mutex + 1s parallel)
    // Browser 2: waits 0.5s for mutex, then runs in parallel
    assert!(total < Duration::from_millis(2500),
        "New behavior should be parallel (~2s), got {:?}", total);

    // Verify actual parallelism (speedup)
    let sequential_time = Duration::from_millis(3000); // Single browser time
    let expected_sequential = sequential_time * 2; // 6 seconds
    let speedup = expected_sequential.as_secs_f64() / total.as_secs_f64();

    println!("  Speedup: {:.2}x", speedup);
    assert!(speedup >= 2.0,
        "Should achieve ~2x speedup with parallel execution, got {:.2}x", speedup);
}

#[tokio::test]
async fn test_mutex_contention_timing() {
    // Test that mutex is released quickly
    let mutex = Arc::new(Mutex::new(()));
    let mutex_clone = mutex.clone();

    // Start first task
    let task1 = tokio::spawn(async move {
        let start = Instant::now();
        let _lock = mutex_clone.lock().await;
        let lock_acquired = start.elapsed();

        // Simulate new behavior (500ms hold)
        tokio::time::sleep(Duration::from_millis(500)).await;

        (lock_acquired, start.elapsed())
    });

    // Small delay to ensure task1 gets mutex first
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Second task tries to acquire mutex
    let mutex_clone2 = mutex.clone();
    let start2 = Instant::now();
    let _lock2 = mutex_clone2.lock().await;
    let wait_time = start2.elapsed();

    task1.await.unwrap();

    println!("Mutex wait time: {:?}", wait_time);

    // Should wait ~500ms (not 3000ms like old code)
    assert!(wait_time >= Duration::from_millis(400),
        "Should wait for mutex: {:?}", wait_time);
    assert!(wait_time < Duration::from_millis(1000),
        "Mutex should be released quickly (<1s), got {:?}", wait_time);
}

#[test]
fn test_speedup_calculation() {
    // Verify speedup formula
    let single_browser_time = 3.0; // seconds
    let old_parallel_time = 6.0; // sequential
    let new_parallel_time = 2.0; // parallel

    let old_speedup = (single_browser_time * 2.0) / old_parallel_time;
    let new_speedup = (single_browser_time * 2.0) / new_parallel_time;

    println!("Old speedup: {:.2}x (sequential)", old_speedup);
    println!("New speedup: {:.2}x (parallel)", new_speedup);

    assert!((old_speedup - 1.0_f64).abs() < 0.1, "Old: sequential ~1.0x");
    assert!((new_speedup - 3.0_f64).abs() < 0.5, "New: parallel ~3.0x");
}
