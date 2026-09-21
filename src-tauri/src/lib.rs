use serde_json::json;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

mod capture;
mod capture_task;
mod cmd;
mod common;
mod constants;
mod global_shortcut;
mod menu;
mod platform;
mod settings;
mod window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(capture_task::CaptureTasks::default())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            #[cfg(desktop)]
            configure_autostart(app)?;

            #[cfg(desktop)]
            if let Err(error) = global_shortcut::register_global_shortcut(app) {
                tracing::warn!(%error, "Could not register capture shortcuts");
            }

            // app.set_activation_policy(ActivationPolicy::Accessory);

            menu::create_tray(app)?;

            let store = app.store("settings.json")?;
            // StoreBuilder ignores load errors; reject unreadable settings before any writes.
            if app
                .path()
                .app_data_dir()?
                .join("settings.json")
                .try_exists()?
            {
                store.reload()?;
            }
            let saved_path = store.get("screenshot_path");
            let first_run = store.get("first_run");
            let needs_startup = settings::needs_startup(first_run.as_ref(), saved_path.as_ref())
                .map_err(std::io::Error::other)?;
            if needs_startup {
                store.set("first_run", json!({ "value": false }));
                store.save()?;
            }
            if saved_path.is_none() {
                let app_local_data = app.path().app_local_data_dir()?;
                store.set("screenshot_path", json!({ "value": app_local_data }));
                store.save()?;
            }

            if needs_startup {
                window::show_startup_window(&app.handle());
            } else {
                window::show_main_window(&app.handle());
            }
            if first_run.as_ref() != Some(&json!({ "value": true })) {
                store.set("first_run", json!({ "value": true }));
                store.save()?;
            }

            Ok(())
        })
        .menu(menu::get_app_menu)
        .on_menu_event(menu::handle_app_menu_event)
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(global_shortcut::tauri_plugin_global_shortcut())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            cmd::greet,
            cmd::start_capture,
            cmd::request_capture_permission,
            cmd::get_capture_task,
            cmd::open_workspace_window,
            cmd::finish_startup,
            cmd::show_preview_window,
            cmd::hide_preview_window,
            cmd::update_preview_window,
            cmd::show_main_window,
            cmd::hide_main_window,
            cmd::show_setting_window,
            cmd::hide_setting_window,
            cmd::copy_image_to_clipboard,
            cmd::get_image_base64,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(desktop)]
fn configure_autostart(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_autostart::MacosLauncher;
    use tauri_plugin_autostart::ManagerExt;

    app.handle().plugin(tauri_plugin_autostart::init(
        MacosLauncher::LaunchAgent,
        None,
    ))?;
    match app.autolaunch().is_enabled() {
        Ok(enabled) => tracing::info!(enabled, "Current autostart preference"),
        Err(error) => tracing::warn!(%error, "Could not read autostart preference"),
    }
    Ok(())
}
