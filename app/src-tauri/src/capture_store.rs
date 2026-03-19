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
pub struct ScrollCaptureStore {
    pub region: Mutex<Option<CaptureRegion>>,
    pub frames: Mutex<Vec<String>>,
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

/// Capture the stored scroll region and append one frame (shared logic).
/// Skips the frame if identical to the previous one (deduplication).
/// Returns the new total frame count.
pub(crate) fn add_frame_to_store(scroll_store: &ScrollCaptureStore) -> Result<usize, String> {
    let region = scroll_store
        .region
        .lock()
        .unwrap()
        .clone()
        .ok_or("No scroll capture region set")?;

    let frame_data = crate::capture::capture_region_sync(&region)?;

    let mut frames = scroll_store.frames.lock().unwrap();

    // Skip duplicate frames (no scroll happened yet).
    if let Some(last) = frames.last() {
        if crate::stitch::is_duplicate(last, &frame_data) {
            return Ok(frames.len());
        }
    }

    frames.push(frame_data);
    Ok(frames.len())
}

/// IPC command: capture one frame manually (kept for backwards compatibility).
#[tauri::command]
pub fn scroll_capture_add_frame(
    scroll_store: tauri::State<'_, ScrollCaptureStore>,
) -> Result<usize, String> {
    add_frame_to_store(&scroll_store)
}

/// Stitch all captured frames and return the result as a base64-encoded PNG.
#[tauri::command]
pub fn stitch_scroll_frames(
    scroll_store: tauri::State<'_, ScrollCaptureStore>,
) -> Result<String, String> {
    use base64::{engine::general_purpose, Engine as _};

    let frames_b64 = scroll_store.frames.lock().unwrap().clone();
    if frames_b64.is_empty() {
        return Err("No frames captured".to_string());
    }

    let frames: Result<Vec<image::DynamicImage>, String> = frames_b64
        .iter()
        .map(|b64| {
            let bytes = general_purpose::STANDARD
                .decode(b64)
                .map_err(|e| e.to_string())?;
            image::load_from_memory(&bytes).map_err(|e| e.to_string())
        })
        .collect();

    crate::stitch::stitch_frames(frames?)
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
