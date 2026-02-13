use anyhow::{Error, Result};
use image::DynamicImage;
use std::sync::Arc;
use xcap::Monitor;

/// Thread-safe monitor wrapper that caches metadata and captures on a blocking thread.
#[derive(Clone)]
pub struct SafeMonitor {
    monitor_id: u32,
    data: Arc<MonitorData>,
}

#[derive(Clone, Debug)]
pub struct MonitorData {
    pub width: u32,
    pub height: u32,
    pub name: String,
    pub is_primary: bool,
}

impl SafeMonitor {
    pub fn new(monitor: Monitor) -> Self {
        let monitor_id = monitor.id().unwrap();
        let data = Arc::new(MonitorData {
            width: monitor.width().unwrap(),
            height: monitor.height().unwrap(),
            name: monitor.name().unwrap().to_string(),
            is_primary: monitor.is_primary().unwrap(),
        });
        Self { monitor_id, data }
    }

    /// Capture a screenshot on a blocking thread (xcap is synchronous).
    pub async fn capture_image(&self) -> Result<DynamicImage> {
        let id = self.monitor_id;
        tokio::task::spawn_blocking(move || -> Result<DynamicImage> {
            let monitor = Monitor::all()
                .map_err(Error::from)?
                .into_iter()
                .find(|m| m.id().unwrap() == id)
                .ok_or_else(|| anyhow::anyhow!("Monitor {} not found", id))?;

            monitor
                .capture_image()
                .map_err(Error::from)
                .map(DynamicImage::ImageRgba8)
        })
        .await?
    }

    pub fn id(&self) -> u32 {
        self.monitor_id
    }

    pub fn info(&self) -> &MonitorData {
        &self.data
    }
}

/// List all available monitors.
pub async fn list_monitors() -> Result<Vec<SafeMonitor>> {
    tokio::task::spawn_blocking(|| {
        Monitor::all()
            .map(|monitors| monitors.into_iter().map(SafeMonitor::new).collect())
            .map_err(Error::from)
    })
    .await?
}
