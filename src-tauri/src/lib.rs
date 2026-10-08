mod common;
mod convert;
mod pdf;
mod raster;
mod video;

use std::path::PathBuf;

use tauri::ipc::Channel;

use common::{Level, Outcome, Progress};

/// Dosyayı uzantısına göre ilgili sıkıştırıcıya yönlendirir.
#[tauri::command]
async fn compress_file(path: String, level: Level, on_progress: Channel<Progress>) -> Result<Outcome, String> {
    let path = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || {
        // Pencere kapanmışsa gönderim hatası önemsizdir; sıkıştırma sürer.
        let mut report = |progress| {
            let _ = on_progress.send(progress);
        };
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "pdf" => pdf::compress_file(&path, level, &mut report),
            "mp4" => video::compress_file(&path, level, &mut report),
            other => match raster::Kind::from_extension(other) {
                Some(kind) => raster::compress_file(&path, kind, level, &mut report),
                None => Err("Bu dosya türü desteklenmiyor.".into()),
            },
        }
    })
    .await
    .map_err(|e| format!("İşlem yarıda kaldı: {e}"))?
}

/// PNG ↔ WebP dönüştürme. `lossless` yalnızca WebP hedefinde anlamlıdır.
#[tauri::command]
async fn convert_file(
    path: String,
    to: convert::Target,
    lossless: bool,
    on_progress: Channel<Progress>,
) -> Result<Outcome, String> {
    let path = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || {
        convert::convert_file(&path, to, lossless, &mut |progress| {
            let _ = on_progress.send(progress);
        })
    })
    .await
    .map_err(|e| format!("İşlem yarıda kaldı: {e}"))?
}

#[tauri::command]
async fn ffmpeg_status() -> video::FfmpegStatus {
    tauri::async_runtime::spawn_blocking(video::status)
        .await
        .expect("FFmpeg durumu okunamadı")
}

#[tauri::command]
fn install_ffmpeg() -> Result<video::InstallMethod, String> {
    video::start_install()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![compress_file, convert_file, ffmpeg_status, install_ffmpeg])
        .run(tauri::generate_context!())
        .expect("uygulama başlatılamadı");
}
