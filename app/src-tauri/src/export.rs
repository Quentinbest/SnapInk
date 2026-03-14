use base64::{engine::general_purpose, Engine as _};
use std::fs;

use crate::settings::load_settings;

fn expand_filename_pattern(pattern: &str, format: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple expansion using current time components
    let dt = format_datetime(now);
    let name = pattern
        .replace("{YYYY}", &dt.year)
        .replace("{MM}", &dt.month)
        .replace("{DD}", &dt.day)
        .replace("{HH}", &dt.hour)
        .replace("{mm}", &dt.minute)
        .replace("{ss}", &dt.second);
    format!("{}.{}", name, format)
}

struct DateTime {
    year: String,
    month: String,
    day: String,
    hour: String,
    minute: String,
    second: String,
}

fn format_datetime(epoch_secs: u64) -> DateTime {
    // Simple manual datetime conversion (no chrono dep to keep binary small)
    let secs = epoch_secs;
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Days since epoch to year/month/day
    let (y, mo, d) = days_to_ymd(days);
    DateTime {
        year: format!("{:04}", y),
        month: format!("{:02}", mo),
        day: format!("{:02}", d),
        hour: format!("{:02}", h),
        minute: format!("{:02}", m),
        second: format!("{:02}", s),
    }
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let mut y = 1970u64;
    let mut rem = days;
    loop {
        let leap = is_leap(y);
        let yd = if leap { 366 } else { 365 };
        if rem < yd {
            break;
        }
        rem -= yd;
        y += 1;
    }
    let months = [31u64, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mo = 1u64;
    for &md in &months {
        if rem < md {
            break;
        }
        rem -= md;
        mo += 1;
    }
    (y, mo, rem + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[tauri::command]
pub fn export_to_file(image_data: String, path: String) -> Result<(), String> {
    let bytes = general_purpose::STANDARD
        .decode(&image_data)
        .map_err(|e| e.to_string())?;
    fs::write(&path, &bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_default_save_path() -> String {
    let settings = load_settings();
    settings.output.save_path
}

#[tauri::command]
pub fn expand_filename(pattern: String, format: String) -> String {
    expand_filename_pattern(&pattern, &format)
}
