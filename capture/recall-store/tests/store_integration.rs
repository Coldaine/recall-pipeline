use chrono::{Duration, Utc};
use recall_store::postgres::PgStorage;
use recall_store::traits::{Storage, VisionStatus};
use std::env;
use uuid::Uuid;

async fn get_test_storage() -> PgStorage {
    // Load .env file or rely on env vars
    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgStorage::new(&db_url).await.expect("Failed to create storage")
}

#[tokio::test]
async fn test_store_integration_full_lifecycle() {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = get_test_storage().await;
    
    // 1. Insert a frame
    let frame_id = Uuid::new_v4();
    let phash: i64 = 0x1234567890ABCDEF; // Unique 64-bit hash
    let now = Utc::now();
    
    let stored_id = storage
        .insert_frame(
            now,
            "test-deployment",
            Some("Test Window"),
            Some("Test App"),
            "path/to/image.jpg",
            1024,
            phash,
        )
        .await
        .expect("Failed to insert frame");
        
    // Note: insert_frame generates its own ID if not passed, but here we see the trait signature returns Uuid.
    // Actually, looking at the trait, insert_frame DOES return Uuid.
    
    // 2. Fetch recent frames
    let recent = storage.get_recent_frames(10, 0).await.expect("Failed to get recent frames");
    assert!(recent.iter().any(|f| f.id == stored_id));
    
    let frame = recent.iter().find(|f| f.id == stored_id).unwrap();
    assert_eq!(frame.app_name.as_deref(), Some("Test App"));
    assert_eq!(frame.phash, phash);
    
    // 3. Test deduplication detection
    // Same hash, immediate check -> should be duplicate
    println!("DEBUG: Testing dedup with phash={} stored_id={}", phash, stored_id);
    let dup_id = storage.is_duplicate(phash, 60).await.expect("Dedup check failed");
    assert_eq!(dup_id, Some(stored_id));
    
    // Different hash -> should not be duplicate
    let no_dup = storage.is_duplicate(phash + 1000, 60).await.expect("Dedup check failed");
    assert_eq!(no_dup, None);
    
    // 4. Update Vision Status
    storage
        .update_vision_summary(stored_id, "A test summary", VisionStatus::Processed)
        .await
        .expect("Failed to update vision");
        
    // Verify update
    let updated_recent = storage.get_recent_frames(10, 0).await.expect("Failed to get recent");
    let updated_frame = updated_recent.iter().find(|f| f.id == stored_id).unwrap();
    assert_eq!(updated_frame.vision_status, VisionStatus::Processed);
    assert_eq!(updated_frame.vision_summary.as_deref(), Some("A test summary"));

    // 5. Cleanup (optional, but good for hygiene if we had a dedicated test DB)
    // For now we just assume the DB is persistent dev DB.
}
