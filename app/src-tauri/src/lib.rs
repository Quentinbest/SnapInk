mod capture;
mod capture_store;
mod clipboard;
mod export;
mod pin;
mod scroll;
mod settings;
mod stitch;
mod types;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use capture_store::{CaptureStore, ScrollCaptureStore};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};

fn open_editor_window(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("editor") {
        let _ = win.show();
        let _ = win.set_focus();
        // The editor is already mounted (we hide rather than destroy it).
        // onMount won't re-fire, so emit an event to trigger the frontend
        // to consume the new capture result from the store.
        let _ = win.emit("new-capture-ready", ());
        return;
    }
    let result = WebviewWindowBuilder::new(app, "editor", WebviewUrl::App("/".into()))
        .title("SnapInk")
        .inner_size(1100.0, 700.0)
        .min_inner_size(800.0, 500.0)
        .resizable(true)
        .decorations(true)
        .build();
    // Hide the editor window on close instead of destroying it so the app
    // keeps running as a menu bar agent (LSUIElement = true).
    if let Ok(win) = result {
        let w = win.clone();
        win.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = w.hide();
            }
        });
    }
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
    let app = app.clone();
    let mode = mode.to_string();
    // Run in a background thread so the tray menu event handler returns
    // immediately. The 200ms delay lets macOS fully dismiss the tray menu
    // popup before we call CGWindowListCreateImage — without this delay the
    // compositor is still in a transitional state and returns only the
    // desktop wallpaper instead of all on-screen windows.
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(350));

        // Take the background screenshot BEFORE the capture window opens so
        // the overlay never appears in the frozen background image.
        // Scrolling mode also needs a background so the overlay isn't a white screen.
        if mode == "area" || mode == "window" || mode == "screen" || mode == "scrolling" {
            if let Some(store) = app.try_state::<CaptureStore>() {
                match capture::take_screenshot_sync() {
                    Ok(data) => {
                        *store.background.lock().unwrap() = Some(data);
                        *store.result.lock().unwrap() = None;
                    }
                    Err(e) => eprintln!("pre-capture screenshot failed: {}", e),
                }
            }
        }

        // For full-screen mode the result is already stored; open editor directly.
        if mode == "screen" {
            let result = app
                .try_state::<CaptureStore>()
                .and_then(|s| s.background.lock().unwrap().clone());
            if let Some(data) = result {
                if let Some(store) = app.try_state::<CaptureStore>() {
                    *store.result.lock().unwrap() = Some(data);
                }
                open_editor_window(&app);
                return;
            }
        }

        if let Some(win) = app.get_webview_window("capture") {
            let _ = win.close();
        }

        let url = format!("/capture?mode={}", mode);
        if let Some(monitor) = app.primary_monitor().ok().flatten() {
            let size = monitor.size();
            let pos = monitor.position();
            // Start hidden; the frontend calls show() once the screenshot
            // background is rendered — prevents the black/white flash.
            let _ = WebviewWindowBuilder::new(&app, "capture", WebviewUrl::App(url.into()))
                .title("SnapInk Capture")
                .inner_size(size.width as f64, size.height as f64)
                .position(pos.x as f64, pos.y as f64)
                .decorations(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .visible(false)
                .build();
        }
    });
}

/// Open the scroll capture control window (small floating pill).
/// `control_x/y` are logical screen coordinates for the window position.
fn open_scroll_control_window(app: &tauri::AppHandle, control_x: f64, control_y: f64) {
    // Close any existing instance first.
    if let Some(win) = app.get_webview_window("scroll-control") {
        let _ = win.close();
    }

    let url = format!("/scroll-control?cx={}&cy={}", control_x as i32, control_y as i32);
    let _ = WebviewWindowBuilder::new(app, "scroll-control", WebviewUrl::App(url.into()))
        .title("Scroll Capture")
        .inner_size(300.0, 68.0)
        .position(control_x, control_y)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .build();
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

/// Called by the capture overlay when the user clicks "Start Scrolling Capture".
/// Stores the physical-pixel region in `ScrollCaptureStore`, closes the overlay,
/// and opens the scroll control window positioned below the selected region.
#[tauri::command]
fn start_scroll_capture_cmd(
    app: tauri::AppHandle,
    scroll_store: tauri::State<'_, ScrollCaptureStore>,
    // Physical pixel coords for xcap screen capture.
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    // Logical (CSS) pixel coords for window positioning.
    logical_x: f64,
    logical_y: f64,
    logical_width: f64,
    logical_height: f64,
) -> Result<(), String> {
    // Store the capture region and reset any previous frames.
    *scroll_store.region.lock().unwrap() = Some(types::CaptureRegion { x, y, width, height });
    scroll_store.frames.lock().unwrap().clear();

    // Close the full-screen capture overlay.
    if let Some(win) = app.get_webview_window("capture") {
        let _ = win.close();
    }

    // Position the control window below the selected region, centered horizontally.
    let control_x = logical_x + logical_width / 2.0 - 120.0;
    let control_y = logical_y + logical_height + 16.0;
    open_scroll_control_window(&app, control_x, control_y);

    Ok(())
}

/// Start the auto-scroll capture loop.
/// Posts CGEvent scroll events at a fixed interval, captures frames, and
/// emits `scroll-frame-added` / `scroll-capture-done` to the frontend.
/// Space key is registered as a global shortcut to stop the loop.
#[tauri::command]
fn start_auto_scroll_capture_cmd(
    app: tauri::AppHandle,
    scroll_stop: tauri::State<'_, scroll::ScrollStop>,
) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    // Reset stop flag for this new capture.
    scroll_stop.0.store(false, Ordering::Relaxed);
    let stop = scroll_stop.0.clone();

    // Register Space to stop the loop.
    let stop_for_shortcut = stop.clone();
    app.global_shortcut()
        .on_shortcut("Space", move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                stop_for_shortcut.store(true, Ordering::Relaxed);
            }
        })
        .map_err(|e| e.to_string())?;

    std::thread::spawn(move || {
        scroll::run_capture_loop(app.clone(), stop, 300);
        // Unregister Space after the loop exits.
        let _ = app.global_shortcut().unregister("Space");
    });

    Ok(())
}

/// Signal the auto-scroll loop to stop (called by the Stop button in the UI).
#[tauri::command]
fn stop_scroll_capture_cmd(scroll_stop: tauri::State<'_, scroll::ScrollStop>) {
    use std::sync::atomic::Ordering;
    scroll_stop.0.store(true, Ordering::Relaxed);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(pin::PinStore(std::sync::Mutex::new(
            std::collections::HashMap::new(),
        )))
        .manage(CaptureStore::new())
        .manage(ScrollCaptureStore::new())
        .manage(scroll::ScrollStop(Arc::new(AtomicBool::new(false))))
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
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "capture_area" => open_capture_window(app, "area"),
                    "capture_screen" => open_capture_window(app, "screen"),
                    "capture_window" => open_capture_window(app, "window"),
                    "capture_scrolling" => open_capture_window(app, "scrolling"),
                    "ocr" => open_capture_window(app, "ocr"),
                    "repeat_last" => open_capture_window(app, "repeat"),
                    "settings" => open_settings_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {}
                })
                .build(app)?;

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
            capture_store::get_capture_background,
            capture_store::crop_and_store,
            capture_store::consume_capture_result,
            capture_store::store_capture_result,
            capture_store::scroll_capture_add_frame,
            capture_store::scroll_capture_reset,
            capture_store::stitch_scroll_frames,
            export::export_to_file,
            export::expand_filename,
            export::get_default_save_path,
            settings::get_settings,
            settings::save_settings,
            open_capture_cmd,
            open_editor_cmd,
            open_settings_cmd,
            start_scroll_capture_cmd,
            start_auto_scroll_capture_cmd,
            stop_scroll_capture_cmd,
            pin::pin_image,
            pin::get_pin_image,
            pin::remove_pin_image,
            clipboard::read_clipboard_image,
        ])
        // Keep the app alive as a menu bar agent even when all windows are closed.
        // Without this handler, Tauri exits as soon as the last window is destroyed.
        // In Tauri 2 the event handler is passed to App::run(), not Builder::run().
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
