use crate::capture_task::{CaptureKind, CaptureState};
use core_foundation::{base::TCFType, boolean::CFBoolean, string::CFString};
use std::{path::Path, process::Command};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

pub fn request_capture_permission() -> bool {
    unsafe { CGRequestScreenCaptureAccess() }
}

pub fn capture(kind: CaptureKind, path: &Path) -> Result<CaptureState, String> {
    if !unsafe { CGPreflightScreenCaptureAccess() } {
        return Err("请在系统设置 → 隐私与安全性 → 屏幕录制中允许 DYD，然后重新启动应用。".into());
    }
    if path.try_exists().map_err(|e| e.to_string())? {
        return Err("截图输出文件已存在，未覆盖原文件。".into());
    }
    let mut command = Command::new("/usr/sbin/screencapture");
    command.args(["-x", "-t", "png"]);
    match kind {
        CaptureKind::Screen => {}
        CaptureKind::Select => {
            command.args(["-i", "-s"]);
        }
        CaptureKind::Window => {
            command.args(["-i", "-w", "-o"]);
        }
    }
    let output = command.arg(path).output().map_err(|e| e.to_string())?;
    let exists = path.try_exists().map_err(|e| e.to_string())?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let state = classify(kind, output.status.code(), stderr.trim(), exists);
    match state {
        Ok(true) => match image::image_dimensions(path) {
            Ok((width, height)) if width > 0 && height > 0 => Ok(CaptureState::Ready {
                path: path.to_string_lossy().into_owned(),
            }),
            _ => {
                std::fs::remove_file(path).map_err(|e| format!("截图无效，清理失败：{e}"))?;
                Err("截图文件为空或无法读取。".into())
            }
        },
        Ok(false) => Ok(CaptureState::Cancelled),
        Err(error) => {
            if exists {
                std::fs::remove_file(path).map_err(|e| format!("{error}；清理失败：{e}"))?;
            }
            Err(error)
        }
    }
}

// Interactive Escape exits without a file; diagnostics and signals remain failures.
fn classify(
    kind: CaptureKind,
    code: Option<i32>,
    stderr: &str,
    exists: bool,
) -> Result<bool, String> {
    if code == Some(0) && exists {
        return Ok(true);
    }
    if kind != CaptureKind::Screen && !exists && stderr.is_empty() && matches!(code, Some(0 | 1)) {
        return Ok(false);
    }
    Err(format!(
        "截图失败（退出状态 {code:?}）：{}",
        if stderr.is_empty() {
            "未生成有效图片"
        } else {
            stderr
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interactive_cancel_is_not_an_error() {
        for kind in [CaptureKind::Select, CaptureKind::Window] {
            for code in [0, 1] {
                assert_eq!(classify(kind, Some(code), "", false), Ok(false));
            }
        }
    }
    #[test]
    fn failures_cannot_be_reported_as_success_or_cancel() {
        assert!(classify(CaptureKind::Screen, Some(0), "", false).is_err());
        assert!(classify(CaptureKind::Select, Some(1), "permission denied", false).is_err());
        assert!(classify(CaptureKind::Window, None, "", false).is_err());
        assert!(classify(CaptureKind::Screen, Some(1), "", true).is_err());
    }
    #[test]
    fn success_requires_both_exit_status_and_file() {
        assert_eq!(classify(CaptureKind::Screen, Some(0), "", true), Ok(true));
    }
}

pub fn open_screen_capture_preferences() {
    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
        .spawn()
        .expect("failed to open system preferences");
}

pub fn check_accessibility_permissions() -> bool {
    let options = {
        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let value = CFBoolean::false_value();
        let pairs = &[(key, value)];
        core_foundation::dictionary::CFDictionary::from_CFType_pairs(pairs)
    };

    let trusted = unsafe {
        let accessibility = CFString::new("AXIsProcessTrustedWithOptions");
        let func: extern "C" fn(*const core_foundation::dictionary::CFDictionary) -> bool =
            std::mem::transmute(libc::dlsym(
                libc::RTLD_DEFAULT,
                accessibility.to_string().as_ptr() as *const _,
            ));
        func(options.as_concrete_TypeRef() as *const _)
    };

    trusted
}
