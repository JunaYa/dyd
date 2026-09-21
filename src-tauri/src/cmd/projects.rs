use crate::projects::{ImportReport, Library, Project, ProjectStore};

fn store(app: &tauri::AppHandle) -> anyhow::Result<ProjectStore> {
    let root = crate::common::get_images_dir(app, "projects".into()).map_err(anyhow::Error::msg)?;
    ProjectStore::open(root)
}

#[tauri::command]
pub async fn list_projects(app: tauri::AppHandle) -> Result<Library, String> {
    tauri::async_runtime::spawn_blocking(move || store(&app)?.list())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub async fn get_project(app: tauri::AppHandle, id: String) -> Result<Project, String> {
    tauri::async_runtime::spawn_blocking(move || store(&app)?.get(&id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub async fn import_legacy_projects(app: tauri::AppHandle) -> Result<ImportReport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let root =
            crate::common::get_images_dir(&app, String::new()).map_err(anyhow::Error::msg)?;
        store(&app)?.import_legacy(&root.join("images"))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub async fn get_project_png(app: tauri::AppHandle, id: String) -> Result<tauri::ipc::Response, String> {
    let bytes = tauri::async_runtime::spawn_blocking(move || store(&app)?.png(&id))
        .await.map_err(|e| e.to_string())?.map_err(|e| format!("{e:#}"))?;
    Ok(tauri::ipc::Response::new(bytes))
}
