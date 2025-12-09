// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use serde::Serialize;
use walkdir::WalkDir;

#[derive(Serialize)]
struct MediaFile {
    path: String,
    media_type: String,
}

/// Scan a directory recursively and return media file paths and types.
#[tauri::command]
fn scan_directory(path: &str) -> Result<Vec<MediaFile>, String> {
    let mut results: Vec<MediaFile> = Vec::new();

    let walker = WalkDir::new(path).into_iter();
    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext_os) = entry.path().extension() {
                if let Some(ext) = ext_os.to_str() {
                    let ext = ext.to_lowercase();
                    let media_type = match ext.as_str() {
                        // video
                        "mp4" | "mkv" | "mov" | "avi" | "m4v" | "webm" | "flv" => Some("video"),
                        // audio
                        "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" => Some("audio"),
                        _ => None,
                    };

                    if let Some(mt) = media_type {
                        let p = entry.path().to_string_lossy().to_string();
                        results.push(MediaFile {
                            path: p,
                            media_type: mt.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(results)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, scan_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
