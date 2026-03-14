use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;
use xcap::{Monitor, Window};

use crate::types::{CaptureRegion, MonitorInfo, WindowInfo};

fn image_to_base64_png(img: &DynamicImage) -> Result<String, String> {
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
    let rel_x = (region.x - mx).max(0) as u32;
    let rel_y = (region.y - my).max(0) as u32;
    let w = region.width.min(dyn_img.width().saturating_sub(rel_x));
    let h = region.height.min(dyn_img.height().saturating_sub(rel_y));

    if w == 0 || h == 0 {
        return Err("Invalid region dimensions".to_string());
    }

    let cropped = dyn_img.crop_imm(rel_x, rel_y, w, h);
    image_to_base64_png(&cropped)
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
