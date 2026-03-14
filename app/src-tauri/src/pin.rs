use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{State, WebviewUrl, WebviewWindowBuilder};

pub struct PinStore(pub Mutex<HashMap<String, String>>);

#[tauri::command]
pub fn pin_image(
    app: tauri::AppHandle,
    store: State<'_, PinStore>,
    image_data: String,
) -> Result<String, String> {
    let win_id = format!(
        "pin-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    store.0.lock().unwrap().insert(win_id.clone(), image_data);

    let url = format!("/pin?id={}", win_id);
    WebviewWindowBuilder::new(&app, &win_id, WebviewUrl::App(url.into()))
        .title("SnapInk Pin")
        .inner_size(400.0, 300.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(win_id)
}

#[tauri::command]
pub fn get_pin_image(
    store: State<'_, PinStore>,
    id: String,
) -> Option<String> {
    store.0.lock().unwrap().get(&id).cloned()
}

#[tauri::command]
pub fn remove_pin_image(store: State<'_, PinStore>, id: String) {
    store.0.lock().unwrap().remove(&id);
}
