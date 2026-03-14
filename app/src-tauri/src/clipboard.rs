use base64::{engine::general_purpose, Engine as _};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub fn read_clipboard_image(app: tauri::AppHandle) -> Result<String, String> {
    let image = app.clipboard().read_image().map_err(|e| e.to_string())?;
    let bytes = image.rgba().to_vec();
    let width = image.width();
    let height = image.height();

    // Convert raw RGBA to PNG
    let img = image::RgbaImage::from_raw(width, height, bytes)
        .ok_or("Failed to construct image")?;
    let dyn_img = image::DynamicImage::ImageRgba8(img);
    let mut buf = std::io::Cursor::new(Vec::new());
    dyn_img
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(general_purpose::STANDARD.encode(buf.get_ref()))
}
