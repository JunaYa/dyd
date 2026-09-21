use tauri::{AppHandle, Manager};

use crate::window;

#[tauri::command]
pub fn show_preview_window(app: AppHandle, path: String) -> Result<String, String> {
    crate::capture::ensure_idle(&app)?;
    window::show_preview_window(&app);
    Ok(path)
}

#[tauri::command]
pub fn update_preview_window(app: AppHandle) -> Result<(), String> {
    crate::capture::ensure_idle(&app)?;
    window::update_preview_window(&app);
    Ok(())
}

#[tauri::command]
pub fn hide_preview_window(app: AppHandle) -> Result<(), String> {
    window::hide_preview_window(&app);
    Ok(())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    crate::capture::ensure_idle(&app)?;
    window::show_main_window(&app);
    Ok(())
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> Result<(), String> {
    window::hide_main_window(&app);
    Ok(())
}

#[tauri::command]
pub fn show_setting_window(app: AppHandle) -> Result<(), String> {
    crate::capture::ensure_idle(&app)?;
    window::show_setting_window(&app);
    Ok(())
}

#[tauri::command]
pub fn hide_setting_window(app: AppHandle) -> Result<(), String> {
    window::hide_setting_window(&app);
    Ok(())
}

#[tauri::command]
pub fn open_workspace_window(app: AppHandle, name: String) -> Result<(), String> {
    window::open_workspace_window(&app, &name)
}

#[tauri::command]
pub fn finish_startup(app: AppHandle) -> Result<(), String> {
    window::open_workspace_window(&app, "main")?;
    if let Some(startup) = app.get_webview_window(crate::constants::STARTUP_WINDOW) {
        startup.hide().map_err(|error| error.to_string())?;
    }
    Ok(())
}
