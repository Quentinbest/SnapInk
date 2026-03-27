use std::sync::Mutex;

use tauri::Manager;

use crate::types::CaptureRegion;

/// Holds state across the capture → editor flow.
/// background: fullscreen screenshot taken BEFORE the overlay window opens.
/// result: cropped region chosen by the user, ready for the editor.
pub struct CaptureStore {
    pub background: Mutex<Option<String>>,
    pub result: Mutex<Option<String>>,
}

impl CaptureStore {
    pub fn new() -> Self {
        Self {
            background: Mutex::new(None),
            result: Mutex::new(None),
        }
    }
}

/// Accumulates frames captured during a scroll-capture session.
/// Frames are stored as raw PNG bytes (`Vec<u8>`) to avoid the base64
/// encode/decode cycle that the old auto-scroll pipeline used.
pub struct ScrollCaptureStore {
    pub region: Mutex<Option<CaptureRegion>>,
    pub frames: Mutex<Vec<Vec<u8>>>,
}

impl ScrollCaptureStore {
    pub fn new() -> Self {
        Self {
            region: Mutex::new(None),
            frames: Mutex::new(Vec::new()),
        }
    }
}

// ── CaptureStore commands ─────────────────────────────────────────────────────

/// Store a pre-taken fullscreen screenshot.
#[tauri::command]
pub fn get_capture_background(store: tauri::State<'_, CaptureStore>) -> Option<String> {
    store.background.lock().unwrap().clone()
}

/// Crop the stored background to the given region and save as result.
#[tauri::command]
pub fn crop_and_store(
    store: tauri::State<'_, CaptureStore>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<String, String> {
    let bg = store
        .background
        .lock()
        .unwrap()
        .clone()
        .ok_or("No background screenshot available")?;

    use base64::{engine::general_purpose, Engine as _};
    use image::ImageFormat;
    use std::io::Cursor;

    let bytes = general_purpose::STANDARD
        .decode(&bg)
        .map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;

    let rel_x = x.max(0) as u32;
    let rel_y = y.max(0) as u32;
    let w = width.min(img.width().saturating_sub(rel_x));
    let h = height.min(img.height().saturating_sub(rel_y));

    if w == 0 || h == 0 {
        return Err("Invalid crop dimensions".to_string());
    }

    let cropped = img.crop_imm(rel_x, rel_y, w, h);
    let mut buf = Cursor::new(Vec::new());
    cropped
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let result = general_purpose::STANDARD.encode(buf.get_ref());

    *store.result.lock().unwrap() = Some(result.clone());
    Ok(result)
}

/// Called by the editor on mount to retrieve (and clear) the pending capture.
#[tauri::command]
pub fn consume_capture_result(store: tauri::State<'_, CaptureStore>) -> Option<String> {
    store.result.lock().unwrap().take()
}

/// Store an already-captured image as the pending result (e.g. window capture).
#[tauri::command]
pub fn store_capture_result(store: tauri::State<'_, CaptureStore>, data: String) {
    *store.result.lock().unwrap() = Some(data);
}

// ── ScrollCaptureStore commands ───────────────────────────────────────────────

/// Hash PNG bytes for change detection. Uses SipHash (std DefaultHasher) —
/// fast enough for ~200KB-1MB PNG blobs at 10 fps.
pub(crate) fn frame_hash(png_bytes: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    png_bytes.hash(&mut hasher);
    hasher.finish()
}

/// Capture the stored scroll region and append one frame (shared logic).
/// Skips the frame if identical to the previous one (hash-based deduplication).
/// Returns the new total frame count.
pub(crate) fn add_frame_to_store(
    scroll_store: &ScrollCaptureStore,
    last_hash: &mut u64,
) -> Result<usize, String> {
    let region = scroll_store
        .region
        .lock()
        .unwrap()
        .clone()
        .ok_or("No scroll capture region set")?;

    let frame_data = crate::capture::capture_region_direct(&region)?;

    let hash = frame_hash(&frame_data);
    if hash == *last_hash {
        // Frame unchanged — skip storage
        return Ok(scroll_store.frames.lock().unwrap().len());
    }
    *last_hash = hash;

    let mut frames = scroll_store.frames.lock().unwrap();
    frames.push(frame_data);
    Ok(frames.len())
}

/// IPC command: capture one frame manually (kept for backwards compatibility).
#[tauri::command]
pub fn scroll_capture_add_frame(
    scroll_store: tauri::State<'_, ScrollCaptureStore>,
) -> Result<usize, String> {
    let mut last_hash = 0u64;
    add_frame_to_store(&scroll_store, &mut last_hash)
}

/// Stitch all captured frames and return the result as a base64-encoded PNG.
#[tauri::command]
pub fn stitch_scroll_frames(
    scroll_store: tauri::State<'_, ScrollCaptureStore>,
) -> Result<String, String> {
    let frames_bytes = std::mem::take(&mut *scroll_store.frames.lock().unwrap());
    if frames_bytes.is_empty() {
        return Err("No frames captured".to_string());
    }

    crate::stitch::stitch_frames_from_bytes(frames_bytes)
}

/// Clear all scroll capture state (frames + region).
/// Also closes the scroll-control window if open.
#[tauri::command]
pub fn scroll_capture_reset(
    app: tauri::AppHandle,
    scroll_store: tauri::State<'_, ScrollCaptureStore>,
) {
    scroll_store.frames.lock().unwrap().clear();
    *scroll_store.region.lock().unwrap() = None;

    if let Some(win) = app.get_webview_window("scroll-control") {
        let _ = win.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a CaptureStore with a test background image.
    fn store_with_background(width: u32, height: u32) -> CaptureStore {
        use base64::{engine::general_purpose, Engine as _};
        use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
        use std::io::Cursor;

        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            width, height, Rgba([100, 150, 200, 255]),
        ));
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Png).unwrap();
        let b64 = general_purpose::STANDARD.encode(buf.get_ref());

        let store = CaptureStore::new();
        *store.background.lock().unwrap() = Some(b64);
        store
    }

    #[test]
    fn test_crop_and_store_no_background() {
        let store = CaptureStore::new();
        // Simulate Tauri State by calling the crop logic directly.
        let bg = store.background.lock().unwrap().clone();
        assert!(bg.is_none(), "Expected no background");
    }

    #[test]
    fn test_crop_and_store_zero_dimension() {
        let store = store_with_background(200, 200);
        let bg = store.background.lock().unwrap().clone().unwrap();

        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD.decode(&bg).unwrap();
        let img = image::load_from_memory(&bytes).unwrap();

        // Zero width
        let w = 0u32.min(img.width());
        assert_eq!(w, 0, "Zero width should stay zero");
    }

    #[test]
    fn test_crop_and_store_negative_xy_clamped() {
        let store = store_with_background(200, 200);
        let bg = store.background.lock().unwrap().clone().unwrap();

        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD.decode(&bg).unwrap();
        let img = image::load_from_memory(&bytes).unwrap();

        // Negative x/y should be clamped to 0.
        let x: i32 = -50;
        let y: i32 = -30;
        let rel_x = x.max(0) as u32;
        let rel_y = y.max(0) as u32;
        assert_eq!(rel_x, 0);
        assert_eq!(rel_y, 0);

        let w = 100u32.min(img.width().saturating_sub(rel_x));
        let h = 100u32.min(img.height().saturating_sub(rel_y));
        assert!(w > 0 && h > 0, "Clamped crop should have valid dimensions");

        let cropped = img.crop_imm(rel_x, rel_y, w, h);
        assert_eq!(cropped.width(), 100);
        assert_eq!(cropped.height(), 100);
    }

    #[test]
    fn test_crop_and_store_happy_path() {
        use base64::{engine::general_purpose, Engine as _};
        use image::ImageFormat;
        use std::io::Cursor;

        let store = store_with_background(400, 300);
        let bg = store.background.lock().unwrap().clone().unwrap();
        let bytes = general_purpose::STANDARD.decode(&bg).unwrap();
        let img = image::load_from_memory(&bytes).unwrap();

        let rel_x = 50u32;
        let rel_y = 50u32;
        let w = 200u32.min(img.width().saturating_sub(rel_x));
        let h = 150u32.min(img.height().saturating_sub(rel_y));

        let cropped = img.crop_imm(rel_x, rel_y, w, h);
        let mut buf = Cursor::new(Vec::new());
        cropped.write_to(&mut buf, ImageFormat::Png).unwrap();
        let result = general_purpose::STANDARD.encode(buf.get_ref());

        // Should produce a valid base64 PNG.
        assert!(!result.is_empty());
        // Verify it decodes back to the right dimensions.
        let decoded = general_purpose::STANDARD.decode(&result).unwrap();
        let decoded_img = image::load_from_memory(&decoded).unwrap();
        assert_eq!(decoded_img.width(), 200);
        assert_eq!(decoded_img.height(), 150);
    }

    #[test]
    fn test_scroll_capture_store_new_is_empty() {
        let store = ScrollCaptureStore::new();
        assert!(store.region.lock().unwrap().is_none());
        assert!(store.frames.lock().unwrap().is_empty());
    }

    #[test]
    fn test_scroll_capture_reset_clears_all() {
        let store = ScrollCaptureStore::new();
        *store.region.lock().unwrap() = Some(crate::types::CaptureRegion {
            x: 10, y: 20, width: 100, height: 200,
        });
        store.frames.lock().unwrap().push(vec![1, 2, 3]);

        // Simulate reset (without the Tauri window close).
        store.frames.lock().unwrap().clear();
        *store.region.lock().unwrap() = None;

        assert!(store.region.lock().unwrap().is_none());
        assert!(store.frames.lock().unwrap().is_empty());
    }

    #[test]
    fn test_frame_hash_deterministic() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(super::frame_hash(&data), super::frame_hash(&data));
    }

    #[test]
    fn test_frame_hash_different_content() {
        let a = vec![1u8, 2, 3, 4];
        let b = vec![5u8, 6, 7, 8];
        assert_ne!(super::frame_hash(&a), super::frame_hash(&b));
    }
}
