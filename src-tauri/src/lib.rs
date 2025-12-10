// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use dirs_next;
use reqwest::blocking::Client;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::json;
use std::env;
use std::path::Path;
use walkdir::WalkDir;
use tauri_plugin_log::{Target, TargetKind};

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
            synopsis_json TEXT,
            tmdb_id INTEGER
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

        // best-effort TMDB match for video files
        let mut best_tmdb: Option<i64> = None;
        if m.media_type == "video" {
            match tmdb_best_match(&title) {
                Ok(opt) => best_tmdb = opt,
                Err(e) => eprintln!("tmdb lookup failed for '{}': {}", title, e),
            }
        }

        tx.execute(
            "INSERT INTO medias (path, title, media_type, last_position, synopsis_json, tmdb_id)
             VALUES (?1, ?2, ?3, 0, NULL, ?4)
             ON CONFLICT(path) DO UPDATE SET media_type = excluded.media_type, title = excluded.title",
            params![m.path, title, m.media_type, best_tmdb],
        )
        .map_err(|e| format!("insert failed: {}", e))?;
    }

    tx.commit().map_err(|e| format!("commit failed: {}", e))?;
    Ok(())
}

fn get_connection() -> Result<Connection, String> {
    let db_path = get_db_path()?;
    Connection::open(db_path).map_err(|e| e.to_string())
}

/// Best-effort TMDB search to return a tmdb_id for a title.
/// Returns Ok(Some(id)) if a confident match is found, Ok(None) if not, Err on HTTP/parse errors.
fn tmdb_best_match(title: &str) -> Result<Option<i64>, String> {
    let api_key = match env::var("TMDB_API_KEY") {
        Ok(k) => k,
        Err(_) => return Ok(None), // no key => skip
    };

    let client = Client::new();

    // Try movie first, then tv
    let endpoints = [
        "https://api.themoviedb.org/3/search/movie",
        "https://api.themoviedb.org/3/search/tv",
    ];

    for ep in endpoints.iter() {
        let resp = client
            .get(*ep)
            .query(&[("api_key", api_key.as_str()), ("query", title)])
            .send()
            .map_err(|e| format!("http error: {}", e))?;

        if !resp.status().is_success() {
            continue;
        }

        let v: serde_json::Value = resp.json().map_err(|e| format!("parse error: {}", e))?;
        let results = v
            .get("results")
            .and_then(|r| r.as_array())
            .ok_or("no results array")?;
        if results.is_empty() {
            continue;
        }

        if let Some(first) = results.get(0) {
            let id = first.get("id").and_then(|i| i.as_i64()).ok_or("no id")?;
            // prefer fairly popular / voted results to avoid weak matches
            let popularity = first
                .get("popularity")
                .and_then(|p| p.as_f64())
                .unwrap_or(0.0);
            let vote_count = first
                .get("vote_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if popularity >= 2.0 || vote_count >= 20 {
                return Ok(Some(id));
            }
            // otherwise continue to next endpoint or return None
        }
    }

    Ok(None)
}

#[derive(Serialize)]
struct MediaDb {
    path: String,
    title: String,
    media_type: String,
    last_position: i64,
    synopsis_json: Option<String>,
    tmdb_id: Option<i64>,
}

#[derive(Serialize)]
struct ListResponse {
    items: Vec<MediaDb>,
    total: i64,
    page: u32,
    per_page: u32,
}

/// List medias with pagination and optional filters (media_type, query)
#[tauri::command]
fn list_medias(
    page: Option<u32>,
    per_page: Option<u32>,
    media_type: Option<&str>,
    query: Option<&str>,
) -> Result<ListResponse, String> {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(50).clamp(1, 500);
    let offset = ((page - 1) as i64) * (per_page as i64);

    let conn = get_connection()?;

    let like_query = query.map(|q| format!("%{}%", q));

    // Count total
    let mut count_stmt = conn
        .prepare(
            "SELECT COUNT(*) FROM medias
             WHERE (?1 IS NULL OR media_type = ?1)
               AND (?2 IS NULL OR title LIKE ?2 OR path LIKE ?2)",
        )
        .map_err(|e| e.to_string())?;

    let total: i64 = count_stmt
        .query_row(params![media_type, like_query.as_deref()], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT path, title, media_type, last_position, synopsis_json, tmdb_id FROM medias
             WHERE (?1 IS NULL OR media_type = ?1)
               AND (?2 IS NULL OR title LIKE ?2 OR path LIKE ?2)
             ORDER BY title COLLATE NOCASE ASC
             LIMIT ?3 OFFSET ?4",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(
            params![media_type, like_query.as_deref(), per_page as i64, offset],
            |row| {
                Ok(MediaDb {
                    path: row.get(0)?,
                    title: row.get(1)?,
                    media_type: row.get(2)?,
                    last_position: row.get(3)?,
                    synopsis_json: row.get(4)?,
                    tmdb_id: row.get(5)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let mut items: Vec<MediaDb> = Vec::new();
    for r in rows {
        items.push(r.map_err(|e| e.to_string())?);
    }

    Ok(ListResponse {
        items,
        total,
        page,
        per_page,
    })
}

/// Get a single media by path
#[tauri::command]
fn get_media(path: &str) -> Result<Option<MediaDb>, String> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare("SELECT path, title, media_type, last_position, synopsis_json, tmdb_id FROM medias WHERE path = ?1 LIMIT 1")
        .map_err(|e| e.to_string())?;

    let res = stmt
        .query_row(params![path], |row| {
            Ok(MediaDb {
                path: row.get(0)?,
                title: row.get(1)?,
                media_type: row.get(2)?,
                last_position: row.get(3)?,
                synopsis_json: row.get(4)?,
                tmdb_id: row.get(5)?,
            })
        })
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(res)
}

/// Update media fields (partial updates allowed)
#[tauri::command]
fn update_media(
    path: &str,
    last_position: Option<i64>,
    synopsis_json: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection()?;

    if last_position.is_some() {
        conn.execute(
            "UPDATE medias SET last_position = ?1 WHERE path = ?2",
            params![last_position.unwrap(), path],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(s) = synopsis_json {
        conn.execute(
            "UPDATE medias SET synopsis_json = ?1 WHERE path = ?2",
            params![s, path],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Delete a media by path
#[tauri::command]
fn delete_media(path: &str) -> Result<bool, String> {
    let conn = get_connection()?;
    let affected = conn
        .execute("DELETE FROM medias WHERE path = ?1", params![path])
        .map_err(|e| e.to_string())?;
    Ok(affected > 0)
}

#[derive(Serialize)]
struct MetadataResult {
    id: i64,
    title: String,
    year: Option<String>,
    overview: Option<String>,
    poster_url: Option<String>,
    poster_path: Option<String>,
}

/// Fetch metadata from TMDB for a given title and media_type ("movie" or "series").
/// Requires environment variable `TMDB_API_KEY` to be set.
#[tauri::command]
fn fetch_metadata(title: &str, media_type: &str) -> Result<serde_json::Value, String> {
    println!("fetch_metadata: start for '{}' ({})", title, media_type);

    let api_key =
        env::var("TMDB_API_KEY").map_err(|_| "TMDB_API_KEY env var is not set".to_string())?;
    println!("fetch_metadata: api key present, preparing request");

    let client = Client::new();

    let endpoint = match media_type {
        "movie" => "https://api.themoviedb.org/3/search/movie",
        "series" | "tv" => "https://api.themoviedb.org/3/search/tv",
        _ => return Err("media_type must be 'movie' or 'series'".to_string()),
    };

    println!("fetch_metadata: sending request to {}", endpoint);
    let resp = client
        .get(endpoint)
        .query(&[("api_key", api_key.as_str()), ("query", title)])
        .send()
        .map_err(|e| format!("http error: {}", e))?;

    println!("fetch_metadata: received status {}", resp.status());
    if !resp.status().is_success() {
        return Err(format!("TMDB returned status {}", resp.status()));
    }

    println!("fetch_metadata: parsing JSON response");
    let v: serde_json::Value = resp.json().map_err(|e| format!("parse error: {}", e))?;
    let results = v
        .get("results")
        .and_then(|r| r.as_array())
        .ok_or("no results")?;
    let first = results.get(0).ok_or("no results for query")?;

    let id = first
        .get("id")
        .and_then(|i| i.as_i64())
        .ok_or("no id in result")?;

    println!("fetch_metadata: selected TMDB id {}", id);

    let overview = first
        .get("overview")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());
    let poster_path = first
        .get("poster_path")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    let year = first
        .get("release_date")
        .or_else(|| first.get("first_air_date"))
        .and_then(|d| d.as_str())
        .and_then(|d| d.split('-').next())
        .map(|s| s.to_string());

    let poster_url = poster_path
        .as_ref()
        .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p));

    // download and cache poster locally, if available
    let mut local_poster: Option<String> = None;
    if let Some(ref poster) = poster_path {
        println!("fetch_metadata: poster available at {}", poster);
        if let Ok(mut posters_dir) = get_db_path() {
            posters_dir.pop(); // epikodi.sqlite -> epikodi dir
            posters_dir.push("posters");
            if let Err(e) = std::fs::create_dir_all(&posters_dir) {
                eprintln!("fetch_metadata: failed to create posters dir: {}", e);
                return Err(format!("failed to create posters dir: {}", e));
            }

            // extension from poster path
            let ext = Path::new(poster)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("jpg");
            let filename = format!("tmdb_{}.{}", id, ext);
            posters_dir.push(&filename);
            let local_path = posters_dir.clone();

            println!("fetch_metadata: local poster path will be {:?}", local_path);

            // If file already exists, reuse it
            if !local_path.exists() {
                if let Some(ref url) = poster_url {
                    println!("fetch_metadata: downloading poster from {}", url);
                    let bytes = client
                        .get(url)
                        .send()
                        .map_err(|e| format!("failed to download poster: {}", e))?
                        .bytes()
                        .map_err(|e| format!("failed to read poster bytes: {}", e))?;

                    println!("fetch_metadata: writing poster to disk");
                    std::fs::write(&local_path, &bytes)
                        .map_err(|e| format!("failed to write poster file: {}", e))?;
                }
            } else {
                println!("fetch_metadata: poster already cached");
            }

            local_poster = Some(local_path.to_string_lossy().to_string());
        } else {
            eprintln!("fetch_metadata: could not determine posters dir");
        }
    } else {
        println!("fetch_metadata: no poster path in TMDB result");
    }

    let metadata_json = json!({
        "id": id,
        "title": title,
        "year": year,
        "overview": overview,
        "poster_url": poster_url,
        "poster_path": local_poster,
    });

    // update medias table: set synopsis_json and tmdb_id where lower(title) = lower(provided title)
    println!(
        "fetch_metadata: updating DB synopsis_json and tmdb_id for title '{}'",
        title
    );
    if let Ok(conn) = get_connection() {
        let mstr = metadata_json.to_string();
        let _ = conn.execute(
            "UPDATE medias SET synopsis_json = ?1, tmdb_id = ?2 WHERE lower(title) = lower(?3)",
            params![mstr, id, title],
        );
        println!("fetch_metadata: DB update attempted");
    } else {
        eprintln!("fetch_metadata: failed to open DB for update");
    }

    println!("fetch_metadata: finished for '{}'", title);
    Ok(metadata_json)
}

// J%6ll9DDJrRbqt
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            scan_directory,
            list_medias,
            get_media,
            update_media,
            delete_media,
            fetch_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
