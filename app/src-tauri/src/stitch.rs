use image::{DynamicImage, RgbaImage};

use crate::capture::image_to_base64_png;

/// Find the vertical overlap in pixels between two images using column sampling.
///
/// Compares the bottom strip of `top` with the top strip of `bottom`, searching
/// from the largest candidate overlap down to the smallest. Returns 0 if no
/// matching overlap is found within the search range.
pub fn find_overlap(top: &DynamicImage, bottom: &DynamicImage) -> u32 {
    let w = top.width().min(bottom.width());
    if w == 0 {
        return 0;
    }

    let h_top = top.height();
    let h_bot = bottom.height();
    let min_overlap: u32 = 10;
    let max_overlap = (h_top.min(h_bot) * 4 / 5).max(min_overlap);

    if max_overlap < min_overlap {
        return 0;
    }

    // Sample 5 columns evenly spaced across the image width.
    let cols: Vec<u32> = (1..=5).map(|i| ((w * i / 6)).min(w.saturating_sub(1))).collect();

    let top_rgba = top.to_rgba8();
    let bot_rgba = bottom.to_rgba8();

    // Search from largest overlap to smallest — the first match is the correct one.
    for n in (min_overlap..=max_overlap).rev() {
        // Sample ~10 rows within the candidate overlap region.
        let row_step = (n / 10).max(1);
        let mut match_count = 0u32;
        let mut total = 0u32;

        let mut row_offset = 0u32;
        while row_offset < n {
            let top_row = h_top - n + row_offset;
            let bot_row = row_offset;

            for &col in &cols {
                let tp = top_rgba.get_pixel(col, top_row);
                let bp = bot_rgba.get_pixel(col, bot_row);
                total += 1;
                if pixels_match(tp, bp, 3) {
                    match_count += 1;
                }
            }
            row_offset += row_step;
        }

        // Require 90% of sampled pixels to match.
        if total > 0 && match_count * 10 >= total * 9 {
            return n;
        }
    }

    0
}

fn pixels_match(a: &image::Rgba<u8>, b: &image::Rgba<u8>, tol: u8) -> bool {
    a[0].abs_diff(b[0]) <= tol && a[1].abs_diff(b[1]) <= tol && a[2].abs_diff(b[2]) <= tol
}

/// Stitch multiple overlapping frames into a single tall image.
/// Returns the stitched result as a base64-encoded PNG string.
pub fn stitch_frames(frames: Vec<DynamicImage>) -> Result<String, String> {
    if frames.is_empty() {
        return Err("No frames to stitch".to_string());
    }
    if frames.len() == 1 {
        return image_to_base64_png(&frames[0]);
    }

    // Detect overlap between each consecutive pair.
    let overlaps: Vec<u32> = (0..frames.len() - 1)
        .map(|i| find_overlap(&frames[i], &frames[i + 1]))
        .collect();

    let width = frames[0].width();

    // Calculate total output height (sum of unique rows contributed by each frame).
    let total_height: u32 = frames[0].height()
        + frames[1..]
            .iter()
            .enumerate()
            .map(|(i, f)| f.height().saturating_sub(overlaps[i]))
            .sum::<u32>();

    if total_height == 0 || width == 0 {
        return Err("Invalid output dimensions".to_string());
    }

    let mut result = RgbaImage::new(width, total_height);
    let mut y_out = 0u32;

    for (i, frame) in frames.iter().enumerate() {
        let skip = if i == 0 { 0 } else { overlaps[i - 1] };
        let src = frame.to_rgba8();
        let frame_w = src.width().min(width);

        for y in skip..src.height() {
            for x in 0..frame_w {
                result.put_pixel(x, y_out, *src.get_pixel(x, y));
            }
            y_out += 1;
        }
    }

    image_to_base64_png(&DynamicImage::ImageRgba8(result))
}

/// Quick check: is this frame identical (or nearly identical) to the previous one?
/// Compares the first `prefix_len` bytes of the base64 string — cheap and effective
/// because any scroll movement changes the PNG header's IDAT chunk.
pub fn is_duplicate(prev_b64: &str, next_b64: &str) -> bool {
    let prefix_len = 1200.min(prev_b64.len()).min(next_b64.len());
    prev_b64[..prefix_len] == next_b64[..prefix_len]
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use image::{Rgba, RgbaImage};

    /// Build a test image where adjacent rows look radically different.
    /// Uses prime-number coefficients so that a y-shift of even 1 row changes
    /// R, G, B by ~59, ~107, ~97 respectively — well above any tolerance threshold.
    fn gradient(width: u32, height: u32, y_offset: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_fn(width, height, |x, y| {
            let row = y + y_offset;
            Rgba([
                ((x.wrapping_mul(37).wrapping_add(row.wrapping_mul(59))) % 256) as u8,
                ((row.wrapping_mul(107).wrapping_add(x.wrapping_mul(13))) % 256) as u8,
                ((x.wrapping_mul(151).wrapping_add(row.wrapping_mul(97))) % 256) as u8,
                255,
            ])
        }))
    }

    #[test]
    fn test_find_overlap_exact() {
        // Full image 200×400. Top frame = rows 0-219, bottom frame = rows 180-399.
        // Real overlap = 40 rows (rows 180-219 shared by both frames).
        let full = gradient(200, 400, 0);
        let top = full.crop_imm(0, 0, 200, 220);
        let bottom = full.crop_imm(0, 180, 200, 220);
        let overlap = find_overlap(&top, &bottom);
        assert_eq!(overlap, 40, "Expected 40px overlap, got {}", overlap);
    }

    #[test]
    fn test_find_overlap_none() {
        let red = DynamicImage::ImageRgba8(RgbaImage::from_pixel(200, 200, Rgba([255, 0, 0, 255])));
        let blue = DynamicImage::ImageRgba8(RgbaImage::from_pixel(200, 200, Rgba([0, 0, 255, 255])));
        assert_eq!(find_overlap(&red, &blue), 0);
    }

    #[test]
    fn test_find_overlap_uniform_images() {
        // Uniform (flat) images have the same pixels everywhere, so any n is a valid
        // overlap. The algorithm should return the maximum candidate it searches.
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(200, 200, Rgba([100, 150, 200, 255])));
        let overlap = find_overlap(&img, &img);
        assert!(overlap > 0, "Uniform images must produce a positive overlap, got {}", overlap);
    }

    #[test]
    fn test_stitch_two_frames_restores_original() {
        // Stitch two overlapping crops of a known gradient — result should equal original.
        let full = gradient(200, 400, 0);
        let top = full.crop_imm(0, 0, 200, 220);
        let bottom = full.crop_imm(0, 180, 200, 220);

        let result_b64 = stitch_frames(vec![top, bottom]).unwrap();
        let bytes = general_purpose::STANDARD.decode(&result_b64).unwrap();
        let result_img = image::load_from_memory(&bytes).unwrap();

        assert_eq!(result_img.width(), 200);
        assert_eq!(result_img.height(), 400);
    }

    #[test]
    fn test_stitch_single_frame_passthrough() {
        let frame = DynamicImage::ImageRgba8(RgbaImage::from_pixel(100, 100, Rgba([0, 0, 0, 255])));
        let result = stitch_frames(vec![frame]).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_stitch_empty_returns_error() {
        assert!(stitch_frames(vec![]).is_err());
    }

    #[test]
    fn test_is_duplicate_identical() {
        let s = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        assert!(is_duplicate(s, s));
    }

    #[test]
    fn test_is_duplicate_different() {
        let a = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let b = "iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAEklEQVQI12P4z8BQDwAEgAF/QualIQAAAABJRU5ErkJggg==";
        assert!(!is_duplicate(a, b));
    }
}
