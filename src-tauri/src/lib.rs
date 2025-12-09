// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use serde::Serialize;
use walkdir::WalkDir;
use rusqlite::{params, Connection};
use std::path::Path;
use dirs_next;

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

    // persist scan results to SQLite
    if let Err(e) = persist_results(&results) {
        return Err(format!("failed to persist scan results: {}", e));
    }

    Ok(results)
}

fn get_db_path() -> Result<std::path::PathBuf, String> {
    let mut base = dirs_next::data_dir().ok_or("Could not determine data directory")?;
    base.push("epikodi");
    std::fs::create_dir_all(&base).map_err(|e| format!("Failed to create data dir: {}", e))?;
    base.push("epikodi.sqlite");
    Ok(base)
}

fn persist_results(results: &[MediaFile]) -> Result<(), String> {
    let db_path = get_db_path()?;
    println!("Using database at {:?}", db_path);
    let mut conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS medias (
            path TEXT PRIMARY KEY,
            title TEXT,
            media_type TEXT,
            last_position INTEGER DEFAULT 0,
            synopsis_json TEXT
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("failed to start transaction: {}", e))?;

    for m in results {
        let title = Path::new(&m.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        tx.execute(
            "INSERT INTO medias (path, title, media_type, last_position, synopsis_json)
             VALUES (?1, ?2, ?3, 0, NULL)
             ON CONFLICT(path) DO UPDATE SET media_type = excluded.media_type, title = excluded.title",
            params![m.path, title, m.media_type],
        )
        .map_err(|e| format!("insert failed: {}", e))?;
    }

    tx.commit().map_err(|e| format!("commit failed: {}", e))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, scan_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
