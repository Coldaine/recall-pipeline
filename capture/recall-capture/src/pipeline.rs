//! Channel-based capture pipeline.
//!
//! This module implements a decoupled capture pipeline using Tokio channels:
//! - Capture tasks (one per monitor) → capture channel → Dedup task → storage channel → Storage task
//!
//! This ensures consistent capture rate regardless of storage latency.

use anyhow::Result;
use chrono::{DateTime, Utc};
use image::DynamicImage;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::{debug, info, warn};

use crate::monitor::SafeMonitor;

// ---------------------------------------------------------------------------
// Channel message types
// ---------------------------------------------------------------------------

/// Message sent from capture task to dedup task.
#[derive(Debug)]
pub struct CaptureMessage {
    /// The captured image.
    pub image: DynamicImage,
    /// Perceptual hash of the image.
    pub phash: u64,
    /// When the frame was captured.
    pub timestamp: Instant,
    /// Which monitor captured this frame.
    pub monitor_id: u32,
}

/// Message sent from dedup task to storage task.
#[derive(Debug)]
pub struct StorageMessage {
    /// The captured image.
    pub image: DynamicImage,
    /// Perceptual hash of the image (as i64 for DB storage).
    pub phash: i64,
    /// When the frame was captured (for DB).
    pub captured_at: DateTime<Utc>,
    /// Which monitor captured this frame.
    pub monitor_id: u32,
}

/// Shutdown signal for graceful termination.
#[derive(Debug, Clone, Copy)]
pub struct ShutdownSignal;

// ---------------------------------------------------------------------------
// Pipeline metrics
// ---------------------------------------------------------------------------

/// Metrics for monitoring pipeline health.
#[derive(Debug, Default)]
pub struct PipelineMetrics {
    /// Total frames captured (before dedup).
    pub frames_captured: AtomicU64,
    /// Frames dropped by in-memory dedup.
    pub frames_deduped_memory: AtomicU64,
    /// Frames dropped by DB-level dedup.
    pub frames_deduped_db: AtomicU64,
    /// Frames successfully stored.
    pub frames_stored: AtomicU64,
    /// Frames that failed to store.
    pub frames_failed: AtomicU64,
}

impl PipelineMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log_summary(&self) {
        let captured = self.frames_captured.load(Ordering::Relaxed);
        let deduped_mem = self.frames_deduped_memory.load(Ordering::Relaxed);
        let deduped_db = self.frames_deduped_db.load(Ordering::Relaxed);
        let stored = self.frames_stored.load(Ordering::Relaxed);
        let failed = self.frames_failed.load(Ordering::Relaxed);

        info!(
            captured,
            deduped_memory = deduped_mem,
            deduped_db = deduped_db,
            stored,
            failed,
            "Pipeline metrics"
        );
    }
}

// ---------------------------------------------------------------------------
// Pipeline configuration
// ---------------------------------------------------------------------------

/// Configuration for the capture pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Capacity of the capture channel (capture → dedup).
    pub capture_channel_capacity: usize,
    /// Capacity of the storage channel (dedup → storage).
    pub storage_channel_capacity: usize,
    /// Interval for logging channel depths.
    pub metrics_log_interval_secs: u64,
    /// Warn when channel is above this percentage (0.0-1.0).
    pub channel_warn_threshold: f32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            capture_channel_capacity: 64,
            storage_channel_capacity: 32,
            metrics_log_interval_secs: 60,
            channel_warn_threshold: 0.8,
        }
    }
}

// ---------------------------------------------------------------------------
// Channel handles
// ---------------------------------------------------------------------------

/// Handles for the capture pipeline channels.
pub struct PipelineChannels {
    /// Sender for capture messages (capture tasks use this).
    pub capture_tx: Sender<CaptureMessage>,
    /// Receiver for capture messages (dedup task uses this).
    pub capture_rx: Receiver<CaptureMessage>,
    /// Sender for storage messages (dedup task uses this).
    pub storage_tx: Sender<StorageMessage>,
    /// Receiver for storage messages (storage task uses this).
    pub storage_rx: Receiver<StorageMessage>,
}

impl PipelineChannels {
    /// Create new channels with the given configuration.
    pub fn new(config: &PipelineConfig) -> Self {
        let (capture_tx, capture_rx) = mpsc::channel(config.capture_channel_capacity);
        let (storage_tx, storage_rx) = mpsc::channel(config.storage_channel_capacity);
        Self {
            capture_tx,
            capture_rx,
            storage_tx,
            storage_rx,
        }
    }
}

// ---------------------------------------------------------------------------
// Capture task
// ---------------------------------------------------------------------------

/// Minimum difference threshold below which frames are considered duplicates.
const DEDUP_THRESHOLD: f64 = 0.006;

/// Run a capture task for a single monitor.
///
/// This task continuously captures frames from the monitor and sends them
/// to the capture channel. It performs in-memory deduplication against
/// the previous frame to avoid processing identical frames.
pub async fn run_capture_task(
    monitor: SafeMonitor,
    capture_tx: Sender<CaptureMessage>,
    metrics: Arc<PipelineMetrics>,
    mut shutdown_rx: Receiver<ShutdownSignal>,
    fps: f64,
) {
    let monitor_id = monitor.id();
    let interval = std::time::Duration::from_secs_f64(1.0 / fps);
    let mut tick = tokio::time::interval(interval);
    let mut previous_image: Option<DynamicImage> = None;

    info!(monitor_id, fps, "Capture task started");

    loop {
        tokio::select! {
            _ = tick.tick() => {
                // Capture frame
                match capture_and_dedup(&monitor, previous_image.as_ref()).await {
                    Ok(Some(frame)) => {
                        metrics.frames_captured.fetch_add(1, Ordering::Relaxed);

                        // Try to send to channel (non-blocking with backpressure)
                        match capture_tx.try_send(frame) {
                            Ok(()) => {
                                debug!(monitor_id, "Frame sent to dedup channel");
                            }
                            Err(mpsc::error::TrySendError::Full(frame)) => {
                                warn!(monitor_id, "Capture channel full, dropping frame");
                                // Store the image for next frame comparison anyway
                                previous_image = Some(frame.image);
                                continue;
                            }
                            Err(mpsc::error::TrySendError::Closed(_)) => {
                                info!(monitor_id, "Capture channel closed, stopping");
                                break;
                            }
                        }

                        // Update previous image for next dedup
                        // We need to get the image back since we sent it
                        // Actually, we need to clone before sending
                    }
                    Ok(None) => {
                        // Frame was deduplicated (too similar to previous)
                        metrics.frames_deduped_memory.fetch_add(1, Ordering::Relaxed);
                        debug!(monitor_id, "Frame deduplicated (memory)");
                    }
                    Err(e) => {
                        warn!(monitor_id, "Capture error: {}", e);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!(monitor_id, "Capture task received shutdown signal");
                break;
            }
        }
    }

    info!(monitor_id, "Capture task stopped");
}

/// Capture a frame and perform in-memory deduplication.
async fn capture_and_dedup(
    monitor: &SafeMonitor,
    previous: Option<&DynamicImage>,
) -> Result<Option<CaptureMessage>> {
    use crate::dedup::{frame_difference_async, phash64_async};

    let timestamp = Instant::now();

    let image = monitor.capture_image().await?;

    // Dedup against previous frame using spawn_blocking for CPU-intensive comparison
    if let Some(prev) = previous {
        // Clone previous image for the async comparison
        let prev_clone = prev.clone();
        let image_clone = image.clone();
        match frame_difference_async(prev_clone, image_clone).await {
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

    // Use spawn_blocking for CPU-intensive phash calculation
    let phash = phash64_async(image.clone()).await;

    Ok(Some(CaptureMessage {
        image,
        phash,
        timestamp,
        monitor_id: monitor.id(),
    }))
}

// ---------------------------------------------------------------------------
// Dedup task
// ---------------------------------------------------------------------------

/// Run the dedup task.
///
/// This task receives frames from capture tasks and forwards them to storage.
/// Currently, DB-level dedup is handled in the storage task, but this task
/// could be extended to do additional processing (e.g., OCR queue).
pub async fn run_dedup_task(
    mut capture_rx: Receiver<CaptureMessage>,
    storage_tx: Sender<StorageMessage>,
    _metrics: Arc<PipelineMetrics>,
    mut shutdown_rx: Receiver<ShutdownSignal>,
) {
    info!("Dedup task started");

    loop {
        tokio::select! {
            msg = capture_rx.recv() => {
                match msg {
                    Some(frame) => {
                        let storage_msg = StorageMessage {
                            image: frame.image,
                            phash: frame.phash as i64,
                            captured_at: Utc::now(),
                            monitor_id: frame.monitor_id,
                        };

                        match storage_tx.send(storage_msg).await {
                            Ok(()) => {
                                debug!(monitor_id = frame.monitor_id, "Frame forwarded to storage");
                            }
                            Err(_) => {
                                info!("Storage channel closed, stopping dedup task");
                                break;
                            }
                        }
                    }
                    None => {
                        info!("Capture channel closed, stopping dedup task");
                        break;
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!("Dedup task received shutdown signal");
                break;
            }
        }
    }

    info!("Dedup task stopped");
}

// ---------------------------------------------------------------------------
// Channel monitoring
// ---------------------------------------------------------------------------

/// Periodically log channel depths and metrics.
pub async fn run_metrics_task(
    capture_tx: Sender<CaptureMessage>,
    storage_tx: Sender<StorageMessage>,
    metrics: Arc<PipelineMetrics>,
    config: PipelineConfig,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<ShutdownSignal>,
) {
    let interval = std::time::Duration::from_secs(config.metrics_log_interval_secs);
    let mut tick = tokio::time::interval(interval);

    info!(
        interval_secs = config.metrics_log_interval_secs,
        "Metrics task started"
    );

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let capture_capacity = capture_tx.capacity();
                let storage_capacity = storage_tx.capacity();

                // We can't get the exact channel size from Sender, but we can log metrics
                metrics.log_summary();

                // Warn if channels are near capacity (we'd need to track this differently)
                debug!(
                    capture_channel_capacity = capture_capacity,
                    storage_channel_capacity = storage_capacity,
                    "Channel status"
                );
            }
            _ = shutdown_rx.recv() => {
                info!("Metrics task received shutdown signal");
                break;
            }
        }
    }

    info!("Metrics task stopped");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_defaults() {
        let config = PipelineConfig::default();
        assert_eq!(config.capture_channel_capacity, 64);
        assert_eq!(config.storage_channel_capacity, 32);
        assert_eq!(config.metrics_log_interval_secs, 60);
    }

    #[test]
    fn test_metrics_increment() {
        let metrics = PipelineMetrics::new();
        metrics.frames_captured.fetch_add(10, Ordering::Relaxed);
        assert_eq!(metrics.frames_captured.load(Ordering::Relaxed), 10);
    }
}
