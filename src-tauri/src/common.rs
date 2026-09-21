use std::path::{Path, PathBuf};

use image::ImageReader;
use tauri::{image::Image, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_store::StoreExt;
use tracing::info;

pub fn get_images_dir(app_handle: &tauri::AppHandle, path: String) -> Result<PathBuf, String> {
    let store = app_handle
        .get_store("settings.json")
        .ok_or_else(|| "Could not get settings store".to_string())?;

    let screenshot_path = store
        .get("screenshot_path")
        .ok_or_else(|| "Screenshot path not found in settings".to_string())?;
    info!("screenshot_path: {:?}", screenshot_path);

    let screenshot_path = crate::settings::screenshot_path(&screenshot_path)?;

    // 获取 AppLocalData 路径
    // 如果 screenshot_path 为空，则使用 app_local_data
    let app_local_data = if screenshot_path.is_empty() {
        app_handle
            .path()
            .app_local_data_dir()
            .map_err(|e| e.to_string())?
    } else {
        PathBuf::from(screenshot_path)
    };
    info!("app_local_data: {:?}", app_local_data);

    // 创建 images 文件夹
    // path 如果为空，则使用 app_local_data
    let images_dir = if path.is_empty() {
        app_local_data
    } else {
        app_local_data.join(path)
    };

    info!("images_dir: {:?}", images_dir);
    std::fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;

    // let filename = format!("screenshot_{}.png", Local::now().format("%Y%m%d_%H%M%S"));
    // let output_path = images_dir.join(&filename);

    Ok(images_dir)
}

pub async fn copy_picture_to_clipboard(
    app_handle: tauri::AppHandle,
    path: String,
) -> Result<(), String> {
    // Validate file exists
    let path = Path::new(&path);
    if !path.exists() {
        return Err("Image file does not exist".to_string());
    }

    let img = ImageReader::open(&path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    let rgba = img.into_rgba8();
    let width = rgba.width() as u32;
    let height = rgba.height() as u32;
    let rgba_data = rgba.into_raw();
    let img = Image::new(&rgba_data, width, height);

    // Copy to clipboard
    app_handle
        .clipboard()
        .write_image(&img)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn get_image_base64_by_path(path: String) -> Result<String, String> {
    use base64::{
        engine::general_purpose,
        Engine as _,
    };
    // Validate file exists
    let path = Path::new(&path);
    if !path.exists() {
        return Err("Image file does not exist".to_string());
    }

    let img = ImageReader::open(&path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    // Convert to base64 string
    let b64 = general_purpose::STANDARD.encode(encode_png(&img)?);

    Ok(b64)
}

pub fn encode_png(image: &image::DynamicImage) -> Result<Vec<u8>, String> {
    let mut output = std::io::Cursor::new(Vec::new());
    image.write_to(&mut output, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(output.into_inner())
}

#[cfg(test)]
mod encoding_tests {
    #[test]
    fn png_encoding_preserves_dimensions_and_transparency() {
        let pixels = image::RgbaImage::from_pixel(7, 19, image::Rgba([12, 34, 56, 78]));
        let bytes = super::encode_png(&image::DynamicImage::ImageRgba8(pixels.clone())).unwrap();
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(image::load_from_memory(&bytes).unwrap().to_rgba8(), pixels);
    }
}
