use anyhow::Result;
use chrono::Utc;
use recall_db::RecallDb;
use std::env;
use uuid::Uuid;

async fn get_test_db() -> Result<Option<RecallDb>> {
    let url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test: DATABASE_URL not set");
            return Ok(None);
        }
    };
    Ok(Some(RecallDb::new(&url).await?))
}

#[tokio::test]
async fn test_insert_and_query_frame() -> Result<()> {
    let db = match get_test_db().await? {
        Some(db) => db,
        None => return Ok(()),
    };

    let id = Uuid::new_v4();
    let captured_at = Utc::now();
    let phash = 1234567890;
    let prefix = (phash >> 48) as i16;

    // 1. Insert frame
    db.insert_frame(
        id,
        captured_at,
        Some("test-deploy"),
        "test/img.jpg",
        Some(1024),
        phash,
        prefix,
    )
    .await?;

    // 2. Query back (using phash candidates logic as a proxy for existence)
    let candidates = db.recent_phash_candidates(prefix, captured_at).await?;
    assert!(
        candidates.iter().any(|(fid, _)| *fid == id),
        "Frame should be discoverable via phash candidates"
    );

    Ok(())
}

#[tokio::test]
async fn test_insert_ocr_text() -> Result<()> {
    let db = match get_test_db().await? {
        Some(db) => db,
        None => return Ok(()),
    };

    let id = Uuid::new_v4();
    let captured_at = Utc::now();

    // Insert frame first (FK constraint)
    db.insert_frame(
        id,
        captured_at,
        Some("test-deploy"),
        "test/ocr.jpg",
        Some(100),
        0,
        0,
    )
    .await?;

    // Insert OCR
    let ocr_id = db
        .insert_ocr_text(id, "sample text", Some(0.99), Some("en"), None)
        .await?;

    assert!(ocr_id > 0, "OCR insert should return a valid ID");
    Ok(())
}

#[tokio::test]
async fn test_insert_window_context() -> Result<()> {
    let db = match get_test_db().await? {
        Some(db) => db,
        None => return Ok(()),
    };

    let id = Uuid::new_v4();
    let captured_at = Utc::now();

    db.insert_frame(
        id,
        captured_at,
        Some("test-deploy"),
        "test/win.jpg",
        Some(100),
        0,
        0,
    )
    .await?;

    let win_id = db
        .insert_window_context(
            id,
            Some("Firefox"),
            Some("Mozilla Developer Network"),
            None,
            Some(true),
            None,
        )
        .await?;

    assert!(win_id > 0, "Window context insert should return a valid ID");
    Ok(())
}
