use std::str::FromStr;
use tauri::plugin::TauriPlugin;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tracing::info;

use crate::{capture, capture_task::CaptureKind, window};

const DEFUALT_HOTKEY_A: &str = "CmdOrCtrl+Shift+A";
const DEFUALT_HOTKEY_S: &str = "CmdOrCtrl+Shift+S";
const DEFUALT_HOTKEY_W: &str = "CmdOrCtrl+Shift+W";
const DEFUALT_HOTKEY_E: &str = "CmdOrCtrl+Shift+E";

pub fn register_global_shortcut(app: &tauri::App) -> anyhow::Result<()> {
    info!("Registering global shortcuts");
    let shortcuts = app.global_shortcut();
    if let Err(error) = shortcuts.unregister_all() {
        info!("Unable to unregister shortcuts {}", error.to_string());
    }

    // capture_screen: ctrl + shift + A
    let shift_ctrl_a_shortcut = Shortcut::from_str(DEFUALT_HOTKEY_A)?;
    // capture_select: ctrl + shift + S
    let shift_ctrl_s_shortcut = Shortcut::from_str(DEFUALT_HOTKEY_S)?;
    // capture_window: ctrl + shift + W
    let shift_ctrl_w_shortcut = Shortcut::from_str(DEFUALT_HOTKEY_W)?;
    // exit: ctrl + shift + E
    let shift_ctrl_e_shortcut = Shortcut::from_str(DEFUALT_HOTKEY_E)?;

    if !shortcuts.is_registered(shift_ctrl_a_shortcut) {
        app.global_shortcut().register(shift_ctrl_a_shortcut)?;
    }

    if !shortcuts.is_registered(shift_ctrl_s_shortcut) {
        app.global_shortcut().register(shift_ctrl_s_shortcut)?;
    }

    if !shortcuts.is_registered(shift_ctrl_w_shortcut) {
        app.global_shortcut().register(shift_ctrl_w_shortcut)?;
    }

    if !shortcuts.is_registered(shift_ctrl_e_shortcut) {
        app.global_shortcut().register(shift_ctrl_e_shortcut)?;
    }

    Ok(())
}

pub fn tauri_plugin_global_shortcut() -> TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            if shortcut.id == Shortcut::from_str(DEFUALT_HOTKEY_A).unwrap().id {
                capture::start_from_menu(app, CaptureKind::Screen);
            } else if shortcut.id == Shortcut::from_str(DEFUALT_HOTKEY_S).unwrap().id {
                capture::start_from_menu(app, CaptureKind::Select);
            } else if shortcut.id == Shortcut::from_str(DEFUALT_HOTKEY_W).unwrap().id {
                capture::start_from_menu(app, CaptureKind::Window);
            } else if shortcut.id == Shortcut::from_str(DEFUALT_HOTKEY_E).unwrap().id {
                window::show_main_window(app);
            }
        })
        .build()
}
