use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, WebviewWindow};
use tauri::{TitleBarStyle, WebviewUrl, WebviewWindowBuilder};
use tracing::info;

use crate::constants::{MAIN_WINDOW, PREVIEW_WINDOW, SETTING_WINDOW, STARTUP_WINDOW};
use crate::platform;

pub fn find_monitor(window: &WebviewWindow) -> Option<Monitor> {
    if let Ok(Some(mon)) = window.primary_monitor() {
        Some(mon)
    } else if let Ok(Some(mon)) = window.current_monitor() {
        Some(mon)
    } else if let Ok(mut monitors) = window.available_monitors() {
        if monitors.is_empty() {
            None
        } else {
            monitors.pop()
        }
    } else {
        None
    }
}

pub fn center_position(window: &WebviewWindow) {
    let window_size = match window.inner_size() {
        Ok(size) => size,
        // Nothing to do if the window is not created yet.
        Err(_) => return,
    };

    if let Some(monitor) = find_monitor(window) {
        let screen_position = monitor.position();
        let screen_size = monitor.size();

        let y = (120f64 * monitor.scale_factor()) as i32;
        let x =
            screen_position.x + ((screen_size.width as i32 / 2) - (window_size.width as i32 / 2));
        let new_position = PhysicalPosition { x, y };

        let _ = window.set_position(tauri::Position::Physical(new_position));
    } else {
        info!("Unable to detect any monitors.");
    }
}

pub fn bottom_right_position(window: &WebviewWindow) {
    let window_size = match window.inner_size() {
        Ok(size) => size,
        // Nothing to do if the window is not created yet.
        Err(_) => return,
    };

    if let Some(monitor) = find_monitor(window) {
        let screen_size = monitor.size();

        let y = (screen_size.height as f64
            - monitor.scale_factor()
            - window_size.height as f64
            - 128.0) as i32;
        let x =
            (screen_size.width as f64 - monitor.scale_factor() - window_size.width as f64 - 128.0)
                as i32;

        let new_position = PhysicalPosition { x, y };

        let _ = window.set_position(tauri::Position::Physical(new_position));
    } else {
        info!("Unable to detect any monitors.");
    }
}

pub fn get_main_window(app: &AppHandle) -> WebviewWindow {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        window
    } else {
        let win_builder = WebviewWindowBuilder::new(app, MAIN_WINDOW, WebviewUrl::App("/".into()))
            .title("dyd")
            .transparent(true)
            .skip_taskbar(true)
            .maximized(true)
            .decorations(false)
            .inner_size(800.0, 600.0);

        #[cfg(target_os = "macos")]
        let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

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
                    0.3,
                );
                ns_window.setBackgroundColor_(bg_color);
            }
            window
        }
    }
}

pub fn get_setting_window(app: &AppHandle) -> WebviewWindow {
    workspace_window(app, SETTING_WINDOW, "/setting", "设置", 680.0, 620.0)
        .expect("Unable to build settings window")
}

fn workspace_window(
    app: &AppHandle,
    label: &str,
    route: &str,
    title: &str,
    width: f64,
    height: f64,
) -> tauri::Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(label) {
        return Ok(window);
    }
    WebviewWindowBuilder::new(app, label, WebviewUrl::App(route.into()))
        .title(title)
        .inner_size(width, height)
        .min_inner_size(360.0, 360.0)
        .build()
}

pub fn open_workspace_window(app: &AppHandle, name: &str) -> Result<(), String> {
    let window = match name {
        "main" => {
            show_main_window(app);
            return get_main_window(app)
                .set_focus()
                .map_err(|error| error.to_string());
        }
        "setting" => workspace_window(app, SETTING_WINDOW, "/setting", "设置", 680.0, 620.0),
        "startup" => workspace_window(
            app,
            STARTUP_WINDOW,
            "/startup",
            "欢迎使用 DYD",
            600.0,
            640.0,
        ),
        "editor" => workspace_window(app, "editor", "/editor", "图片编辑器", 1000.0, 720.0),
        "history" => workspace_window(app, "history", "/history", "历史记录", 800.0, 640.0),
        _ => return Err("Unknown workspace window".to_string()),
    }
    .map_err(|error| error.to_string())?;
    window.unminimize().map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn get_preview_window(app: &AppHandle) -> WebviewWindow {
    if let Some(window) = app.get_webview_window(PREVIEW_WINDOW) {
        window
    } else {
        let window =
            WebviewWindowBuilder::new(app, PREVIEW_WINDOW, WebviewUrl::App("/preview".into()))
                .title("preview")
                .decorations(false)
                .transparent(true)
                .visible(true)
                .skip_taskbar(true)
                .shadow(false)
                .resizable(false)
                .inner_size(240.0, 240.0);

        let window = window.build().expect("Unable to build startup window");
        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::{NSColor, NSWindow};
            use cocoa::base::{id, nil};

            let ns_window = window.ns_window().unwrap() as id;
            unsafe {
                // macOS: Handle multiple spaces correctly
                ns_window.setCollectionBehavior_(cocoa::appkit::NSWindowCollectionBehavior::NSWindowCollectionBehaviorMoveToActiveSpace);

                let bg_color = NSColor::colorWithRed_green_blue_alpha_(
                    nil,
                    33.0 / 255.0,
                    54.0 / 255.0,
                    201.0 / 255.0,
                    0.0,
                );
                ns_window.setBackgroundColor_(bg_color);
            }
        }

        window
    }
}

pub fn get_startup_window(app: &AppHandle) -> WebviewWindow {
    workspace_window(
        app,
        STARTUP_WINDOW,
        "/startup",
        "欢迎使用 DYD",
        600.0,
        640.0,
    )
    .expect("Unable to build startup window")
}

pub fn show_preview_window(app: &AppHandle) -> WebviewWindow {
    let window = get_preview_window(app);
    platform::show_preview_window(&window);
    window
}

pub fn update_preview_window(app: &AppHandle) -> WebviewWindow {
    let window = get_preview_window(app);
    platform::update_preview_window(&window);
    window
}

pub fn hide_preview_window(app: &AppHandle) {
    if let Some(preview_window) = app.get_webview_window(PREVIEW_WINDOW) {
        if preview_window.is_visible().unwrap_or_default() {
            platform::hide_preview_window(&preview_window);
        }
    }
}

pub fn show_main_window(app: &AppHandle) {
    let window = get_main_window(app);
    platform::show_main_window(&window);
}

pub fn hide_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window(MAIN_WINDOW) {
        if main_window.is_visible().unwrap_or_default() {
            platform::hide_main_window(&main_window);
        }
    }
}

pub fn show_setting_window(app: &AppHandle) {
    let window = get_setting_window(app);
    platform::show_setting_window(&window);
}

pub fn hide_setting_window(app: &AppHandle) {
    if let Some(setting_window) = app.get_webview_window(SETTING_WINDOW) {
        if setting_window.is_visible().unwrap_or_default() {
            platform::hide_setting_window(&setting_window);
        }
    }
}

pub fn show_startup_window(app: &AppHandle) {
    let window = get_startup_window(app);
    platform::show_startup_window(&window);
}

pub fn hide_startup_window(app: &AppHandle) {
    if let Some(startup_window) = app.get_webview_window(STARTUP_WINDOW) {
        if startup_window.is_visible().unwrap_or_default() {
            platform::hide_startup_window(&startup_window);
        }
    }
}
