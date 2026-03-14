use std::sync::Mutex;

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
