use image::imageops::FilterType;
use image::DynamicImage;
use image_compare::Metric;
use std::hash::{DefaultHasher, Hash, Hasher};
use tracing::debug;

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
            let histogram_diff = compare_histogram(prev_img, &curr_img).unwrap_or(1.0);
            // SSIM omitted for simplicity in port, as single_metric is default TRUE in screenpipe
            histogram_diff
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
