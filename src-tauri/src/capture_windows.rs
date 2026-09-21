use tauri::WebviewWindow;

pub trait CaptureWindow: Clone {
    fn label(&self) -> &str;
    fn visible(&self) -> Result<bool, String>;
    fn minimized(&self) -> Result<bool, String>;
    fn focused(&self) -> Result<bool, String>;
    fn hide(&self) -> Result<(), String>;
    fn show(&self) -> Result<(), String>;
    fn focus(&self) -> Result<(), String>;
}

impl CaptureWindow for WebviewWindow {
    fn label(&self) -> &str {
        self.label()
    }
    fn visible(&self) -> Result<bool, String> {
        self.is_visible().map_err(|e| e.to_string())
    }
    fn minimized(&self) -> Result<bool, String> {
        self.is_minimized().map_err(|e| e.to_string())
    }
    fn focused(&self) -> Result<bool, String> {
        self.is_focused().map_err(|e| e.to_string())
    }
    fn hide(&self) -> Result<(), String> {
        self.hide().map_err(|e| e.to_string())
    }
    fn show(&self) -> Result<(), String> {
        self.show().map_err(|e| e.to_string())
    }
    fn focus(&self) -> Result<(), String> {
        self.set_focus().map_err(|e| e.to_string())
    }
}

pub struct HiddenWindows<W> {
    windows: Vec<(W, bool)>,
}

impl<W: CaptureWindow> HiddenWindows<W> {
    pub fn prepare(windows: impl IntoIterator<Item = W>) -> Result<Self, String> {
        let mut visible = Vec::new();
        // Read the complete snapshot before changing any window.
        for window in windows {
            if window.visible()? && !window.minimized()? {
                let focused = window.focused()?;
                visible.push((window, focused));
            }
        }
        let mut hidden = Self {
            windows: Vec::new(),
        };
        for (window, focused) in visible {
            hidden.windows.push((window.clone(), focused));
            if let Err(error) = window.hide() {
                let recovery = hidden.restore(false);
                return Err(format!(
                    "无法隐藏窗口 {}：{error}{}",
                    window.label(),
                    recovery.err().map(|e| format!("；{e}")).unwrap_or_default()
                ));
            }
        }
        Ok(hidden)
    }

    pub fn restore(self, success: bool) -> Result<(), String> {
        let mut errors = Vec::new();
        let mut focused = None;
        for (window, was_focused) in self.windows {
            if success && matches!(window.label(), "main" | "preview") {
                continue;
            }
            match window.show() {
                Ok(()) if was_focused && !success => focused = Some(window),
                Ok(()) => {}
                Err(error) => errors.push(format!("无法恢复窗口 {}：{error}", window.label())),
            }
        }
        if let Some(window) = focused {
            if let Err(error) = window.focus() {
                errors.push(format!("无法恢复焦点：{error}"));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("；"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    #[derive(Clone)]
    struct Fake {
        name: &'static str,
        visible: bool,
        minimized: bool,
        focused: bool,
        fail_hide: bool,
        fail_show: bool,
        fail_read: bool,
        events: Rc<RefCell<Vec<String>>>,
    }
    impl CaptureWindow for Fake {
        fn label(&self) -> &str {
            self.name
        }
        fn visible(&self) -> Result<bool, String> {
            if self.fail_read {
                Err("read failed".into())
            } else {
                Ok(self.visible)
            }
        }
        fn minimized(&self) -> Result<bool, String> {
            Ok(self.minimized)
        }
        fn focused(&self) -> Result<bool, String> {
            Ok(self.focused)
        }
        fn hide(&self) -> Result<(), String> {
            self.events.borrow_mut().push(format!("hide:{}", self.name));
            if self.fail_hide {
                Err("hide failed".into())
            } else {
                Ok(())
            }
        }
        fn show(&self) -> Result<(), String> {
            self.events.borrow_mut().push(format!("show:{}", self.name));
            if self.fail_show {
                Err("show failed".into())
            } else {
                Ok(())
            }
        }
        fn focus(&self) -> Result<(), String> {
            self.events
                .borrow_mut()
                .push(format!("focus:{}", self.name));
            Ok(())
        }
    }
    fn window(name: &'static str, events: &Rc<RefCell<Vec<String>>>) -> Fake {
        Fake {
            name,
            visible: true,
            minimized: false,
            focused: false,
            fail_hide: false,
            fail_show: false,
            fail_read: false,
            events: events.clone(),
        }
    }
    #[test]
    fn cancel_and_failure_restore_only_visible_windows_and_focus_last() {
        let events = Rc::default();
        let mut main = window("main", &events);
        main.focused = true;
        let mut hidden = window("startup", &events);
        hidden.visible = false;
        let mut minimized = window("history", &events);
        minimized.minimized = true;
        let snapshot =
            HiddenWindows::prepare([main, window("editor", &events), hidden, minimized]).unwrap();
        snapshot.restore(false).unwrap();
        assert_eq!(
            *events.borrow(),
            [
                "hide:main",
                "hide:editor",
                "show:main",
                "show:editor",
                "focus:main"
            ]
        );
    }
    #[test]
    fn partial_hide_failure_rolls_back_before_returning() {
        let events = Rc::default();
        let mut broken = window("editor", &events);
        broken.fail_hide = true;
        assert!(HiddenWindows::prepare([window("main", &events), broken]).is_err());
        assert_eq!(
            *events.borrow(),
            ["hide:main", "hide:editor", "show:main", "show:editor"]
        );
    }
    #[test]
    fn snapshot_failure_leaves_every_window_untouched() {
        let events = Rc::default();
        let mut broken = window("editor", &events);
        broken.fail_read = true;
        assert!(HiddenWindows::prepare([window("main", &events), broken]).is_err());
        assert!(events.borrow().is_empty());
    }
    #[test]
    fn restore_failure_does_not_skip_other_windows() {
        let events = Rc::default();
        let mut broken = window("main", &events);
        broken.fail_show = true;
        let snapshot = HiddenWindows::prepare([broken, window("editor", &events)]).unwrap();
        assert!(snapshot.restore(false).is_err());
        assert!(events.borrow().contains(&"show:editor".into()));
    }
    #[test]
    fn success_keeps_whiteboard_and_old_preview_hidden() {
        let events = Rc::default();
        let snapshot = HiddenWindows::prepare([
            window("main", &events),
            window("preview", &events),
            window("setting", &events),
        ])
        .unwrap();
        snapshot.restore(true).unwrap();
        assert_eq!(
            *events.borrow(),
            ["hide:main", "hide:preview", "hide:setting", "show:setting"]
        );
    }
}
