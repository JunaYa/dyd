use crate::platform;

#[tauri::command]
pub async fn capture_screen(app_handle: tauri::AppHandle, path: String) -> Result<String, String> {
    let filename = platform::capture_screen(&app_handle, path).await?;
    Ok(filename)
}

#[tauri::command]
pub async fn capture_select(app_handle: tauri::AppHandle, path: String) -> Result<String, String> {
    let filename = platform::capture_select(&app_handle, path).await?;
    Ok(filename)
}

#[tauri::command]
pub async fn capture_window(app_handle: tauri::AppHandle, path: String) -> Result<String, String> {
    let filename = platform::capture_window(&app_handle, path).await?;
    Ok(filename)
}

#[tauri::command]
pub fn open_screen_capture_preferences() {
    platform::open_screen_capture_preferences();
}

#[tauri::command]
pub fn check_accessibility_permissions() -> bool {
    platform::check_accessibility_permissions()
}
