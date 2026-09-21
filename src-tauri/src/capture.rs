use crate::{
    capture_task::{CaptureKind, CaptureState, CaptureTask, CaptureTasks},
    capture_windows::HiddenWindows,
    window,
};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

pub fn ensure_idle(app: &tauri::AppHandle) -> Result<(), String> {
    let tasks = app.state::<CaptureTasks>();
    let log = tasks.0.lock().map_err(|e| e.to_string())?;
    if log
        .get(None)
        .is_some_and(|task| task.state == CaptureState::Capturing)
    {
        Err("截图进行中，请完成选区或按 Esc 取消后再打开窗口。".into())
    } else {
        Ok(())
    }
}

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
    let capture_app = app.clone();
    let id = task.id.clone();
    if let Err(error) = app.run_on_main_thread(move || {
        let snapshot = match HiddenWindows::prepare(capture_app.webview_windows().into_values()) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                complete(&capture_app, &id, CaptureState::Failed { error });
                return;
            }
        };
        tauri::async_runtime::spawn(async move {
            let worker_app = capture_app.clone();
            let worker_id = id.clone();
            let result = tauri::async_runtime::spawn_blocking(move || {
                // Hide is synchronous on the UI thread, but the compositor needs time to repaint.
                std::thread::sleep(std::time::Duration::from_millis(200));
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
            let restore_app = capture_app.clone();
            let restore_id = id.clone();
            if let Err(error) = capture_app.run_on_main_thread(move || {
                let recovery = snapshot.restore(matches!(state, CaptureState::Ready { .. }));
                let recovery_error = recovery.err();
                // Keep the task active until recovery is done, preventing overlapping window snapshots.
                complete(&restore_app, &restore_id, state);
                if let Some(error) = recovery_error {
                    show_error(&restore_app, &error);
                }
            }) {
                // A stopped UI event loop cannot restore native windows.
                complete(
                    &capture_app,
                    &id,
                    CaptureState::Failed {
                        error: format!("窗口恢复调度失败：{error}"),
                    },
                );
            }
        });
    }) {
        complete(
            app,
            &task.id,
            CaptureState::Failed {
                error: error.to_string(),
            },
        );
        return Err(error.to_string());
    }
    Ok(task)
}

fn complete(app: &tauri::AppHandle, id: &str, state: CaptureState) {
    let result = app
        .state::<CaptureTasks>()
        .0
        .lock()
        .map_err(|e| e.to_string())
        .and_then(|mut log| log.finish(id, state));
    match result {
        Ok(task) => {
            publish(app, &task);
            match task.state {
                CaptureState::Ready { .. } => {
                    window::show_preview_window(app);
                }
                CaptureState::Failed { error } => {
                    tracing::warn!(%error, "Capture failed");
                    if !app.get_webview_window("editor").is_some_and(|w| {
                        w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(true)
                    }) {
                        show_error(app, &error);
                    }
                }
                _ => {}
            }
        }
        Err(error) => tracing::error!(%error, "Could not complete capture task"),
    }
}

fn show_error(app: &tauri::AppHandle, error: &str) {
    tracing::warn!(%error, "Capture window error");
    app.dialog()
        .message(error)
        .title("截图失败或窗口恢复异常")
        .kind(tauri_plugin_dialog::MessageDialogKind::Error)
        .show(|_| {});
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
