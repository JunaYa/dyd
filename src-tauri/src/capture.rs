use crate::{
    capture_task::{CaptureKind, CaptureState, CaptureTask, CaptureTasks},
    window,
};
use tauri::{Emitter, Manager};

pub fn start(app: &tauri::AppHandle, kind: CaptureKind) -> Result<CaptureTask, String> {
    let (task, started) = app
        .state::<CaptureTasks>()
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .begin(kind);
    if !started {
        return Ok(task);
    }
    publish(app, &task);
    let app = app.clone();
    let id = task.id.clone();
    tauri::async_runtime::spawn(async move {
        let worker_app = app.clone();
        let worker_id = id.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            let directory = crate::common::get_images_dir(&worker_app, "images".into())?;
            let output = directory.join(format!("screenshot_{worker_id}.png"));
            #[cfg(target_os = "macos")]
            {
                crate::platform::capture(kind, &output)
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = (kind, output);
                Err("Screenshot capture is not supported on this platform yet".into())
            }
        })
        .await;
        let state = match result {
            Ok(Ok(state)) => state,
            Ok(Err(error)) => CaptureState::Failed { error },
            Err(error) => CaptureState::Failed {
                error: format!("Capture worker failed: {error}"),
            },
        };
        let completed = app
            .state::<CaptureTasks>()
            .0
            .lock()
            .map_err(|e| e.to_string())
            .and_then(|mut log| log.finish(&id, state));
        match completed {
            Ok(task) => {
                publish(&app, &task);
                let result_app = app.clone();
                if let Err(error) = app.run_on_main_thread(move || {
                    let tasks = result_app.state::<CaptureTasks>();
                    let latest = tasks.0.lock().map(|log| log.get(None));
                    match latest {
                        Ok(Some(latest)) if latest.id == task.id => {}
                        Ok(_) => return,
                        Err(error) => {
                            tracing::error!(%error, "Could not query latest capture task");
                            return;
                        }
                    }
                    match task.state {
                        CaptureState::Ready { .. } => {
                            window::hide_main_window(&result_app);
                            window::show_preview_window(&result_app);
                        }
                        CaptureState::Failed { .. } => {
                            if let Err(error) = window::open_workspace_window(&result_app, "editor")
                            {
                                tracing::error!(%error, "Could not show capture error");
                            }
                        }
                        _ => {}
                    }
                }) {
                    tracing::error!(%error, "Could not present capture result");
                }
            }
            Err(error) => tracing::error!(%error, "Could not complete capture task"),
        }
    });
    Ok(task)
}

fn publish(app: &tauri::AppHandle, task: &CaptureTask) {
    if let Err(error) = app.emit("capture-task-changed", task) {
        tracing::warn!(%error, "Could not emit capture task; result remains queryable");
    }
}

pub fn start_from_menu(app: &tauri::AppHandle, kind: CaptureKind) {
    if let Err(error) = start(app, kind) {
        tracing::error!(%error, "Could not start capture task");
    }
}
