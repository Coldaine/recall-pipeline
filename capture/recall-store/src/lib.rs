pub mod images;
pub mod postgres;
pub mod traits;

pub use images::ImageStorage;
pub use postgres::PgStorage;
pub use traits::{
    AppStats, EmbeddingStatus, FrameWithContext, Storage, StorageStats, VisionStatus,
};

/// Factory: create storage engine from DATABASE_URL env var.
pub async fn create_storage() -> anyhow::Result<PgStorage> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/recall".to_string());
    PgStorage::new(&database_url).await
}
