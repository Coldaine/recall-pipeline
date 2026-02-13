use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

/// Postgres database manager for recall-pipeline.
pub struct RecallDb {
    pool: Pool<Postgres>,
}

impl RecallDb {
    pub async fn new(connection_string: &str) -> Result<Self, sqlx::Error> {
        info!("Connecting to Postgres: {}", connection_string);

        let pool = PgPoolOptions::new()
            .max_connections(50)
            .min_connections(3)
            .acquire_timeout(Duration::from_secs(10))
            .connect(connection_string)
            .await?;

        let db = RecallDb { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./src/migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Insert a new frame.
    pub async fn insert_frame(
        &self,
        id: Uuid,
        captured_at: DateTime<Utc>,
        deployment_id: Option<&str>,
        image_ref: &str,
        image_size_bytes: Option<i64>,
        phash: i64,
        phash_prefix: i16,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO frames (id, captured_at, deployment_id, image_ref, image_size_bytes, phash, phash_prefix, has_text, has_activity)
            VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, FALSE)
            ON CONFLICT (id, captured_at) DO NOTHING
            "#,
        )
        .bind(id)
        .bind(captured_at)
        .bind(deployment_id)
        .bind(image_ref)
        .bind(image_size_bytes)
        .bind(phash)
        .bind(phash_prefix)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Find recent frames with the same phash prefix for dedup candidate filtering.
    pub async fn recent_phash_candidates(
        &self,
        phash_prefix: i16,
        since: DateTime<Utc>,
    ) -> Result<Vec<(Uuid, i64)>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, phash
            FROM frames
            WHERE phash_prefix = $1 AND captured_at >= $2
            LIMIT 5000
            "#,
        )
        .bind(phash_prefix)
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let id: Uuid = r.try_get("id").ok()?;
                let phash: i64 = r.try_get("phash").ok()?;
                Some((id, phash))
            })
            .collect())
    }

    /// Mark a frame as having OCR text.
    pub async fn set_frame_has_text(&self, frame_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r#"UPDATE frames SET has_text = TRUE WHERE id = $1"#)
            .bind(frame_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Insert OCR text for a frame.
    pub async fn insert_ocr_text(
        &self,
        frame_id: Uuid,
        text: &str,
        confidence: Option<f32>,
        language: Option<&str>,
        bbox_json: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let rec = sqlx::query(
            r#"
            INSERT INTO ocr_text (frame_id, text, confidence, language, bbox)
            VALUES ($1, $2, $3, $4, COALESCE($5::jsonb, NULL))
            RETURNING id
            "#,
        )
        .bind(frame_id)
        .bind(text)
        .bind(confidence)
        .bind(language)
        .bind(bbox_json)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec.get::<i64, _>("id"))
    }

    /// Insert window context for a frame.
    pub async fn insert_window_context(
        &self,
        frame_id: Uuid,
        app_name: Option<&str>,
        window_title: Option<&str>,
        process_name: Option<&str>,
        is_focused: Option<bool>,
        url: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let rec = sqlx::query(
            r#"
            INSERT INTO window_context (frame_id, app_name, window_title, process_name, is_focused, url)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(frame_id)
        .bind(app_name)
        .bind(window_title)
        .bind(process_name)
        .bind(is_focused)
        .bind(url)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec.get::<i64, _>("id"))
    }
}
