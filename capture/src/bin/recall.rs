use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use recall_capture::{
    monitor::list_monitors, pipeline::run_metrics_task, CaptureMessage, PipelineChannels,
    PipelineConfig, PipelineMetrics, ShutdownSignal, StorageMessage,
};
use recall_store::{create_storage, ImageStorage, Storage};

/// Recall capture daemon -- screenshots, dedup, store.
/// TODO: [CONFIG] Replace CLI args with config file (TOML/YAML) for per-monitor settings
#[derive(Parser, Debug)]
#[command(name = "recall", version, about)]
struct Args {
    /// Directory for JPEG image storage
    #[arg(long, default_value = "/var/lib/recall/data")]
    data_dir: PathBuf,

    /// Capture rate in frames per second (global, TODO: per-monitor)
    #[arg(long, default_value_t = 0.5)]
    fps: f64,

    /// Days to keep captured data before cleanup
    #[arg(long, default_value_t = 30)]
    retention_days: u32,

    /// JPEG quality (1-100)
    #[arg(long, default_value_t = 85)]
    jpeg_quality: u8,

    /// Hamming-distance window for DB-level dedup (seconds)
    #[arg(long, default_value_t = 10)]
    dedup_window_secs: u64,

    /// Capture channel capacity
    #[arg(long, default_value_t = 64)]
    capture_channel_capacity: usize,

    /// Storage channel capacity
    #[arg(long, default_value_t = 32)]
    storage_channel_capacity: usize,
    
    /// TODO: [CONFIG] Config file path (e.g., ~/.config/recall.toml)
    /// Will allow per-monitor: fps, dedup_threshold, enabled, max_inactive_secs
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    let interval = Duration::from_secs_f64(1.0 / args.fps);
    info!(
        fps = args.fps,
        interval_ms = interval.as_millis() as u64,
        retention_days = args.retention_days,
        data_dir = %args.data_dir.display(),
        capture_channel_capacity = args.capture_channel_capacity,
        storage_channel_capacity = args.storage_channel_capacity,
        "Starting recall daemon with channel-based pipeline"
    );

    // Connect to Postgres
    let storage = create_storage()
        .await
        .context("Failed to connect to Postgres (check DATABASE_URL)")?;
    info!("Connected to Postgres");

    // Image storage on disk
    let image_storage = ImageStorage::new(&args.data_dir)
        .context("Failed to initialize image storage")?;
    info!(path = %args.data_dir.display(), "Image storage ready");

    // Deployment identity
    let deployment_id = hostname::get()
        .context("Failed to get hostname")?
        .to_string_lossy()
        .to_string();
    info!(deployment_id = %deployment_id, "Identified deployment");

    // Discover monitors
    // TODO: [CONFIG] Filter monitors based on config (enabled flag, name/id matching)
    let monitors = list_monitors()
        .await
        .context("Failed to list monitors")?;
    if monitors.is_empty() {
        anyhow::bail!("No monitors found");
    }
    for m in &monitors {
        let info = m.info();
        info!(
            id = m.id(),
            name = %info.name,
            resolution = format_args!("{}x{}", info.width, info.height),
            primary = info.is_primary,
            "Found monitor"
        );
    }

    // Wrap in Arc for sharing with background tasks
    let storage = Arc::new(storage);
    let image_storage = Arc::new(image_storage);

    // Spawn daily cleanup task
    let cleanup_storage = Arc::clone(&storage);
    let cleanup_images = Arc::clone(&image_storage);
    let retention = args.retention_days;
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
        tick.tick().await; // skip immediate first tick
        loop {
            tick.tick().await;
            info!("Running daily cleanup (retention={} days)", retention);
            match cleanup_storage.cleanup_old_data(retention).await {
                Ok(n) => info!(deleted = n, "Database cleanup complete"),
                Err(e) => error!("Database cleanup failed: {}", e),
            }
            match cleanup_images.cleanup_old_images(retention) {
                Ok(n) => info!(deleted = n, "Image cleanup complete"),
                Err(e) => error!("Image cleanup failed: {}", e),
            }
        }
    });

    // Create pipeline configuration
    let pipeline_config = PipelineConfig {
        capture_channel_capacity: args.capture_channel_capacity,
        storage_channel_capacity: args.storage_channel_capacity,
        ..Default::default()
    };

    // Create pipeline channels
    let channels = PipelineChannels::new(&pipeline_config);

    // Create metrics
    let metrics = Arc::new(PipelineMetrics::new());

    // Create shutdown channel
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<ShutdownSignal>(1);

    // Spawn capture tasks (one per monitor)
    // TODO: [CONFIG] Pass per-monitor config (fps, dedup_threshold, max_inactive_secs)
    let mut capture_handles = Vec::new();
    for monitor in monitors {
        let capture_tx = channels.capture_tx.clone();
        let metrics = Arc::clone(&metrics);
        let shutdown_rx = shutdown_tx.subscribe();
        let fps = args.fps;

        let handle = tokio::spawn(async move {
            run_capture_task(monitor, capture_tx, metrics, shutdown_rx, fps).await
        });
        capture_handles.push(handle);
    }

    // Spawn dedup task
    let dedup_shutdown_rx = shutdown_tx.subscribe();
    let dedup_handle = tokio::spawn(run_dedup_task(
        channels.capture_rx,
        channels.storage_tx.clone(),
        Arc::clone(&metrics),
        dedup_shutdown_rx,
    ));

    // Spawn storage task
    let storage_shutdown_rx = shutdown_tx.subscribe();
    let storage_handle = tokio::spawn(run_storage_task(
        channels.storage_rx,
        storage,
        image_storage,
        deployment_id,
        args.jpeg_quality,
        args.dedup_window_secs,
        Arc::clone(&metrics),
        storage_shutdown_rx,
    ));

    // Spawn metrics task
    let metrics_shutdown_rx = shutdown_tx.subscribe();
    let metrics_handle = tokio::spawn(run_metrics_task(
        channels.capture_tx.clone(),
        channels.storage_tx,
        Arc::clone(&metrics),
        pipeline_config,
        metrics_shutdown_rx,
    ));

    // Wait for Ctrl+C
    info!("Press Ctrl+C to shut down gracefully");
    match signal::ctrl_c().await {
        Ok(()) => info!("Received shutdown signal"),
        Err(e) => error!("Failed to listen for shutdown signal: {}", e),
    }

    // Send shutdown signal to all tasks
    info!("Sending shutdown signal to all tasks...");
    let _ = shutdown_tx.send(ShutdownSignal);

    // Wait for all tasks to complete (with timeout)
    let shutdown_timeout = Duration::from_secs(10);

    info!("Waiting for capture tasks to finish...");
    for handle in capture_handles {
        match tokio::time::timeout(shutdown_timeout, handle).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => warn!("Capture task error: {}", e),
            Err(_) => warn!("Capture task did not shut down in time"),
        }
    }

    info!("Waiting for dedup task to finish...");
    match tokio::time::timeout(shutdown_timeout, dedup_handle).await {
        Ok(Ok(())) => info!("Dedup task finished"),
        Ok(Err(e)) => warn!("Dedup task error: {}", e),
        Err(_) => warn!("Dedup task did not shut down in time"),
    }

    info!("Waiting for storage task to finish...");
    match tokio::time::timeout(shutdown_timeout, storage_handle).await {
        Ok(Ok(())) => info!("Storage task finished"),
        Ok(Err(e)) => warn!("Storage task error: {}", e),
        Err(_) => warn!("Storage task did not shut down in time"),
    }

    info!("Waiting for metrics task to finish...");
    match tokio::time::timeout(shutdown_timeout, metrics_handle).await {
        Ok(Ok(())) => info!("Metrics task finished"),
        Ok(Err(e)) => warn!("Metrics task error: {}", e),
        Err(_) => warn!("Metrics task did not shut down in time"),
    }

    // Final metrics summary
    metrics.log_summary();
    info!("Recall daemon stopped");

    Ok(())
}

/// Run a capture task for a single monitor.
async fn run_capture_task(
    monitor: recall_capture::monitor::SafeMonitor,
    capture_tx: mpsc::Sender<CaptureMessage>,
    metrics: Arc<PipelineMetrics>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<ShutdownSignal>,
    fps: f64,
) {
    use image::DynamicImage;
    use recall_capture::dedup::{frame_difference, phash64};
    use std::time::Instant;
    use tracing::debug;

    const DEDUP_THRESHOLD: f64 = 0.006;

    let monitor_id = monitor.id();
    let interval = Duration::from_secs_f64(1.0 / fps);
    let mut tick = tokio::time::interval(interval);
    let mut previous_image: Option<DynamicImage> = None;

    info!(monitor_id, fps, "Capture task started");

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let timestamp = Instant::now();

                // Capture frame
                let image = match monitor.capture_image().await {
                    Ok(img) => img,
                    Err(e) => {
                        warn!(monitor_id, "Capture error: {}", e);
                        continue;
                    }
                };

                // Dedup against previous frame
                if let Some(ref prev) = previous_image {
                    match frame_difference(prev, &image) {
                        Ok(diff) if diff < DEDUP_THRESHOLD => {
                            debug!(
                                monitor_id,
                                diff = format!("{:.4}", diff),
                                "Frame deduplicated (memory)"
                            );
                            metrics.frames_deduped_memory.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            continue;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            debug!(monitor_id, "Image comparison failed ({}), capturing anyway", e);
                        }
                    }
                }

                let phash = phash64(&image);
                metrics.frames_captured.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                // TODO: [BUG] Use actual capture time, not current wall-clock time
                // Currently timestamp (Instant) is unused; captured_at uses Utc::now() (line 356)
                let msg = CaptureMessage {
                    image: image.clone(),
                    phash,
                    timestamp,
                    monitor_id,
                };

                // Try to send to channel (non-blocking with backpressure)
                match capture_tx.try_send(msg) {
                    Ok(()) => {
                        debug!(monitor_id, "Frame sent to dedup channel");
                        previous_image = Some(image);
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        warn!(monitor_id, "Capture channel full, dropping frame");
                        previous_image = Some(image);
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        info!(monitor_id, "Capture channel closed, stopping");
                        break;
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

/// Run the dedup task.
async fn run_dedup_task(
    mut capture_rx: mpsc::Receiver<CaptureMessage>,
    storage_tx: mpsc::Sender<StorageMessage>,
    _metrics: Arc<PipelineMetrics>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<ShutdownSignal>,
) {
    use chrono::Utc;
    use tracing::debug;

    info!("Dedup task started");

    loop {
        tokio::select! {
            msg = capture_rx.recv() => {
                match msg {
                    Some(frame) => {
                        // TODO: [BUG] Convert frame.timestamp (Instant) to Utc (captured_at should reflect actual capture time)
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

/// Run the storage task.
async fn run_storage_task(
    mut storage_rx: mpsc::Receiver<StorageMessage>,
    storage: Arc<dyn Storage>,
    image_storage: Arc<ImageStorage>,
    deployment_id: String,
    jpeg_quality: u8,
    dedup_window_secs: u64,
    metrics: Arc<PipelineMetrics>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<ShutdownSignal>,
) {
    use tracing::debug;

    info!("Storage task started");

    loop {
        tokio::select! {
            msg = storage_rx.recv() => {
                match msg {
                    Some(frame) => {
                        let monitor_id = frame.monitor_id;
                        let phash = frame.phash;

                        // DB-level dedup: check recent frames with similar hash
                        match storage.is_duplicate(phash, dedup_window_secs).await {
                            Ok(Some(existing_id)) => {
                                debug!(
                                    monitor_id,
                                    existing_frame = %existing_id,
                                    "DB dedup: skipping duplicate"
                                );
                                metrics.frames_deduped_db.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                continue;
                            }
                            Ok(None) => {} // not a duplicate, proceed
                            Err(e) => {
                                warn!(monitor_id, "DB dedup check failed: {}", e);
                                // Proceed with insert on dedup failure
                            }
                        }

                        // Save JPEG to disk
                        let (image_ref, image_size_bytes) =
                            match image_storage.save_jpeg(&frame.image, frame.captured_at, jpeg_quality) {
                                Ok(result) => result,
                                Err(e) => {
                                    error!(monitor_id, "Failed to save JPEG: {}", e);
                                    metrics.frames_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    continue;
                                }
                            };

                        // Insert frame into database
                        match storage
                            .insert_frame(
                                frame.captured_at,
                                &deployment_id,
                                None::<&str>,  // window_title (not yet captured)
                                None::<&str>,  // app_name (not yet captured)
                                &image_ref,
                                image_size_bytes as i64,
                                phash,
                            )
                            .await
                        {
                            Ok(frame_id) => {
                                metrics.frames_stored.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                info!(
                                    monitor_id,
                                    frame_id = %frame_id,
                                    size_kb = image_size_bytes / 1024,
                                    "Frame stored"
                                );
                            }
                            Err(e) => {
                                metrics.frames_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                error!(monitor_id, "Failed to insert frame: {}", e);
                            }
                        }
                    }
                    None => {
                        info!("Storage channel closed, stopping storage task");
                        break;
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!("Storage task received shutdown signal");
                // Drain remaining frames before exiting
                info!("Draining remaining frames from storage channel...");
                while let Ok(frame) = storage_rx.try_recv() {
                    let monitor_id = frame.monitor_id;
                    let phash = frame.phash;

                    // Skip DB dedup during drain for speed
                    match image_storage.save_jpeg(&frame.image, frame.captured_at, jpeg_quality) {
                        Ok((image_ref, image_size_bytes)) => {
                            match storage
                                .insert_frame(
                                    frame.captured_at,
                                    &deployment_id,
                                    None::<&str>,
                                    None::<&str>,
                                    &image_ref,
                                    image_size_bytes as i64,
                                    phash,
                                )
                                .await
                            {
                                Ok(frame_id) => {
                                    metrics.frames_stored.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    info!(monitor_id, frame_id = %frame_id, "Frame stored (drain)");
                                }
                                Err(e) => {
                                    metrics.frames_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    error!(monitor_id, "Failed to insert frame (drain): {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            metrics.frames_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            error!(monitor_id, "Failed to save JPEG (drain): {}", e);
                        }
                    }
                }
                break;
            }
        }
    }

    info!("Storage task stopped");
}
