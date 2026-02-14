use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use sqlx::Row;
use tracing::{debug, info};
use uuid::Uuid;

use recall_db::RecallDb;

use crate::traits::{
    AppStats, EmbeddingStatus, FrameWithContext, Storage, StorageStats, VisionStatus,
};

// ---------------------------------------------------------------------------
// Inlined dedup helpers (avoids circular dep on recall-capture)
// ---------------------------------------------------------------------------

/// Hamming distance between two 64-bit perceptual hashes.
fn hamming_distance(a: i64, b: i64) -> u32 {
    (a ^ b).count_ones()
}

/// Extract top-16-bit prefix for fast DB candidate filtering.
fn hash_prefix(phash: i64) -> i16 {
    ((phash >> 48) & 0xFFFF) as i16
}

/// Default Hamming-distance threshold for duplicate detection.
const DEDUP_THRESHOLD: u32 = 10;

// ---------------------------------------------------------------------------
// PgStorage
// ---------------------------------------------------------------------------

/// Postgres-backed [`Storage`] implementation.
pub struct PgStorage {
    db: RecallDb,
}

impl PgStorage {
    /// Connect to Postgres and run migrations.
    ///
    /// TODO: Add integration tests. See TESTING_TODOS_RUST.md section 3.2 for details.
    pub async fn new(database_url: &str) -> Result<Self> {
        let db = RecallDb::new(database_url)
            .await
            .context("Failed to connect to Postgres")?;
        info!("PgStorage ready");
        Ok(Self { db })
    }

    /// Borrow the inner `RecallDb` (useful for one-off queries).
    pub fn db(&self) -> &RecallDb {
        &self.db
    }
}

// ---------------------------------------------------------------------------
// Row → FrameWithContext helper
// ---------------------------------------------------------------------------

/// The column list shared by every frame query.
const FRAME_COLUMNS: &str = r#"
    id, captured_at, deployment_id, window_title, app_name,
    image_ref, image_size_bytes, phash,
    has_text, has_activity,
    ocr_text, vision_summary, vision_status, embedding_status
"#;

fn row_to_frame(row: &sqlx::postgres::PgRow) -> Result<FrameWithContext> {
    Ok(FrameWithContext {
        id: row.try_get("id")?,
        captured_at: row.try_get("captured_at")?,
        deployment_id: row
            .try_get::<Option<String>, _>("deployment_id")?
            .unwrap_or_default(),
        window_title: row.try_get("window_title")?,
        app_name: row.try_get("app_name")?,
        image_ref: row.try_get("image_ref")?,
        image_size_bytes: row
            .try_get::<Option<i64>, _>("image_size_bytes")?
            .unwrap_or(0),
        phash: row.try_get("phash")?,
        has_text: row
            .try_get::<Option<bool>, _>("has_text")?
            .unwrap_or(false),
        has_activity: row
            .try_get::<Option<bool>, _>("has_activity")?
            .unwrap_or(false),
        ocr_text: row.try_get("ocr_text")?,
        ocr_confidence: None, // stored in the ocr_text table, not denormalised
        vision_summary: row.try_get("vision_summary")?,
        vision_status: VisionStatus::from(
            row.try_get::<Option<i16>, _>("vision_status")?
                .unwrap_or(0),
        ),
        embedding_status: EmbeddingStatus::from(
            row.try_get::<Option<i16>, _>("embedding_status")?
                .unwrap_or(0),
        ),
    })
}

// ---------------------------------------------------------------------------
// Storage impl
// ---------------------------------------------------------------------------

#[async_trait]
impl Storage for PgStorage {
    async fn is_duplicate(&self, phash: i64, window_secs: u64) -> Result<Option<Uuid>> {
        let prefix = hash_prefix(phash);
        let since = Utc::now() - Duration::seconds(window_secs as i64);

        let candidates = self
            .db
            .recent_phash_candidates(prefix, since)
            .await
            .context("Failed to fetch phash candidates")?;

        info!(?prefix, ?since, count = candidates.len(), "Checking duplicates");

        for (id, candidate_hash) in &candidates {
            let dist = hamming_distance(phash, *candidate_hash);
            info!(?id, ?dist, "Checking candidate");
            if dist <= DEDUP_THRESHOLD {
                info!(
                    existing_id = %id,
                    distance = dist,
                    "Duplicate frame detected"
                );
                return Ok(Some(*id));
            }
        }

        Ok(None)
    }

    async fn insert_frame(
        &self,
        captured_at: DateTime<Utc>,
        deployment_id: &str,
        window_title: Option<&str>,
        app_name: Option<&str>,
        image_ref: &str,
        image_size_bytes: i64,
        phash: i64,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let prefix = hash_prefix(phash);

        // Use RecallDb's insert_frame for the core columns …
        self.db
            .insert_frame(
                id,
                captured_at,
                Some(deployment_id),
                image_ref,
                Some(image_size_bytes),
                phash,
                prefix,
            )
            .await
            .context("Failed to insert frame")?;

        // … then patch window_title / app_name which RecallDb doesn't set.
        if window_title.is_some() || app_name.is_some() {
            sqlx::query(
                r#"
                UPDATE frames
                SET window_title = COALESCE($2, window_title),
                    app_name     = COALESCE($3, app_name)
                WHERE id = $1
                "#,
            )
            .bind(id)
            .bind(window_title)
            .bind(app_name)
            .execute(self.db.pool())
            .await
            .context("Failed to update window context on frame")?;
        }

        debug!(frame_id = %id, "Frame inserted");
        Ok(id)
    }

    async fn get_recent_frames(&self, limit: u32, offset: u32) -> Result<Vec<FrameWithContext>> {
        let sql = format!(
            "SELECT {} FROM frames ORDER BY captured_at DESC LIMIT $1 OFFSET $2",
            FRAME_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(self.db.pool())
            .await
            .context("get_recent_frames query failed")?;

        rows.iter().map(row_to_frame).collect()
    }

    async fn search_text(&self, query: &str, limit: u32) -> Result<Vec<FrameWithContext>> {
        let sql = format!(
            r#"
            SELECT {}
            FROM frames
            WHERE to_tsvector('english', COALESCE(ocr_text, ''))
                  @@ plainto_tsquery('english', $1)
            ORDER BY captured_at DESC
            LIMIT $2
            "#,
            FRAME_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(query)
            .bind(limit as i64)
            .fetch_all(self.db.pool())
            .await
            .context("search_text query failed")?;

        rows.iter().map(row_to_frame).collect()
    }

    async fn search_by_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<FrameWithContext>> {
        let sql = format!(
            r#"
            SELECT {}
            FROM frames
            WHERE captured_at >= $1 AND captured_at <= $2
            ORDER BY captured_at DESC
            "#,
            FRAME_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(start)
            .bind(end)
            .fetch_all(self.db.pool())
            .await
            .context("search_by_time query failed")?;

        rows.iter().map(row_to_frame).collect()
    }

    async fn search_by_app(&self, app_name: &str, limit: u32) -> Result<Vec<FrameWithContext>> {
        let sql = format!(
            r#"
            SELECT {}
            FROM frames
            WHERE app_name = $1
            ORDER BY captured_at DESC
            LIMIT $2
            "#,
            FRAME_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(app_name)
            .bind(limit as i64)
            .fetch_all(self.db.pool())
            .await
            .context("search_by_app query failed")?;

        rows.iter().map(row_to_frame).collect()
    }

    async fn get_app_stats(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AppStats>> {
        let rows = sqlx::query(
            r#"
            SELECT
                app_name,
                COUNT(*)                                         AS frame_count,
                EXTRACT(EPOCH FROM MAX(captured_at) - MIN(captured_at))::BIGINT AS total_seconds,
                MIN(captured_at)                                 AS first_seen,
                MAX(captured_at)                                 AS last_seen
            FROM frames
            WHERE app_name IS NOT NULL
              AND captured_at >= $1 AND captured_at <= $2
            GROUP BY app_name
            ORDER BY frame_count DESC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(self.db.pool())
        .await
        .context("get_app_stats query failed")?;

        rows.iter()
            .map(|r| {
                Ok(AppStats {
                    app_name: r.try_get("app_name")?,
                    frame_count: r.try_get("frame_count")?,
                    total_seconds: r
                        .try_get::<Option<i64>, _>("total_seconds")?
                        .unwrap_or(0),
                    first_seen: r.try_get("first_seen")?,
                    last_seen: r.try_get("last_seen")?,
                })
            })
            .collect()
    }

    async fn cleanup_old_data(&self, retention_days: u32) -> Result<u64> {
        let cutoff = Utc::now() - Duration::days(retention_days as i64);
        let result = sqlx::query("DELETE FROM frames WHERE captured_at < $1")
            .bind(cutoff)
            .execute(self.db.pool())
            .await
            .context("cleanup_old_data failed")?;

        let deleted = result.rows_affected();
        info!(deleted, retention_days, "Old frames cleaned up");
        Ok(deleted)
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*)                                    AS total_frames,
                COUNT(*) FILTER (WHERE has_text = TRUE)     AS frames_with_ocr,
                COALESCE(SUM(image_size_bytes), 0)          AS total_image_bytes
            FROM frames
            "#,
        )
        .fetch_one(self.db.pool())
        .await
        .context("get_stats query failed")?;

        Ok(StorageStats {
            total_frames: row.try_get("total_frames")?,
            frames_with_ocr: row.try_get("frames_with_ocr")?,
            total_image_bytes: row.try_get("total_image_bytes")?,
        })
    }

    async fn set_frame_has_text(&self, frame_id: Uuid, has_text: bool) -> Result<()> {
        sqlx::query("UPDATE frames SET has_text = $2 WHERE id = $1")
            .bind(frame_id)
            .bind(has_text)
            .execute(self.db.pool())
            .await
            .context("set_frame_has_text failed")?;
        Ok(())
    }

    async fn insert_ocr_text(
        &self,
        frame_id: Uuid,
        text: &str,
        confidence: f32,
        language: Option<&str>,
        bbox: Option<&str>,
    ) -> Result<()> {
        // 1. Insert detailed OCR row.
        self.db
            .insert_ocr_text(frame_id, text, Some(confidence), language, bbox)
            .await
            .context("Failed to insert OCR text row")?;

        // 2. Denormalise onto the frames table for fast full-text search.
        sqlx::query(
            r#"
            UPDATE frames
            SET ocr_text = $2, has_text = TRUE
            WHERE id = $1
            "#,
        )
        .bind(frame_id)
        .bind(text)
        .execute(self.db.pool())
        .await
        .context("Failed to denormalise OCR text onto frame")?;

        debug!(frame_id = %frame_id, "OCR text stored");
        Ok(())
    }

    async fn insert_window_context(
        &self,
        frame_id: Uuid,
        app_name: &str,
        window_title: &str,
        process_name: Option<&str>,
        is_focused: bool,
        url: Option<&str>,
    ) -> Result<()> {
        self.db
            .insert_window_context(
                frame_id,
                Some(app_name),
                Some(window_title),
                process_name,
                Some(is_focused),
                url,
            )
            .await
            .context("Failed to insert window context")?;

        // Also denormalise onto the frames row.
        sqlx::query(
            r#"
            UPDATE frames
            SET app_name     = COALESCE($2, app_name),
                window_title = COALESCE($3, window_title)
            WHERE id = $1
            "#,
        )
        .bind(frame_id)
        .bind(app_name)
        .bind(window_title)
        .execute(self.db.pool())
        .await
        .context("Failed to denormalise window context onto frame")?;

        Ok(())
    }

    async fn get_frames_pending_vision(&self, limit: u32) -> Result<Vec<FrameWithContext>> {
        let sql = format!(
            r#"
            SELECT {}
            FROM frames
            WHERE vision_status = 0
              AND has_text = TRUE
            ORDER BY captured_at DESC
            LIMIT $1
            "#,
            FRAME_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(limit as i64)
            .fetch_all(self.db.pool())
            .await
            .context("get_frames_pending_vision query failed")?;

        rows.iter().map(row_to_frame).collect()
    }

    async fn update_vision_summary(
        &self,
        frame_id: Uuid,
        summary: &str,
        status: VisionStatus,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE frames
            SET vision_summary = $2,
                vision_status  = $3
            WHERE id = $1
            "#,
        )
        .bind(frame_id)
        .bind(summary)
        .bind(status.to_smallint())
        .execute(self.db.pool())
        .await
        .context("update_vision_summary failed")?;

        debug!(frame_id = %frame_id, ?status, "Vision summary updated");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hamming_distance() {
        assert_eq!(hamming_distance(0, 0), 0);
        assert_eq!(hamming_distance(0, -1), 64); // All bits different
        assert_eq!(hamming_distance(1, 2), 2);   // 01 vs 10
        assert_eq!(hamming_distance(0xF0, 0x0F), 8);
    }

    #[test]
    fn test_hash_prefix() {
        // 0x1234_5678_9ABC_DEF0
        let hash: i64 = 0x1234_5678_9ABC_DEF0; // Note: i64 might interpret this as negative if high bit set
        
        // Manual calc:
        // 0x1234_5678_9ABC_DEF0 >> 48 = 0x1234
        // 0x1234 & 0xFFFF = 0x1234
        let expected = 0x1234;
        
        assert_eq!(hash_prefix(hash), expected);
    }
}
