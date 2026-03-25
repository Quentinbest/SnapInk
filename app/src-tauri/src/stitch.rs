use image::{DynamicImage, RgbaImage};

use crate::capture::image_to_base64_png;

/// Find the vertical overlap in pixels between two images using column sampling.
///
/// Compares the bottom strip of `top` with the top strip of `bottom`, searching
/// from the largest candidate overlap down to the smallest. Returns 0 if no
/// matching overlap is found within the search range.
///
/// ## Why search up to h-5?
/// At 400 ms capture intervals, slow trackpad scrolling moves as little as
/// 5–50 physical pixels per frame, meaning the real overlap can be 95–99%
/// of the frame height. A cap at 80% would miss those slow-scroll cases,
/// cause find_overlap to return 0, and let the stitcher include the full
/// subsequent frame — producing visible duplicate regions in the output.
pub fn find_overlap(top: &DynamicImage, bottom: &DynamicImage) -> u32 {
    let w = top.width().min(bottom.width());
    if w == 0 {
        return 0;
    }

    let h_top = top.height();
    let h_bot = bottom.height();

    // Minimum scroll we require before treating frames as distinct: 5 physical px.
    // Maximum overlap: everything except those 5 px (covers even very slow scrolling).
    let min_overlap: u32 = 10;
    let max_overlap = h_top.min(h_bot).saturating_sub(5).max(min_overlap);

    // Sample 5 columns evenly spaced across the image width.
    let cols: Vec<u32> = (1..=5).map(|i| (w * i / 6).min(w.saturating_sub(1))).collect();

    let top_rgba = top.to_rgba8();
    let bot_rgba = bottom.to_rgba8();

    // Search from largest overlap to smallest — the first match is the correct one
    // because greedy-largest avoids returning a small false-positive that happens to
    // score above the threshold by chance.
    for n in (min_overlap..=max_overlap).rev() {
        // Sample ~20 rows evenly distributed across the candidate overlap strip.
        // More samples → fewer false positives, especially for near-full overlaps.
        let row_step = (n / 20).max(1);
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

        // Row-wise copy: RgbaImage stores rows contiguously (RGBA = 4 bytes/pixel).
        // copy_from_slice is ~10x faster than per-pixel put_pixel (eliminates
        // bounds checks and function call overhead per pixel).
        let bytes_per_row = (frame_w * 4) as usize;
        for y in skip..src.height() {
            let src_offset = (y * src.width() * 4) as usize;
            let dst_offset = (y_out * width * 4) as usize;
            result.as_mut()[dst_offset..dst_offset + bytes_per_row]
                .copy_from_slice(&src.as_raw()[src_offset..src_offset + bytes_per_row]);
            y_out += 1;
        }
    }

    image_to_base64_png(&DynamicImage::ImageRgba8(result))
}

/// Stitch frames from raw PNG byte vectors (no base64 involved).
/// Thin wrapper: decodes PNG bytes → DynamicImage, then delegates to `stitch_frames()`.
pub fn stitch_frames_from_bytes(frames: Vec<Vec<u8>>) -> Result<String, String> {
    let images: Result<Vec<DynamicImage>, String> = frames
        .iter()
        .enumerate()
        .map(|(i, bytes)| {
            image::load_from_memory(bytes)
                .map_err(|e| format!("Failed to decode frame {i}: {e}"))
        })
        .collect();

    stitch_frames(images?)
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
    fn test_find_overlap_normal_scroll() {
        // 20% scroll: overlap is 80% of frame height (previously failed with 80% cap).
        let full = gradient(200, 400, 0);
        let top = full.crop_imm(0, 0, 200, 220);
        let bottom = full.crop_imm(0, 180, 200, 220); // 40px scroll → 180px overlap
        assert_eq!(find_overlap(&top, &bottom), 40, "Normal scroll overlap");
    }

    #[test]
    fn test_find_overlap_slow_scroll() {
        // Slow scroll: only 10px moved → overlap is 210 of 220 rows (~95%).
        // This was the main failure case: the old 80% cap (176px max) would miss it.
        let full = gradient(200, 400, 0);
        let top = full.crop_imm(0, 0, 200, 220);
        let bottom = full.crop_imm(0, 10, 200, 220); // 10px scroll → 210px overlap
        assert_eq!(find_overlap(&top, &bottom), 210, "Slow scroll overlap");
    }

    #[test]
    fn test_find_overlap_very_slow_scroll() {
        // Very slow scroll: only 6px moved → overlap is 214 of 220 rows (~97%).
        let full = gradient(200, 400, 0);
        let top = full.crop_imm(0, 0, 200, 220);
        let bottom = full.crop_imm(0, 6, 200, 220); // 6px scroll → 214px overlap
        assert_eq!(find_overlap(&top, &bottom), 214, "Very slow scroll overlap");
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
    fn test_stitch_two_frames_normal_scroll() {
        // Stitch two overlapping crops — result should equal original dimensions.
        let full = gradient(200, 400, 0);
        let top = full.crop_imm(0, 0, 200, 220);
        let bottom = full.crop_imm(0, 180, 200, 220); // 40px scroll

        let result_b64 = stitch_frames(vec![top, bottom]).unwrap();
        let bytes = general_purpose::STANDARD.decode(&result_b64).unwrap();
        let result_img = image::load_from_memory(&bytes).unwrap();

        assert_eq!(result_img.width(), 200);
        assert_eq!(result_img.height(), 400);
    }

    #[test]
    fn test_stitch_two_frames_slow_scroll() {
        // Regression test: slow scroll (10px) was the main failure case.
        // Old algorithm (80% cap) returned overlap=0, causing full-frame duplication.
        let full = gradient(200, 400, 0);
        let top = full.crop_imm(0, 0, 200, 220);
        let bottom = full.crop_imm(0, 10, 200, 220); // only 10px scroll

        let result_b64 = stitch_frames(vec![top, bottom]).unwrap();
        let bytes = general_purpose::STANDARD.decode(&result_b64).unwrap();
        let result_img = image::load_from_memory(&bytes).unwrap();

        // Expected height: 220 (top) + 220 - 210 (new rows from bottom) = 230
        assert_eq!(result_img.width(), 200);
        assert_eq!(result_img.height(), 230, "Slow-scroll stitch height mismatch");
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
    fn test_stitch_frames_from_bytes_two_frames() {
        let full = gradient(200, 400, 0);
        let top = full.crop_imm(0, 0, 200, 220);
        let bottom = full.crop_imm(0, 180, 200, 220);

        // Encode to PNG bytes
        fn to_png_bytes(img: &DynamicImage) -> Vec<u8> {
            let mut buf = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
            buf.into_inner()
        }

        let frames = vec![to_png_bytes(&top), to_png_bytes(&bottom)];
        let result_b64 = stitch_frames_from_bytes(frames).unwrap();
        let bytes = general_purpose::STANDARD.decode(&result_b64).unwrap();
        let result_img = image::load_from_memory(&bytes).unwrap();

        assert_eq!(result_img.width(), 200);
        assert_eq!(result_img.height(), 400);
    }

    #[test]
    fn test_stitch_frames_from_bytes_empty() {
        assert!(stitch_frames_from_bytes(vec![]).is_err());
    }

    #[test]
    fn test_stitch_frames_from_bytes_invalid_png() {
        let bad_data = vec![vec![0u8, 1, 2, 3]]; // not valid PNG
        assert!(stitch_frames_from_bytes(bad_data).is_err());
    }

    /// CRITICAL REGRESSION TEST: Verify row-wise copy_from_slice produces
    /// identical results. If the byte offsets are wrong, stitching silently
    /// produces corrupted output.
    #[test]
    fn test_row_copy_produces_correct_output() {
        // Stitch two overlapping frames and verify specific pixel values
        // match the original gradient.
        let full = gradient(200, 400, 0);
        let top = full.crop_imm(0, 0, 200, 220);
        let bottom = full.crop_imm(0, 180, 200, 220);

        let result_b64 = stitch_frames(vec![top, bottom]).unwrap();
        let bytes = general_purpose::STANDARD.decode(&result_b64).unwrap();
        let result_img = image::load_from_memory(&bytes).unwrap().to_rgba8();

        // Check a few pixels from the original gradient match
        let full_rgba = full.to_rgba8();
        for &y in &[0, 100, 200, 300, 399] {
            for &x in &[0, 50, 100, 150, 199] {
                let expected = full_rgba.get_pixel(x, y);
                let actual = result_img.get_pixel(x, y);
                assert_eq!(
                    expected, actual,
                    "Pixel mismatch at ({x}, {y}): expected {expected:?}, got {actual:?}"
                );
            }
        }
    }

    #[test]
    fn test_stitch_three_frames() {
        // Three overlapping frames: verifies cumulative overlap detection.
        let full = gradient(200, 600, 0);
        let f1 = full.crop_imm(0, 0, 200, 220);
        let f2 = full.crop_imm(0, 180, 200, 220); // 40px scroll
        let f3 = full.crop_imm(0, 360, 200, 220); // 40px scroll

        let result_b64 = stitch_frames(vec![f1, f2, f3]).unwrap();
        let bytes = general_purpose::STANDARD.decode(&result_b64).unwrap();
        let result_img = image::load_from_memory(&bytes).unwrap();

        assert_eq!(result_img.width(), 200);
        // 220 + (220-40) + (220-40) = 580
        assert_eq!(result_img.height(), 580, "Three-frame stitch height mismatch");
    }

    #[test]
    fn test_stitch_four_frames_slow_scroll() {
        // Four frames with slow scroll (10px each).
        let full = gradient(200, 300, 0);
        let f1 = full.crop_imm(0, 0, 200, 100);
        let f2 = full.crop_imm(0, 10, 200, 100);
        let f3 = full.crop_imm(0, 20, 200, 100);
        let f4 = full.crop_imm(0, 30, 200, 100);

        let result_b64 = stitch_frames(vec![f1, f2, f3, f4]).unwrap();
        let bytes = general_purpose::STANDARD.decode(&result_b64).unwrap();
        let result_img = image::load_from_memory(&bytes).unwrap();

        assert_eq!(result_img.width(), 200);
        // 100 + 10 + 10 + 10 = 130
        assert_eq!(result_img.height(), 130, "Four-frame slow stitch height mismatch");
    }
}
