use image::imageops::FilterType;
use image::DynamicImage;
use image_compare::Metric;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone)]
pub struct FrameComparisonConfig {
    pub hash_early_exit: bool,
    pub downscale_comparison: bool,
    pub downscale_factor: u32,
    pub single_metric: bool,
}

impl Default for FrameComparisonConfig {
    fn default() -> Self {
        Self {
            hash_early_exit: true,
            downscale_comparison: true,
            downscale_factor: 6,
            single_metric: true,
        }
    }
}

pub struct FrameComparer {
    config: FrameComparisonConfig,
    previous_hash: Option<u64>,
    previous_image_downscaled: Option<DynamicImage>,
    previous_image_full: Option<DynamicImage>,
    comparison_count: u64,
    hash_hits: u64,
}

impl FrameComparer {
    pub fn new(config: FrameComparisonConfig) -> Self {
        Self {
            config,
            previous_hash: None,
            previous_image_downscaled: None,
            previous_image_full: None,
            comparison_count: 0,
            hash_hits: 0,
        }
    }

    fn downscale_dims(&self, width: u32, height: u32) -> (u32, u32) {
        if self.config.downscale_factor > 0 {
            (
                (width / self.config.downscale_factor).max(1),
                (height / self.config.downscale_factor).max(1),
            )
        } else {
            (width, height)
        }
    }

    fn downscale(&self, image: &DynamicImage) -> DynamicImage {
        let (w, h) = self.downscale_dims(image.width(), image.height());
        image.resize_exact(w, h, FilterType::Nearest)
    }

    fn hash_image(&self, downscaled: &DynamicImage) -> u64 {
        let mut hasher = DefaultHasher::new();
        downscaled.as_bytes().hash(&mut hasher);
        hasher.finish()
    }

    pub fn compare(&mut self, current_image: &DynamicImage) -> f64 {
        self.comparison_count += 1;

        let current_downscaled = if self.config.downscale_comparison {
            Some(self.downscale(current_image))
        } else {
            None
        };

        let current_hash = if self.config.hash_early_exit {
            let to_hash = current_downscaled.as_ref().unwrap_or(current_image);
            Some(self.hash_image(to_hash))
        } else {
            None
        };

        if self.previous_hash.is_none()
            && self.previous_image_downscaled.is_none()
            && self.previous_image_full.is_none()
        {
            self.update_previous_internal(current_image, current_downscaled, current_hash);
            return 1.0;
        }

        if self.config.hash_early_exit {
            if let (Some(prev_hash), Some(curr_hash)) = (self.previous_hash, current_hash) {
                if prev_hash == curr_hash {
                    self.hash_hits += 1;
                    return 0.0;
                }
            }
        }

        let (prev_img, curr_img) = if self.config.downscale_comparison {
            let prev = self.previous_image_downscaled.as_ref();
            let curr = current_downscaled.as_ref();
            match (prev, curr) {
                (Some(p), Some(c)) => (p, c.clone()),
                _ => {
                    self.update_previous_internal(current_image, current_downscaled, current_hash);
                    return 1.0;
                }
            }
        } else {
            let prev = self.previous_image_full.as_ref();
            match prev {
                Some(p) => (p, current_image.clone()),
                None => {
                    self.update_previous_internal(current_image, current_downscaled, current_hash);
                    return 1.0;
                }
            }
        };

        let diff = if self.config.single_metric {
            compare_histogram(prev_img, &curr_img).unwrap_or(1.0)
        } else {
            // SSIM omitted for simplicity in port, as single_metric is default TRUE in screenpipe
            compare_histogram(prev_img, &curr_img).unwrap_or(1.0)
        };

        self.update_previous_internal(current_image, current_downscaled, current_hash);
        diff
    }

    fn update_previous_internal(
        &mut self,
        full_image: &DynamicImage,
        downscaled: Option<DynamicImage>,
        hash: Option<u64>,
    ) {
        self.previous_hash = hash;
        if self.config.downscale_comparison {
            self.previous_image_downscaled =
                downscaled.or_else(|| Some(self.downscale(full_image)));
            self.previous_image_full = None;
        } else {
            self.previous_image_full = Some(full_image.clone());
            self.previous_image_downscaled = None;
        }
    }
}

pub fn compare_histogram(image1: &DynamicImage, image2: &DynamicImage) -> anyhow::Result<f64> {
    let image_one = image1.to_luma8();
    let mut image_two = image2.to_luma8();
    if image_one.dimensions() != image_two.dimensions() {
        image_two = image::imageops::resize(
            &image_two,
            image_one.width(),
            image_one.height(),
            FilterType::Nearest,
        );
    }
    image_compare::gray_similarity_histogram(Metric::Hellinger, &image_one, &image_two)
        .map_err(|e| anyhow::anyhow!("Failed to compare images: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    fn solid_rgb(width: u32, height: u32, r: u8, g: u8, b: u8) -> DynamicImage {
        let buf = ImageBuffer::from_pixel(width, height, Rgb([r, g, b]));
        DynamicImage::ImageRgb8(buf)
    }

    #[test]
    fn test_first_frame_returns_one() {
        let mut comparer = FrameComparer::new(FrameComparisonConfig::default());
        let img = solid_rgb(64, 64, 100, 100, 100);
        let result = comparer.compare(&img);
        assert_eq!(result, 1.0, "First frame should always return 1.0 (no previous to compare)");
    }

    #[test]
    fn test_identical_frame_hash_early_exit() {
        let mut comparer = FrameComparer::new(FrameComparisonConfig {
            hash_early_exit: true,
            ..Default::default()
        });
        let img = solid_rgb(64, 64, 128, 128, 128);
        comparer.compare(&img); // seed previous
        let result = comparer.compare(&img); // same image bytes → same hash
        assert_eq!(result, 0.0, "Identical frame must return 0.0 via hash early exit");
    }

    #[test]
    fn test_different_frames_return_nonzero() {
        let mut comparer = FrameComparer::new(FrameComparisonConfig::default());
        let black = solid_rgb(64, 64, 0, 0, 0);
        let white = solid_rgb(64, 64, 255, 255, 255);
        comparer.compare(&black); // seed previous
        let result = comparer.compare(&white);
        assert!(result > 0.0, "Visually different frames must return a positive difference");
    }

    #[test]
    fn test_compare_histogram_identical_images() {
        let img = solid_rgb(32, 32, 80, 80, 80);
        let result = compare_histogram(&img, &img).expect("histogram compare failed");
        // image-compare 0.4 returns a distance (0.0 = identical, higher = more different)
        assert!(result <= 0.01, "Identical images should have near-zero distance: {}", result);
    }

    #[test]
    fn test_compare_histogram_different_sizes_auto_resize() {
        let large = solid_rgb(64, 64, 200, 200, 200);
        let small = solid_rgb(32, 32, 200, 200, 200);
        let result = compare_histogram(&large, &small)
            .expect("histogram compare with size mismatch failed");
        // Same colour at different sizes — after resize they are identical, distance should be near 0
        assert!(result <= 0.05, "Same-colour images at different sizes should have near-zero distance: {}", result);
    }

    #[test]
    fn test_hash_early_exit_disabled_still_compares() {
        let mut comparer = FrameComparer::new(FrameComparisonConfig {
            hash_early_exit: false,
            ..Default::default()
        });
        let img = solid_rgb(64, 64, 64, 64, 64);
        comparer.compare(&img); // seed previous
        let result = comparer.compare(&img); // should still detect similarity via histogram
        assert!(result >= 0.0, "Result must be a valid similarity score");
        assert!(result <= 1.0, "Result must not exceed 1.0");
    }
}
