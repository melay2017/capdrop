// CapDrop - Library entry (Tauri 2 pattern)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod config;
mod editor;
mod hotkey;
mod storage;

use base64::Engine;
use capture::Screenshot;
use config::AppConfig;
use storage::SaveResult;
use tauri::{Emitter, Listener, Manager};

/// Capture fullscreen screenshot, return base64 PNG
#[tauri::command]
fn capture_fullscreen() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        capture::macos::capture_screen()
    }
    #[cfg(target_os = "windows")]
    {
        Err("Windows capture not yet implemented".into())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("Unsupported platform".into())
    }
}

/// Capture a region screenshot, return base64 PNG
#[tauri::command]
fn capture_region(x: u32, y: u32, w: u32, h: u32) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        capture::macos::capture_region_b64(x, y, w, h)
    }
    #[cfg(target_os = "windows")]
    {
        let _ = (x, y, w, h);
        Err("Windows capture not yet implemented".into())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (x, y, w, h);
        Err("Unsupported platform".into())
    }
}

/// Save a base64-encoded screenshot to all configured targets
#[tauri::command]
fn save_screenshot(image_b64: String, state: tauri::State<'_, std::sync::Mutex<AppConfig>>) -> Result<Vec<SaveResult>, String> {
    let png_data = base64::engine::general_purpose::STANDARD
        .decode(&image_b64)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    let img = image::load_from_memory(&png_data)
        .map_err(|e| format!("Image decode error: {}", e))?;

    let screenshot = Screenshot {
        data: png_data,
        width: img.width(),
        height: img.height(),
    };

    let config = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(storage::save_to_all_targets(&screenshot, &config))
}

/// Get current config
#[tauri::command]
fn get_config(state: tauri::State<'_, std::sync::Mutex<AppConfig>>) -> Result<AppConfig, String> {
    let config = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(config.clone())
}

/// Update config
#[tauri::command]
fn update_config(new_config: AppConfig, state: tauri::State<'_, std::sync::Mutex<AppConfig>>) -> Result<(), String> {
    let mut config = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    config.update(new_config)
}

/// Save PNG data directly to a file path
#[tauri::command]
fn save_to_file(image_b64: String, file_path: String) -> Result<String, String> {
    let png_data = base64::engine::general_purpose::STANDARD
        .decode(&image_b64)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    let path = std::path::PathBuf::from(&file_path);
    storage::local::save_png_to_path(&png_data, &path)?;
    Ok(file_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load().unwrap_or_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(std::sync::Mutex::new(config))
        .invoke_handler(tauri::generate_handler![
            capture_fullscreen,
            capture_region,
            save_screenshot,
            get_config,
            update_config,
            save_to_file,
        ])
        .setup(|app| {
            // Register global hotkey
            let config_state = app.state::<std::sync::Mutex<AppConfig>>();
            let hotkey = {
                let cfg = config_state.lock().unwrap();
                cfg.hotkey.clone()
            };

            if let Err(e) = hotkey::register_hotkey(&app.handle().clone(), &hotkey) {
                eprintln!("Warning: failed to register hotkey '{}': {}", hotkey, e);
            }

            // Listen for screenshot hotkey events from frontend
            let handle = app.handle().clone();
            app.listen("hotkey:screenshot", move |_event| {
                let _ = handle.emit("screenshot:triggered", ());
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running CapDrop");
}
