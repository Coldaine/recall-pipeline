use anyhow::Result;
use recall_capture::monitor::list_monitors;

#[tokio::test]
async fn test_live_hardware_capture() -> Result<()> {
    println!("Starting live hardware capture test...");

    let monitors = list_monitors().await?;
    if monitors.is_empty() {
        println!("Skipping test: No monitors found (CI environment?)");
        return Ok(());
    }
    
    println!("Found {} monitor(s)", monitors.len());
    
    // Test all monitors
    for (idx, monitor) in monitors.iter().enumerate() {
        println!("\n[Monitor {}] {} ({} x {})", 
            idx, 
            monitor.info().name, 
            monitor.info().width, 
            monitor.info().height);

        // Capture an actual screenshot
        println!("  Capturing screenshot...");
        let image = monitor.capture_image().await?;
        
        println!("  ✓ Captured: {} x {}", image.width(), image.height());
        
        // Verify we got a real image
        assert!(image.width() > 0, "Image should have non-zero width");
        assert!(image.height() > 0, "Image should have non-zero height");
        assert_eq!(image.color(), image::ColorType::Rgba8, "Image should be RGBA8");
    }
    
    println!("\n✓ All {} monitor(s) successfully captured!", monitors.len());
    
    Ok(())
}
