use crate::capture_task::{CaptureKind, CaptureTask, CaptureTasks};

#[tauri::command]
pub fn request_capture_permission(app: tauri::AppHandle) -> Result<bool, String> {
    crate::capture::ensure_idle(&app)?;
    #[cfg(target_os = "macos")]
    {
        Ok(crate::platform::request_capture_permission())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Screen capture permission is not supported on this platform yet".into())
    }
}

#[tauri::command]
pub fn start_capture(app: tauri::AppHandle, kind: CaptureKind) -> Result<CaptureTask, String> {
    crate::capture::start(&app, kind)
}

#[tauri::command]
pub fn get_capture_task(
    tasks: tauri::State<'_, CaptureTasks>,
    id: Option<String>,
) -> Result<Option<CaptureTask>, String> {
    Ok(tasks
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .get(id.as_deref()))
}
