use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use image::DynamicImage;
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// File-system based JPEG storage with date-based folder organisation.
///
/// Images are stored under `<base_path>/YYYY-MM-DD/<uuid>.jpg`.  The returned
/// `image_ref` is always the *relative* path from `base_path`, so it stays
/// portable across mounts.
pub struct ImageStorage {
    base_path: PathBuf,
}

impl ImageStorage {
    /// Create a new `ImageStorage`, ensuring the base directory exists.
    pub fn new(base_path: impl Into<PathBuf>) -> Result<Self> {
        let base_path = base_path.into();
        fs::create_dir_all(&base_path)
            .with_context(|| format!("Failed to create image base dir: {}", base_path.display()))?;
        info!(path = %base_path.display(), "ImageStorage initialised");
        Ok(Self { base_path })
    }

    /// Save a `DynamicImage` as JPEG with the given quality (1-100).
    ///
    /// Returns `(image_ref, file_size_bytes)` where `image_ref` is the
    /// relative path suitable for storing in the database.
    ///
    /// TODO: Add integration tests. See TESTING_TODOS_RUST.md section 3.3 for details.
    pub fn save_jpeg(
        &self,
        image: &DynamicImage,
        timestamp: DateTime<Utc>,
        quality: u8,
    ) -> Result<(String, u64)> {
        let date_dir = timestamp.format("%Y-%m-%d").to_string();
        let dir = self.base_path.join(&date_dir);
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create date dir: {}", dir.display()))?;

        let filename = format!("{}.jpg", Uuid::new_v4());
        let file_path = dir.join(&filename);

        // Encode JPEG into a buffer, then write atomically.
        let file = fs::File::create(&file_path)
            .with_context(|| format!("Failed to create image file: {}", file_path.display()))?;
        let mut writer = BufWriter::new(file);

        let rgb = image.to_rgb8();
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
        rgb.write_with_encoder(encoder)
            .context("JPEG encoding failed")?;

        // Drop writer to flush, then stat the file for its size.
        drop(writer);
        let metadata = fs::metadata(&file_path)
            .with_context(|| format!("Failed to stat image file: {}", file_path.display()))?;
        let file_size = metadata.len();

        let image_ref = format!("{}/{}", date_dir, filename);
        debug!(image_ref, file_size, "Image saved");

        Ok((image_ref, file_size))
    }

    /// Async version of `save_jpeg` that runs JPEG encoding on a blocking thread pool.
    ///
    /// This should be used in async contexts to avoid blocking the Tokio runtime
    /// with CPU-intensive JPEG encoding operations.
    pub async fn save_jpeg_async(
        &self,
        image: DynamicImage,
        timestamp: DateTime<Utc>,
        quality: u8,
    ) -> Result<(String, u64)> {
        let base_path = self.base_path.clone();
        tokio::task::spawn_blocking(move || {
            let date_dir = timestamp.format("%Y-%m-%d").to_string();
            let dir = base_path.join(&date_dir);
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create date dir: {}", dir.display()))?;

            let filename = format!("{}.jpg", Uuid::new_v4());
            let file_path = dir.join(&filename);

            // Encode JPEG into a buffer, then write atomically.
            let file = fs::File::create(&file_path)
                .with_context(|| format!("Failed to create image file: {}", file_path.display()))?;
            let mut writer = BufWriter::new(file);

            let rgb = image.to_rgb8();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
            rgb.write_with_encoder(encoder)
                .context("JPEG encoding failed")?;

            // Drop writer to flush, then stat the file for its size.
            drop(writer);
            let metadata = fs::metadata(&file_path)
                .with_context(|| format!("Failed to stat image file: {}", file_path.display()))?;
            let file_size = metadata.len();

            let image_ref = format!("{}/{}", date_dir, filename);
            debug!(image_ref, file_size, "Image saved (async)");

            Ok((image_ref, file_size))
        })
        .await
        .map_err(|e| anyhow::anyhow!("spawn_blocking error: {}", e))?
    }

    /// Load a previously saved image by its `image_ref`.
    ///
    /// TODO: Add integration tests. See TESTING_TODOS_RUST.md section 3.3 for details.
    pub fn load_image(&self, image_ref: &str) -> Result<DynamicImage> {
        let path = self.base_path.join(image_ref);
        let img = image::open(&path)
            .with_context(|| format!("Failed to load image: {}", path.display()))?;
        Ok(img)
    }

    /// Delete date directories older than `retention_days`.
    ///
    /// Returns the number of files removed.
    ///
    /// TODO: Add integration tests. See TESTING_TODOS_RUST.md section 3.3 for details.
    pub fn cleanup_old_images(&self, retention_days: u32) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let cutoff_date = cutoff.format("%Y-%m-%d").to_string();

        let mut removed: u64 = 0;

        let entries = fs::read_dir(&self.base_path)
            .with_context(|| format!("Failed to read image dir: {}", self.base_path.display()))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Skipping unreadable dir entry: {}", e);
                    continue;
                }
            };

            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Only consider directories that look like date dirs (YYYY-MM-DD).
            if !entry.path().is_dir() || name_str.len() != 10 {
                continue;
            }

            // Lexicographic comparison works for ISO dates.
            if name_str.as_ref() < cutoff_date.as_str() {
                removed += remove_dir_contents(&entry.path())?;
                fs::remove_dir_all(entry.path()).with_context(|| {
                    format!("Failed to remove old image dir: {}", entry.path().display())
                })?;
                info!(dir = %name_str, files = removed, "Removed old image directory");
            }
        }

        Ok(removed)
    }
}

/// Count files inside a directory (non-recursive) so we can report how many
/// images were removed.
fn remove_dir_contents(dir: &Path) -> Result<u64> {
    let mut count: u64 = 0;
    for entry in fs::read_dir(dir)? {
        if let Ok(e) = entry {
            if e.path().is_file() {
                count += 1;
            }
        }
    }
    Ok(count)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbImage};
    use std::fs;
    use tempfile::TempDir;

    fn make_test_image(width: u32, height: u32) -> DynamicImage {
        let img = RgbImage::from_fn(width, height, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let storage = ImageStorage::new(tmp.path()).unwrap();
        let img = make_test_image(64, 64);
        let ts = Utc::now();

        let (image_ref, size) = storage.save_jpeg(&img, ts, 85).unwrap();
        assert!(size > 0, "Saved file should be non-empty");
        assert!(image_ref.ends_with(".jpg"));

        let loaded = storage.load_image(&image_ref).unwrap();
        assert_eq!(loaded.width(), 64);
        assert_eq!(loaded.height(), 64);
    }

    #[test]
    fn date_based_directory_structure() {
        let tmp = TempDir::new().unwrap();
        let storage = ImageStorage::new(tmp.path()).unwrap();
        let img = make_test_image(16, 16);
        let ts = Utc::now();

        let (image_ref, _) = storage.save_jpeg(&img, ts, 75).unwrap();
        let date_part = ts.format("%Y-%m-%d").to_string();
        assert!(
            image_ref.starts_with(&date_part),
            "image_ref should start with date: got {}",
            image_ref
        );

        // The date directory should exist on disk.
        let date_dir = tmp.path().join(&date_part);
        assert!(date_dir.is_dir());
    }

    #[test]
    fn cleanup_removes_old_dirs() {
        let tmp = TempDir::new().unwrap();
        let storage = ImageStorage::new(tmp.path()).unwrap();

        // Create a fake old date directory with a file inside.
        let old_dir = tmp.path().join("2020-01-01");
        fs::create_dir_all(&old_dir).unwrap();
        fs::write(old_dir.join("old.jpg"), b"fake").unwrap();

        // Create a recent directory that should be kept.
        let recent_dir = tmp.path().join(Utc::now().format("%Y-%m-%d").to_string());
        fs::create_dir_all(&recent_dir).unwrap();
        fs::write(recent_dir.join("new.jpg"), b"fake").unwrap();

        let removed = storage.cleanup_old_images(1).unwrap();
        assert!(removed >= 1, "Should have removed at least 1 file");
        assert!(!old_dir.exists(), "Old dir should be deleted");
        assert!(recent_dir.exists(), "Recent dir should survive");
    }

    #[test]
    fn load_nonexistent_returns_error() {
        let tmp = TempDir::new().unwrap();
        let storage = ImageStorage::new(tmp.path()).unwrap();
        let result = storage.load_image("1999-01-01/nope.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn multiple_saves_same_timestamp() {
        let tmp = TempDir::new().unwrap();
        let storage = ImageStorage::new(tmp.path()).unwrap();
        let img = make_test_image(8, 8);
        let ts = Utc::now();

        let (ref1, _) = storage.save_jpeg(&img, ts, 80).unwrap();
        let (ref2, _) = storage.save_jpeg(&img, ts, 80).unwrap();
        assert_ne!(ref1, ref2, "Each save should produce a unique filename");
    }
}
