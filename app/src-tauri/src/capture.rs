use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;
use xcap::{Monitor, Window};

use crate::types::{CaptureRegion, MonitorInfo, WindowInfo};

/// Crop a region from a full-screen image and return as base64-encoded PNG.
/// Shared by `capture_region` (IPC) and `capture_region_sync` (internal).
fn crop_and_encode(
    img: &DynamicImage,
    monitor_x: i32,
    monitor_y: i32,
    region: &CaptureRegion,
) -> Result<String, String> {
    let rel_x = (region.x - monitor_x).max(0) as u32;
    let rel_y = (region.y - monitor_y).max(0) as u32;
    let w = region.width.min(img.width().saturating_sub(rel_x));
    let h = region.height.min(img.height().saturating_sub(rel_y));

    if w == 0 || h == 0 {
        return Err("Invalid region dimensions".to_string());
    }

    let cropped = img.crop_imm(rel_x, rel_y, w, h);
    image_to_base64_png(&cropped)
}

pub(crate) fn image_to_base64_png(img: &DynamicImage) -> Result<String, String> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(general_purpose::STANDARD.encode(buf.get_ref()))
}

#[tauri::command]
pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    let result = monitors
        .iter()
        .enumerate()
        .map(|(i, m)| MonitorInfo {
            id: i as u32,
            name: m.name().unwrap_or_default(),
            x: m.x().unwrap_or_default(),
            y: m.y().unwrap_or_default(),
            width: m.width().unwrap_or_default(),
            height: m.height().unwrap_or_default(),
            scale_factor: m.scale_factor().unwrap_or(1.0) as f64,
            is_primary: i == 0,
        })
        .collect();
    Ok(result)
}

#[tauri::command]
pub fn get_windows() -> Result<Vec<WindowInfo>, String> {
    let windows = Window::all().map_err(|e| e.to_string())?;
    let result = windows
        .iter()
        .filter(|w| !w.title().unwrap_or_default().is_empty())
        .map(|w| WindowInfo {
            id: w.id().unwrap_or_default(),
            title: w.title().unwrap_or_default(),
            app_name: w.app_name().unwrap_or_default(),
            x: w.x().unwrap_or_default(),
            y: w.y().unwrap_or_default(),
            width: w.width().unwrap_or_default(),
            height: w.height().unwrap_or_default(),
        })
        .collect();
    Ok(result)
}

#[tauri::command]
pub fn capture_fullscreen(monitor_index: usize) -> Result<String, String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    let monitor = monitors
        .get(monitor_index)
        .ok_or("Monitor not found")?;
    let img = monitor.capture_image().map_err(|e| e.to_string())?;
    let dyn_img = DynamicImage::ImageRgba8(img);
    image_to_base64_png(&dyn_img)
}

#[tauri::command]
pub fn capture_region(region: CaptureRegion, monitor_index: usize) -> Result<String, String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    let monitor = monitors
        .get(monitor_index)
        .ok_or("Monitor not found")?;
    let img = monitor.capture_image().map_err(|e| e.to_string())?;
    let dyn_img = DynamicImage::ImageRgba8(img);
    let mx = monitor.x().unwrap_or_default();
    let my = monitor.y().unwrap_or_default();
    crop_and_encode(&dyn_img, mx, my, &region)
}


/// Capture a specific region of the screen directly as PNG bytes.
///
/// Uses `CGWindowListCreateImage` to capture composited window content at the
/// requested rectangle. This properly reflects scroll changes (unlike
/// `CGDisplayCreateImageForRect` which captures physical display pixels).
///
/// `CaptureRegion` stores **physical pixel** coordinates (Svelte multiplies by
/// `scaleFactor`). `CGWindowListCreateImage` expects **point** coordinates,
/// so we divide by the display scale factor.
#[cfg(target_os = "macos")]
#[allow(deprecated)] // Apple deprecates CGWindowListCreateImage in favor of ScreenCaptureKit
pub fn capture_region_direct(region: &CaptureRegion) -> Result<Vec<u8>, String> {
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    #[allow(deprecated)]
    use objc2_core_graphics::{
        kCGNullWindowID, CGDataProvider, CGDisplayBounds, CGImage, CGMainDisplayID,
        CGWindowImageOption, CGWindowListCreateImage, CGWindowListOption,
    };

    let display_id = CGMainDisplayID();

    // Compute scale factor: physical pixels / logical points
    let bounds = CGDisplayBounds(display_id);
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    let scale = monitors
        .first()
        .and_then(|m| m.scale_factor().ok())
        .unwrap_or(1.0) as f64;

    // Convert physical-pixel coords → point coords for CG API
    let rect = CGRect {
        origin: CGPoint {
            x: region.x as f64 / scale + bounds.origin.x,
            y: region.y as f64 / scale + bounds.origin.y,
        },
        size: CGSize {
            width: region.width as f64 / scale,
            height: region.height as f64 / scale,
        },
    };

    let cg_image = CGWindowListCreateImage(
        rect,
        CGWindowListOption::OptionOnScreenOnly,
        kCGNullWindowID,
        CGWindowImageOption::BestResolution,
    )
    .ok_or("Screen capture failed — check Screen Recording permission")?;

    // Extract raw pixel data from CGImage (same pattern as xcap)
    let width = CGImage::width(Some(&cg_image));
    let height = CGImage::height(Some(&cg_image));
    let bytes_per_row = CGImage::bytes_per_row(Some(&cg_image));

    let data_provider = CGImage::data_provider(Some(&cg_image))
        .ok_or("Failed to get CGImage data provider")?;
    let raw_data = CGDataProvider::data(Some(&data_provider))
        .ok_or("Failed to copy pixel data from CGImage")?
        .to_vec();

    // Handle row padding: bytes_per_row may exceed width * 4
    let mut buffer = Vec::with_capacity(width * height * 4);
    for row in raw_data.chunks_exact(bytes_per_row) {
        buffer.extend_from_slice(&row[..width * 4]);
    }

    // CGImage uses BGRA byte order — swap to RGBA
    for bgra in buffer.chunks_exact_mut(4) {
        bgra.swap(0, 2);
    }

    let rgba_image = image::RgbaImage::from_raw(width as u32, height as u32, buffer)
        .ok_or("Failed to create RgbaImage from captured pixels")?;

    // Encode to PNG bytes
    let mut png_bytes = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(rgba_image)
        .write_to(&mut png_bytes, ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    Ok(png_bytes.into_inner())
}

#[cfg(not(target_os = "macos"))]
pub fn capture_region_direct(_region: &CaptureRegion) -> Result<Vec<u8>, String> {
    Err("capture_region_direct is only supported on macOS".to_string())
}

pub fn take_screenshot_sync() -> Result<String, String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    let monitor = monitors.first().ok_or("No monitor found")?;
    let img = monitor.capture_image().map_err(|e| e.to_string())?;
    let dyn_img = DynamicImage::ImageRgba8(img);
    image_to_base64_png(&dyn_img)
}

#[tauri::command]
pub fn capture_window_by_id(window_id: u32) -> Result<String, String> {
    let windows = Window::all().map_err(|e| e.to_string())?;
    let window = windows
        .iter()
        .find(|w| w.id().unwrap_or_default() == window_id)
        .ok_or("Window not found")?;
    let img = window.capture_image().map_err(|e| e.to_string())?;
    let dyn_img = DynamicImage::ImageRgba8(img);
    image_to_base64_png(&dyn_img)
}
