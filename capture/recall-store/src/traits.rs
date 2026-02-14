use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A frame together with its denormalised context (OCR, vision, status flags).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameWithContext {
    pub id: Uuid,
    pub captured_at: DateTime<Utc>,
    pub deployment_id: String,
    pub window_title: Option<String>,
    pub app_name: Option<String>,
    pub image_ref: String,
    pub image_size_bytes: i64,
    pub phash: i64,
    pub has_text: bool,
    pub has_activity: bool,
    pub ocr_text: Option<String>,
    pub ocr_confidence: Option<f32>,
    pub vision_summary: Option<String>,
    pub vision_status: VisionStatus,
    pub embedding_status: EmbeddingStatus,
}

/// Processing status for LLM-based vision summarisation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VisionStatus {
    Pending,
    Processed,
    Failed,
    Skipped,
}

impl From<i16> for VisionStatus {
    fn from(v: i16) -> Self {
        match v {
            1 => Self::Processed,
            2 => Self::Failed,
            3 => Self::Skipped,
            _ => Self::Pending,
        }
    }
}

impl VisionStatus {
    /// Convert to the SMALLINT representation stored in Postgres.
    pub fn to_smallint(self) -> i16 {
        match self {
            Self::Pending => 0,
            Self::Processed => 1,
            Self::Failed => 2,
            Self::Skipped => 3,
        }
    }
}

/// Processing status for embedding generation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmbeddingStatus {
    Pending,
    Processed,
    Failed,
    Skipped,
}

impl From<i16> for EmbeddingStatus {
    fn from(v: i16) -> Self {
        match v {
            1 => Self::Processed,
            2 => Self::Failed,
            3 => Self::Skipped,
            _ => Self::Pending,
        }
    }
}

impl EmbeddingStatus {
    /// Convert to the SMALLINT representation stored in Postgres.
    pub fn to_smallint(self) -> i16 {
        match self {
            Self::Pending => 0,
            Self::Processed => 1,
            Self::Failed => 2,
            Self::Skipped => 3,
        }
    }
}

/// Per-application usage statistics over a time range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStats {
    pub app_name: String,
    pub frame_count: i64,
    pub total_seconds: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// High-level storage metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_frames: i64,
    pub frames_with_ocr: i64,
    pub total_image_bytes: i64,
}

// ---------------------------------------------------------------------------
// Storage trait
// ---------------------------------------------------------------------------

#[async_trait]
pub trait Storage: Send + Sync {
    /// Check whether a frame with a similar phash was already stored within
    /// the last `window_secs` seconds.  Returns the existing frame id if so.
    async fn is_duplicate(&self, phash: i64, window_secs: u64) -> Result<Option<Uuid>>;

    /// Persist a new frame and return its generated id.
    async fn insert_frame(
        &self,
        captured_at: DateTime<Utc>,
        deployment_id: &str,
        window_title: Option<&str>,
        app_name: Option<&str>,
        image_ref: &str,
        image_size_bytes: i64,
        phash: i64,
    ) -> Result<Uuid>;

    /// Return the most recent frames (paged).
    async fn get_recent_frames(&self, limit: u32, offset: u32) -> Result<Vec<FrameWithContext>>;

    /// Full-text search over OCR content.
    async fn search_text(&self, query: &str, limit: u32) -> Result<Vec<FrameWithContext>>;

    /// Return frames captured within a time range.
    async fn search_by_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<FrameWithContext>>;

    /// Return frames captured from a specific application.
    async fn search_by_app(&self, app_name: &str, limit: u32) -> Result<Vec<FrameWithContext>>;

    /// Compute per-app usage statistics for a time range.
    async fn get_app_stats(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AppStats>>;

    /// Delete frames older than `retention_days`.  Returns the number of rows
    /// removed.
    async fn cleanup_old_data(&self, retention_days: u32) -> Result<u64>;

    /// Return high-level storage metrics.
    async fn get_stats(&self) -> Result<StorageStats>;

    /// Toggle the `has_text` flag on a frame.
    async fn set_frame_has_text(&self, frame_id: Uuid, has_text: bool) -> Result<()>;

    /// Store an OCR result for a frame.  Also denormalises the text onto the
    /// `frames` row so that full-text search works directly.
    async fn insert_ocr_text(
        &self,
        frame_id: Uuid,
        text: &str,
        confidence: f32,
        language: Option<&str>,
        bbox: Option<&str>,
    ) -> Result<()>;

    /// Store window / application context for a frame.
    async fn insert_window_context(
        &self,
        frame_id: Uuid,
        app_name: &str,
        window_title: &str,
        process_name: Option<&str>,
        is_focused: bool,
        url: Option<&str>,
    ) -> Result<()>;

    /// Fetch frames that still need a vision summary.
    async fn get_frames_pending_vision(&self, limit: u32) -> Result<Vec<FrameWithContext>>;

    /// Write the LLM vision summary for a frame.
    async fn update_vision_summary(
        &self,
        frame_id: Uuid,
        summary: &str,
        status: VisionStatus,
    ) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_status_conversion() {
        assert_eq!(VisionStatus::from(0), VisionStatus::Pending);
        assert_eq!(VisionStatus::from(1), VisionStatus::Processed);
        assert_eq!(VisionStatus::from(2), VisionStatus::Failed);
        assert_eq!(VisionStatus::from(3), VisionStatus::Skipped);
        assert_eq!(VisionStatus::from(99), VisionStatus::Pending); // Default

        assert_eq!(VisionStatus::Pending.to_smallint(), 0);
        assert_eq!(VisionStatus::Processed.to_smallint(), 1);
        assert_eq!(VisionStatus::Failed.to_smallint(), 2);
        assert_eq!(VisionStatus::Skipped.to_smallint(), 3);
    }

    #[test]
    fn test_embedding_status_conversion() {
        assert_eq!(EmbeddingStatus::from(0), EmbeddingStatus::Pending);
        assert_eq!(EmbeddingStatus::from(1), EmbeddingStatus::Processed);
        assert_eq!(EmbeddingStatus::from(2), EmbeddingStatus::Failed);
        assert_eq!(EmbeddingStatus::from(3), EmbeddingStatus::Skipped);
        assert_eq!(EmbeddingStatus::from(-1), EmbeddingStatus::Pending); // Default

        assert_eq!(EmbeddingStatus::Pending.to_smallint(), 0);
        assert_eq!(EmbeddingStatus::Processed.to_smallint(), 1);
        assert_eq!(EmbeddingStatus::Failed.to_smallint(), 2);
        assert_eq!(EmbeddingStatus::Skipped.to_smallint(), 3);
    }
}


