use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub id: u32,
    pub title: String,
    pub app_name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyBinding {
    pub action: String,
    pub shortcut: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSettings {
    pub save_path: String,
    pub filename_pattern: String,
    pub format: String,
    pub jpeg_quality: u8,
    pub retina_clipboard: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSettings {
    pub default_mode: String,
    pub show_cursor: bool,
    pub capture_delay: u32,
    pub play_sound_on_capture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationSettings {
    pub default_color: String,
    pub palette: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSettings {
    pub theme: String,
    pub show_menu_bar_icon: bool,
    pub launch_at_login: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub version: u32,
    pub capture: CaptureSettings,
    pub after_capture: String,
    pub also_copy_after_annotating: bool,
    pub output: OutputSettings,
    pub hotkeys: Vec<HotkeyBinding>,
    pub annotations: AnnotationSettings,
    pub ui: UiSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            version: 1,
            capture: CaptureSettings {
                default_mode: "region".to_string(),
                show_cursor: false,
                capture_delay: 0,
                play_sound_on_capture: false,
            },
            after_capture: "open_editor".to_string(),
            also_copy_after_annotating: true,
            output: OutputSettings {
                save_path: dirs::desktop_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                filename_pattern: "SnapInk {YYYY-MM-DD} at {HH.mm.ss}".to_string(),
                format: "png".to_string(),
                jpeg_quality: 85,
                retina_clipboard: true,
            },
            hotkeys: vec![
                HotkeyBinding { action: "capture_area".to_string(), shortcut: "CommandOrControl+Shift+4".to_string() },
                HotkeyBinding { action: "capture_screen".to_string(), shortcut: "CommandOrControl+Shift+3".to_string() },
                HotkeyBinding { action: "capture_window".to_string(), shortcut: "CommandOrControl+Shift+5".to_string() },
                HotkeyBinding { action: "capture_scrolling".to_string(), shortcut: "CommandOrControl+Shift+6".to_string() },
                HotkeyBinding { action: "capture_ocr".to_string(), shortcut: "CommandOrControl+Shift+7".to_string() },
                HotkeyBinding { action: "repeat_last".to_string(), shortcut: "CommandOrControl+Shift+R".to_string() },
            ],
            annotations: AnnotationSettings {
                default_color: "#FF3B30".to_string(),
                palette: vec![
                    "#FF3B30".to_string(),
                    "#FF9500".to_string(),
                    "#FFCC00".to_string(),
                    "#34C759".to_string(),
                    "#007AFF".to_string(),
                    "#AF52DE".to_string(),
                    "#1D1D1F".to_string(),
                    "#FFFFFF".to_string(),
                ],
            },
            ui: UiSettings {
                theme: "system".to_string(),
                show_menu_bar_icon: true,
                launch_at_login: false,
            },
        }
    }
}
