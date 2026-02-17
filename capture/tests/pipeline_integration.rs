use anyhow::Result;
use recall_capture::pipeline::continuous_capture;
use std::time::Duration;
use tokio::time::timeout;
use tracing_subscriber;

use recall_capture::monitor::list_monitors;

#[tokio::test]
async fn test_live_hardware_capture() -> Result<()> {
    // Setup logging to see output
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter("debug")
        .try_init();

    println!("Starting live hardware capture test...");

    let monitors = list_monitors().await;
    if monitors.is_empty() {
        println!("Skipping test: No monitors found (CI environment?)");
        return Ok(());
    }
    let monitor_id = monitors[0].id();
    println!("Using monitor ID: {}", monitor_id);

    // Run capture for 3 seconds
    // We expect it to run continuously, so we wrap in timeout
    let result = timeout(Duration::from_secs(3), continuous_capture(monitor_id, Duration::from_millis(100))).await;

    // Timeout is expected (as the loop is infinite)
    // If it returns Ok(Err), that means the internal loop failed
    match result {
        Ok(Err(e)) => {
            panic!("Capture loop failed: {}", e);
        }
        Ok(Ok(_)) => {
             // Should not happen unless loop breaks
             println!("Capture loop finished unexpectedly (but successfully)");
        }
        Err(_) => {
            println!("Capture ran for 3 seconds successfully (timeout reached)");
        }
    }

    // TODO: Connect to DB and verify frames were written
    // For now, we rely on the fact that it didn't panic or error out
    
    Ok(())
}
