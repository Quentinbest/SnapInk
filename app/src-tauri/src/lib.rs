mod capture;
mod clipboard;
mod export;
mod pin;
mod settings;
mod types;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder,
};

fn open_editor_window(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("editor") {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "editor", WebviewUrl::App("/".into()))
        .title("SnapInk")
        .inner_size(1100.0, 700.0)
        .min_inner_size(800.0, 500.0)
        .resizable(true)
        .decorations(true)
        .build();
}

fn open_settings_window(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("/settings".into()))
        .title("SnapInk Settings")
        .inner_size(520.0, 560.0)
        .resizable(false)
        .decorations(true)
        .build();
}

fn open_capture_window(app: &tauri::AppHandle, mode: &str) {
    if let Some(win) = app.get_webview_window("capture") {
        let _ = win.close();
    }
    let url = format!("/capture?mode={}", mode);
    if let Some(monitor) = app.primary_monitor().ok().flatten() {
        let size = monitor.size();
        let pos = monitor.position();
        let _ = WebviewWindowBuilder::new(app, "capture", WebviewUrl::App(url.into()))
            .title("SnapInk Capture")
            .inner_size(size.width as f64, size.height as f64)
            .position(pos.x as f64, pos.y as f64)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .build();
    }
}

#[tauri::command]
fn open_capture_cmd(app: tauri::AppHandle, mode: String) {
    open_capture_window(&app, &mode);
}

#[tauri::command]
fn open_editor_cmd(app: tauri::AppHandle) {
    open_editor_window(&app);
}

#[tauri::command]
fn open_settings_cmd(app: tauri::AppHandle) {
    open_settings_window(&app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(pin::PinStore(std::sync::Mutex::new(std::collections::HashMap::new())))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();

            let capture_area = MenuItem::with_id(app, "capture_area", "Capture Area", true, Some("Ctrl+Shift+4"))?;
            let capture_screen = MenuItem::with_id(app, "capture_screen", "Capture Screen", true, Some("Ctrl+Shift+3"))?;
            let capture_window = MenuItem::with_id(app, "capture_window", "Capture Window", true, Some("Ctrl+Shift+5"))?;
            let capture_scrolling = MenuItem::with_id(app, "capture_scrolling", "Scrolling Capture", true, Some("Ctrl+Shift+6"))?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let ocr = MenuItem::with_id(app, "ocr", "Recognize Text (OCR)", true, Some("Ctrl+Shift+7"))?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let repeat_last = MenuItem::with_id(app, "repeat_last", "Repeat Last Capture", true, Some("Ctrl+Shift+R"))?;
            let sep3 = PredefinedMenuItem::separator(app)?;
            let settings = MenuItem::with_id(app, "settings", "Settings…", true, Some("CmdOrCtrl+,"))?;
            let sep4 = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Quit SnapInk", true, Some("CmdOrCtrl+Q"))?;

            let more_submenu = Submenu::with_id_and_items(
                app,
                "more",
                "More",
                true,
                &[&repeat_last],
            )?;

            let menu = Menu::with_items(app, &[
                &capture_screen,
                &capture_area,
                &capture_window,
                &capture_scrolling,
                &sep1,
                &ocr,
                &sep2,
                &more_submenu,
                &sep3,
                &settings,
                &sep4,
                &quit,
            ])?;

            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("failed to load tray icon");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "capture_area" => open_capture_window(app, "area"),
                        "capture_screen" => open_capture_window(app, "screen"),
                        "capture_window" => open_capture_window(app, "window"),
                        "capture_scrolling" => open_capture_window(app, "scrolling"),
                        "ocr" => open_capture_window(app, "ocr"),
                        "repeat_last" => open_capture_window(app, "repeat"),
                        "settings" => open_settings_window(app),
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                    }
                })
                .build(app)?;

            // Register global shortcuts
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

            let h1 = handle.clone();
            app.global_shortcut().on_shortcut("Ctrl+Shift+4", move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    open_capture_window(&h1, "area");
                }
            })?;

            let h2 = handle.clone();
            app.global_shortcut().on_shortcut("Ctrl+Shift+3", move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    open_capture_window(&h2, "screen");
                }
            })?;

            let h3 = handle.clone();
            app.global_shortcut().on_shortcut("Ctrl+Shift+5", move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    open_capture_window(&h3, "window");
                }
            })?;

            let h4 = handle.clone();
            app.global_shortcut().on_shortcut("Ctrl+Shift+6", move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    open_capture_window(&h4, "scrolling");
                }
            })?;

            let h5 = handle.clone();
            app.global_shortcut().on_shortcut("Ctrl+Shift+7", move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    open_capture_window(&h5, "ocr");
                }
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            capture::get_monitors,
            capture::get_windows,
            capture::capture_fullscreen,
            capture::capture_region,
            capture::capture_window_by_id,
            export::export_to_file,
            export::expand_filename,
            export::get_default_save_path,
            settings::get_settings,
            settings::save_settings,
            open_capture_cmd,
            open_editor_cmd,
            open_settings_cmd,
            pin::pin_image,
            pin::get_pin_image,
            pin::remove_pin_image,
            clipboard::read_clipboard_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
