use crate::dedup::{frame_difference, phash64};
use crate::monitor::SafeMonitor;
use anyhow::Result;
use image::DynamicImage;
use std::time::Instant;
use tracing::{debug, warn};

/// Result of capturing and deduplicating a single frame.
pub struct CaptureFrame {
    pub image: DynamicImage,
    pub phash: u64,
    pub timestamp: Instant,
    pub monitor_id: u32,
}

/// Minimum difference threshold below which frames are considered duplicates.
const DEDUP_THRESHOLD: f64 = 0.006;

/// Capture a screenshot from the given monitor, returning None if the frame
/// is too similar to `previous` (i.e. screen hasn't changed).
pub async fn capture_frame(
    monitor: &SafeMonitor,
    previous: Option<&DynamicImage>,
) -> Result<Option<CaptureFrame>> {
    let timestamp = Instant::now();

    let image = match monitor.capture_image().await {
        Ok(img) => img,
        Err(e) => {
            warn!("Failed to capture monitor {}: {}", monitor.id(), e);
            return Err(e);
        }
    };

    // Dedup against previous frame
    if let Some(prev) = previous {
        match frame_difference(prev, &image) {
            Ok(diff) if diff < DEDUP_THRESHOLD => {
                debug!(
                    "Skipping frame from monitor {} (diff={:.4} < {:.4})",
                    monitor.id(),
                    diff,
                    DEDUP_THRESHOLD,
                );
                return Ok(None);
            }
            Ok(diff) => {
                debug!("Frame diff={:.4}, capturing", diff);
            }
            Err(e) => {
                debug!("Image comparison failed ({}), capturing anyway", e);
            }
        }
    }

    let phash = phash64(&image);

    Ok(Some(CaptureFrame {
        image,
        phash,
        timestamp,
        monitor_id: monitor.id(),
    }))
}
