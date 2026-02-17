use anyhow::{Error, Result};
use image::DynamicImage;
use std::fmt;
use std::sync::Arc;
use tracing;
use xcap::Monitor as XcapMonitor;

/// Error type for monitor listing
#[derive(Debug)]
pub enum MonitorListError {
    PermissionDenied,
    NoMonitorsFound,
    Other(String),
}

impl fmt::Display for MonitorListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonitorListError::PermissionDenied => write!(
                f,
                "Screen recording permission not granted. Check privacy settings."
            ),
            MonitorListError::NoMonitorsFound => write!(f, "No monitors found"),
            MonitorListError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Clone)]
pub struct SafeMonitor {
    monitor_id: u32,
    monitor_data: Arc<MonitorData>,
}

#[derive(Clone, Debug)]
pub struct MonitorData {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub is_primary: bool,
}

impl SafeMonitor {
    pub fn new(monitor: XcapMonitor) -> Self {
        let monitor_id = monitor.id().unwrap_or(0);
        let monitor_data = Arc::new(MonitorData {
            width: monitor.width().unwrap_or(0),
            height: monitor.height().unwrap_or(0),
            x: monitor.x().unwrap_or(0),
            y: monitor.y().unwrap_or(0),
            name: monitor.name().unwrap_or_default().to_string(),
            is_primary: monitor.is_primary().unwrap_or(false),
        });

        Self {
            monitor_id,
            monitor_data,
        }
    }

    /// Capture a screenshot.
    /// On Windows/Linux, XcapMonitor is not Send, so we must re-enumerate inside spawn_blocking.
    pub async fn capture_image(&self) -> Result<DynamicImage> {
        let monitor_id = self.monitor_id;

        let image = tokio::task::spawn_blocking(move || -> Result<DynamicImage> {
            let monitors = XcapMonitor::all().map_err(|e| Error::msg(e.to_string()))?;
            let monitor = monitors
                .into_iter()
                .find(|m| m.id().unwrap_or(0) == monitor_id)
                .ok_or_else(|| anyhow::anyhow!("Monitor {} not found", monitor_id))?;

            if monitor.width().unwrap_or(0) == 0 || monitor.height().unwrap_or(0) == 0 {
                return Err(anyhow::anyhow!("Invalid monitor dimensions"));
            }

            let buffer = monitor.capture_image().map_err(|e| Error::msg(e.to_string()))?;
            
            // Xcap returns RgbaImage, convert to DynamicImage
            Ok(DynamicImage::ImageRgba8(buffer))
        })
        .await
        .map_err(|e| anyhow::anyhow!("capture task panicked: {}", e))??;

        Ok(image)
    }

    /// Refresh monitor metadata.
    pub async fn refresh(&mut self) -> Result<()> {
        let monitor_id = self.monitor_id;

        let refreshed = tokio::task::spawn_blocking(move || -> Result<MonitorData> {
            let monitors = XcapMonitor::all().map_err(|e| Error::msg(e.to_string()))?;
            let monitor = monitors
                .into_iter()
                .find(|m| m.id().unwrap_or(0) == monitor_id)
                .ok_or_else(|| anyhow::anyhow!("Monitor {} not found during refresh", monitor_id))?;

            Ok(MonitorData {
                width: monitor.width().unwrap_or(0),
                height: monitor.height().unwrap_or(0),
                x: monitor.x().unwrap_or(0),
                y: monitor.y().unwrap_or(0),
                name: monitor.name().unwrap_or_default().to_string(),
                is_primary: monitor.is_primary().unwrap_or(false),
            })
        })
        .await
        .map_err(|e| anyhow::anyhow!("refresh task panicked: {}", e))??;

        self.monitor_data = Arc::new(refreshed);
        tracing::debug!("Refreshed monitor {} metadata", self.monitor_id);
        Ok(())
    }

    pub fn id(&self) -> u32 {
        self.monitor_id
    }

    pub fn name(&self) -> &str {
        &self.monitor_data.name
    }
}

pub async fn list_monitors() -> Vec<SafeMonitor> {
    tokio::task::spawn_blocking(|| match XcapMonitor::all() {
        Ok(monitors) => monitors.into_iter().map(SafeMonitor::new).collect(),
        Err(e) => {
            tracing::error!("Failed to list monitors: {}", e);
            Vec::new()
        }
    })
    .await
    .unwrap_or_default()
}

pub async fn get_monitor_by_id(id: u32) -> Option<SafeMonitor> {
    tokio::task::spawn_blocking(move || match XcapMonitor::all() {
        Ok(monitors) => monitors
            .into_iter()
            .find(|m| m.id().unwrap_or(0) == id)
            .map(SafeMonitor::new),
        Err(e) => {
            tracing::error!("Failed to get monitor {}: {}", id, e);
            None
        }
    })
    .await
    .unwrap_or(None)
}
