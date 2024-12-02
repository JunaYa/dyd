use std::thread::sleep;

use tauri::{TitleBarStyle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = get_main_window(app);
            tauri::async_runtime::spawn(async move {
                sleep(std::time::Duration::from_millis(3000));
                let _ = window.show();
                let _ = window.set_focus();
            });
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn get_main_window(app: &tauri::App) -> WebviewWindow {
    let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("DYD")
        .title_bar_style(TitleBarStyle::Transparent)
        .fullscreen(true)
        .decorations(false)
        // .transparent(true)
        .visible(false)
        .skip_taskbar(true)
        .shadow(true)
        .resizable(false)
        .fullscreen(false)
        .maximized(true)
        .visible(false)
        .inner_size(800.0, 600.0);

    let window = win_builder.build().unwrap();

    // set background color only when building for macOS
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSColor, NSWindow};
        use cocoa::base::{id, nil};

        let ns_window = window.ns_window().unwrap() as id;
        unsafe {
            let bg_color = NSColor::colorWithRed_green_blue_alpha_(
                nil,
                33.0 / 255.0,
                54.0 / 255.0,
                201.0 / 255.0,
                0.1,
            );
            ns_window.setBackgroundColor_(bg_color);
        }
        window
    }
}
