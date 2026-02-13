use image::DynamicImage;
use image_compare::{Algorithm, Metric, Similarity};

/// Compute a 64-bit perceptual hash by resizing to 8x8 grayscale and comparing
/// pixel values against the mean.
pub fn phash64(image: &DynamicImage) -> u64 {
    let gray = image.to_luma8();
    let resized = image::imageops::resize(&gray, 8, 8, image::imageops::FilterType::Triangle);

    let sum: u64 = resized.pixels().map(|p| p.0[0] as u64).sum();
    let avg = (sum / 64) as u8;

    let mut bits: u64 = 0;
    for (i, p) in resized.pixels().enumerate() {
        if p.0[0] >= avg {
            bits |= 1u64 << i;
        }
    }
    bits
}

/// Async version of `phash64` that runs on a blocking thread pool.
///
/// This should be used in async contexts to avoid blocking the Tokio runtime.
pub async fn phash64_async(image: DynamicImage) -> u64 {
    tokio::task::spawn_blocking(move || phash64(&image))
        .await
        .unwrap_or(0)
}

/// Hamming distance between two 64-bit perceptual hashes.
pub fn hamming_distance(a: i64, b: i64) -> u32 {
    (a ^ b).count_ones()
}

/// Extract top-16-bit prefix for fast DB candidate filtering.
pub fn hash_prefix(phash: i64) -> i16 {
    ((phash >> 48) & 0xFFFF) as i16
}

/// Check if two hashes are perceptually similar within a Hamming threshold.
pub fn is_similar(a: i64, b: i64, threshold: u32) -> bool {
    hamming_distance(a, b) <= threshold
}

/// Compare two images using histogram similarity (Hellinger metric).
pub fn compare_histogram(a: &DynamicImage, b: &DynamicImage) -> anyhow::Result<f64> {
    let la = a.to_luma8();
    let lb = b.to_luma8();
    image_compare::gray_similarity_histogram(Metric::Hellinger, &la, &lb)
        .map_err(|e| anyhow::anyhow!("Histogram compare failed: {}", e))
}

/// Compare two images using SSIM (structural similarity).
pub fn compare_ssim(a: &DynamicImage, b: &DynamicImage) -> f64 {
    let la = a.to_luma8();
    let lb = b.to_luma8();
    let result: Similarity =
        image_compare::gray_similarity_structure(&Algorithm::MSSIMSimple, &la, &lb)
            .expect("Images had different dimensions");
    result.score
}

/// Combined similarity score: average of histogram diff and SSIM diff.
/// Returns a value where 0.0 = identical, higher = more different.
pub fn frame_difference(a: &DynamicImage, b: &DynamicImage) -> anyhow::Result<f64> {
    let histogram_diff = compare_histogram(a, b)?;
    let ssim_diff = 1.0 - compare_ssim(a, b);
    Ok((histogram_diff + ssim_diff) / 2.0)
}

/// Async version of `frame_difference` that runs on a blocking thread pool.
///
/// This should be used in async contexts to avoid blocking the Tokio runtime
/// with CPU-intensive image comparison operations.
pub async fn frame_difference_async(a: DynamicImage, b: DynamicImage) -> anyhow::Result<f64> {
    tokio::task::spawn_blocking(move || frame_difference(&a, &b))
        .await
        .map_err(|e| anyhow::anyhow!("spawn_blocking error: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_hashes() {
        let hash = 0x123456789ABCDEF0i64;
        assert_eq!(hamming_distance(hash, hash), 0);
    }

    #[test]
    fn one_bit_difference() {
        let a = 0x0000000000000000i64;
        let b = 0x0000000000000001i64;
        assert_eq!(hamming_distance(a, b), 1);
    }

    #[test]
    fn prefix_extraction() {
        let hash = 0xABCD_1234_5678_9EF0u64 as i64;
        assert_eq!(hash_prefix(hash), 0xABCDu16 as i16);
    }

    #[test]
    fn similarity_threshold() {
        let a = 0x0000000000000000i64;
        let b = 0x00000000000003FFi64; // 10 bits different
        assert!(is_similar(a, b, 10));
        assert!(!is_similar(a, b, 9));
    }

    #[test]
    fn phash_identical_images() {
        let img = DynamicImage::new_rgb8(100, 100);
        let h1 = phash64(&img);
        let h2 = phash64(&img);
        assert_eq!(h1, h2);
    }
}
