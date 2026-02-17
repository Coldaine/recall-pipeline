use crate::frame_comparer::{FrameComparer, FrameComparisonConfig};
use crate::monitor::{get_monitor_by_id, SafeMonitor};
use anyhow::Result;
use chrono::{DateTime, Utc};
use image::DynamicImage;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

pub struct CaptureEvent {
    pub image: DynamicImage,
    pub timestamp: DateTime<Utc>,
    pub frame_number: u64,
}

pub async fn continuous_capture(
    monitor_id: u32,
    interval: Duration,
) -> Result<()> {
    let mut frame_counter: u64 = 0;
    
    // Default config: downscale by 4, hash early exit enabled
    let mut frame_comparer = FrameComparer::new(FrameComparisonConfig {
        downscale_factor: 4,
        ..Default::default()
    });

    let max_skip_duration = Duration::from_secs(10);
    let mut last_capture_time = Instant::now();

    info!("Starting capture on monitor {}", monitor_id);

    let mut monitor = match get_monitor_by_id(monitor_id).await {
        Some(m) => m,
        None => {
            error!("Monitor {} not found", monitor_id);
            return Err(anyhow::anyhow!("Monitor not found"));
        }
    };

    let mut consecutive_failures = 0;
    const MAX_RETRIES: u32 = 3;

    loop {
        let capture_start = Instant::now();
        let captured_at = Utc::now();

        // 1. Capture
        let image = match capture_monitoring_safe(&mut monitor, monitor_id).await {
            Ok(img) => {
                consecutive_failures = 0;
                img
            },
            Err(e) => {
                consecutive_failures += 1;
                error!("Capture failed ({}): {}", consecutive_failures, e);
                if consecutive_failures > 10 {
                     return Err(anyhow::anyhow!("Too many consecutive capture failures"));
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        // 2. Compare
        let diff = frame_comparer.compare(&image);
        let skip_threshold = 0.01; // 1% difference
        
        let time_since_last = last_capture_time.elapsed();
        let force_capture = time_since_last >= max_skip_duration;

        if diff < skip_threshold && !force_capture {
            debug!("Skipping frame {} (diff: {:.4})", frame_counter, diff);
            frame_counter += 1;
            tokio::time::sleep(interval).await;
            continue;
        }

        // 3. Process (Stub for DB write)
        last_capture_time = Instant::now();
        info!("captured frame {} (diff: {:.4}, forced: {})", frame_counter, diff, force_capture);

        // TODO: Write to Postgres here
        // write_frame_to_db(&image, captured_at).await?;

        frame_counter += 1;
        
        let elapsed = capture_start.elapsed();
        if elapsed < interval {
            tokio::time::sleep(interval - elapsed).await;
        }
    }
}

async fn capture_monitoring_safe(monitor: &mut SafeMonitor, _monitor_id: u32) -> Result<DynamicImage> {
    for attempt in 0..3 {
        match monitor.capture_image().await {
            Ok(img) => return Ok(img),
            Err(e) => {
                debug!("Capture attempt {} failed: {}", attempt, e);
                // Try refresh
                if let Err(re) = monitor.refresh().await {
                    debug!("Refresh failed: {}", re);
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
    Err(anyhow::anyhow!("Failed into capture after retries"))
}
