use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_store::StoreExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actions {
    pub editor: bool,
    pub copy: bool,
    pub save: bool,
}
impl Default for Actions {
    fn default() -> Self {
        Self {
            editor: true,
            copy: false,
            save: false,
        }
    }
}
impl Actions {
    fn validate(&self) -> Result<(), String> {
        if self.editor || self.copy || self.save {
            Ok(())
        } else {
            Err("至少保留一个截图后动作".into())
        }
    }
}

#[tauri::command]
pub fn get_capture_actions(app: tauri::AppHandle) -> Result<Actions, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let actions: Actions = match store.get("capture_actions") {
        Some(value) => serde_json::from_value(value).map_err(|e| e.to_string())?,
        None => Actions::default(),
    };
    actions.validate()?;
    Ok(actions)
}

#[tauri::command]
pub fn set_capture_actions(app: tauri::AppHandle, actions: Actions) -> Result<(), String> {
    crate::capture::ensure_idle(&app)?;
    actions.validate()?;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let previous = store.get("capture_actions");
    store.set(
        "capture_actions",
        serde_json::to_value(actions).map_err(|e| e.to_string())?,
    );
    if let Err(error) = store.save() {
        match previous {
            Some(value) => store.set("capture_actions", value),
            None => {
                store.delete("capture_actions");
            }
        }
        return Err(error.to_string());
    }
    Ok(())
}

fn png(app: &tauri::AppHandle, id: &str) -> Result<Vec<u8>, String> {
    let root = crate::common::get_images_dir(app, "projects".into())?;
    crate::projects::ProjectStore::open(root)
        .and_then(|store| store.png(id))
        .map_err(|e| format!("{e:#}"))
}

pub fn copy(app: &tauri::AppHandle, bytes: &[u8]) -> Result<(), String> {
    let rgba = image::load_from_memory(bytes)
        .map_err(|e| e.to_string())?
        .into_rgba8();
    app.clipboard()
        .write_image(&tauri::image::Image::new(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
        ))
        .map_err(|e| e.to_string())
}

pub fn save_unique(directory: &Path, name: &str, bytes: &[u8]) -> std::io::Result<PathBuf> {
    fs::create_dir_all(directory)?;
    for suffix in 0..10000 {
        let path = directory.join(format!("{name}-{suffix}.png"));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
                    let _ = fs::remove_file(&path);
                    return Err(error);
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::other("无法分配唯一保存文件名"))
}

pub fn after_capture(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    let actions = get_capture_actions(app.clone())?;
    if !actions.copy && !actions.save {
        return Ok(());
    }
    let bytes = png(app, id)?;
    let mut errors = Vec::new();
    if actions.copy {
        if let Err(error) = copy(app, &bytes) {
            errors.push(format!("复制失败：{error}"));
        }
    }
    if actions.save {
        let result = crate::common::get_images_dir(app, "exports".into())
            .and_then(|dir| save_unique(&dir, id, &bytes).map_err(|e| e.to_string()));
        if let Err(error) = result {
            errors.push(format!("自动保存失败：{error}"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "截图项目已保存，可从历史记录重新打开。{}",
            errors.join("；")
        ))
    }
}

#[tauri::command]
pub async fn copy_project(app: tauri::AppHandle, id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || copy(&app, &png(&app, &id)?))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn save_project(app: tauri::AppHandle, id: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = png(&app, &id)?;
        let Some(selected) = app
            .dialog()
            .file()
            .add_filter("PNG 图片", &["png"])
            .set_file_name(format!("{id}.png"))
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let mut path = selected.into_path().map_err(|e| e.to_string())?;
        path.set_extension("png");
        let parent = path
            .parent()
            .ok_or("保存路径无效")?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let root = crate::common::get_images_dir(&app, "projects".into())?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if parent.starts_with(root) {
            return Err("请选择项目存储目录以外的位置，避免修改项目原图。".into());
        }
        let replacing = path.try_exists().map_err(|e| e.to_string())?;
        if let Ok(meta) = fs::symlink_metadata(&path) {
            if !meta.is_file() {
                return Err("目标不是普通文件".into());
            }
            if !app
                .dialog()
                .message("目标文件已存在，是否替换？")
                .buttons(MessageDialogButtons::OkCancel)
                .blocking_show()
            {
                return Ok(None);
            }
        }
        let temporary = save_unique(&parent, &format!(".dyd-export-{id}"), &bytes)
            .map_err(|e| e.to_string())?;
        let result = if replacing {
            fs::rename(&temporary, &path)
        } else {
            fs::hard_link(&temporary, &path).and_then(|_| fs::remove_file(&temporary))
        };
        if let Err(error) = result {
            let _ = fs::remove_file(temporary);
            return Err(error.to_string());
        }
        Ok(Some(path.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_actions_are_rejected() {
        assert!(Actions::default().validate().is_ok());
        assert!(Actions {
            editor: false,
            copy: false,
            save: false
        }
        .validate()
        .is_err());
    }
    #[test]
    fn automatic_export_never_overwrites() {
        let root = std::env::temp_dir().join(format!("dyd-delivery-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let a = save_unique(&root, "image", b"original").unwrap();
        let b = save_unique(&root, "image", b"second").unwrap();
        assert_ne!(a, b);
        assert_eq!(fs::read(a).unwrap(), b"original");
        assert_eq!(fs::read(b).unwrap(), b"second");
        fs::remove_dir_all(root).unwrap();
    }
}
