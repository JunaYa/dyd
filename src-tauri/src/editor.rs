use crate::projects::Project;
use serde::Serialize;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Clone, Default, Serialize)]
pub struct Selection {
    pub revision: u64,
    pub project: Option<Project>,
}

impl Selection {
    fn select(&mut self, project: Project) {
        self.revision += 1;
        self.project = Some(project);
    }
}

#[derive(Default)]
pub struct EditorState(pub Mutex<Selection>);

pub fn open(app: &tauri::AppHandle, project: Project) -> Result<(), String> {
    crate::capture::ensure_idle(app)?;
    let state = app.state::<EditorState>();
    let mut selection = state.0.lock().map_err(|e| e.to_string())?;
    selection.select(project);
    drop(selection);
    crate::window::open_workspace_window(app, "editor")
}

#[tauri::command]
pub fn editor_ready(state: tauri::State<'_, EditorState>) -> Result<Selection, String> {
    state.0.lock().map(|value| value.clone()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_project(app: tauri::AppHandle, id: String) -> Result<(), String> {
    crate::capture::ensure_idle(&app)?;
    let root = crate::common::get_images_dir(&app, "projects".into())?;
    let project = crate::projects::ProjectStore::open(root)
        .and_then(|store| store.get(&id))
        .map_err(|e| format!("{e:#}"))?;
    open(&app, project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_selection_is_versioned_and_latest_project_survives_readiness() {
        let mut selection = Selection::default();
        for id in ["first", "first", "second"] {
            selection.select(Project {
                version: 1, id: id.into(), name: id.into(), created_at: "2026-09-21T00:00:00Z".into(),
                width: 2, height: 2, source_path: None,
            });
        }
        let ready = selection.clone();
        assert_eq!(ready.revision, 3);
        assert_eq!(ready.project.unwrap().id, "second");
        assert_eq!(selection.project.unwrap().id, "second");
    }
}
