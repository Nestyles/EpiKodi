use dirs_next;
use reqwest::blocking::Client;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::json;
use std::env;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Serialize)]
struct MediaFile {
    path: String,
    media_type: String,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
                        // images
                        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => Some("image"),
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
    persist_results_internal(&mut conn, results)
}

fn persist_results_internal(conn: &mut Connection, results: &[MediaFile]) -> Result<(), String> {

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
/// Internal function for testing
fn list_medias_internal(
    conn: &Connection,
    page: Option<u32>,
    per_page: Option<u32>,
    media_type: Option<String>,
    query: Option<String>,
    has_metadata: Option<bool>,
) -> Result<ListResponse, String> {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(50).clamp(1, 500);
    let offset = ((page - 1) as i64) * (per_page as i64);

    let like_query = query.as_ref().map(|q| format!("%{}%", q.to_lowercase()));

    // Count total
    let mut count_stmt = conn
        .prepare(
            "SELECT COUNT(*) FROM medias
             WHERE (?1 IS NULL OR media_type = ?1)
               AND (?2 IS NULL OR LOWER(title) LIKE ?2)
               AND (?3 IS NULL OR (?3 = 1 AND synopsis_json IS NOT NULL) OR (?3 = 0 AND synopsis_json IS NULL))",
        )
        .map_err(|e| e.to_string())?;

    let total: i64 = count_stmt
        .query_row(params![media_type.as_deref(), like_query.as_deref(), has_metadata.map(|b| b as i32)], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT path, title, media_type, last_position, synopsis_json, tmdb_id FROM medias
             WHERE (?1 IS NULL OR media_type = ?1)
               AND (?2 IS NULL OR LOWER(title) LIKE ?2)
               AND (?5 IS NULL OR (?5 = 1 AND synopsis_json IS NOT NULL) OR (?5 = 0 AND synopsis_json IS NULL))
             ORDER BY title COLLATE NOCASE ASC
             LIMIT ?3 OFFSET ?4",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(
            params![media_type.as_deref(), like_query.as_deref(), per_page as i64, offset, has_metadata.map(|b| b as i32)],
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

/// List medias with pagination and optional filters (media_type, query, has_metadata)
#[tauri::command]
fn list_medias(
    page: Option<u32>,
    per_page: Option<u32>,
    media_type: Option<&str>,
    query: Option<String>,
    has_metadata: Option<bool>,
) -> Result<ListResponse, String> {
    let conn = get_connection()?;
    list_medias_internal(&conn, page, per_page, media_type.map(|s| s.to_string()), query, has_metadata)
}

/// Get a single media by path
/// Internal function for testing
fn get_media_internal(conn: &Connection, path: &str) -> Result<Option<MediaDb>, String> {
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

/// Get a single media by path
#[tauri::command]
fn get_media(path: &str) -> Result<Option<MediaDb>, String> {
    let conn = get_connection()?;
    get_media_internal(&conn, path)
}

/// Update media fields (partial updates allowed)
/// Internal function for testing
fn update_media_internal(
    conn: &Connection,
    path: &str,
    last_position: Option<i64>,
    synopsis_json: Option<&str>,
) -> Result<(), String> {
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

/// Update media fields (partial updates allowed)
#[tauri::command]
fn update_media(
    path: &str,
    last_position: Option<i64>,
    synopsis_json: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection()?;
    update_media_internal(&conn, path, last_position, synopsis_json)
}

/// Delete a media by path
/// Internal function for testing
fn delete_media_internal(conn: &Connection, path: &str) -> Result<bool, String> {
    let affected = conn
        .execute("DELETE FROM medias WHERE path = ?1", params![path])
        .map_err(|e| e.to_string())?;
    Ok(affected > 0)
}

/// Delete a media by path
#[tauri::command]
fn delete_media(path: &str) -> Result<bool, String> {
    let conn = get_connection()?;
    delete_media_internal(&conn, path)
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

/// Fetch metadata from TMDB for a given title.
/// Tries movie first, then TV series.
/// Requires environment variable `TMDB_API_KEY` to be set.
#[tauri::command]
fn fetch_metadata(title: &str) -> Result<serde_json::Value, String> {
    println!("fetch_metadata: start for '{}'", title);

    let api_key =
        env::var("TMDB_API_KEY").map_err(|_| "TMDB_API_KEY env var is not set".to_string())?;
    println!("fetch_metadata: api key present, preparing request");

    let client = Client::new();

    // Try movie first, then tv
    let endpoints = [
        ("movie", "https://api.themoviedb.org/3/search/movie"),
        ("tv", "https://api.themoviedb.org/3/search/tv"),
    ];

    let mut best_result: Option<serde_json::Value> = None;
    let mut best_id: Option<i64> = None;

    for (media_type, endpoint) in endpoints.iter() {
        println!("fetch_metadata: trying {} endpoint", media_type);
        let resp = client
            .get(*endpoint)
            .query(&[("api_key", api_key.as_str()), ("query", title)])
            .send()
            .map_err(|e| format!("http error: {}", e))?;

        println!("fetch_metadata: received status {}", resp.status());
        if !resp.status().is_success() {
            continue;
        }

        println!("fetch_metadata: parsing JSON response");
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse error: {}", e))?;
        let results = v
            .get("results")
            .and_then(|r| r.as_array())
            .ok_or("no results")?;
        if results.is_empty() {
            continue;
        }

        let first = results.get(0).ok_or("no results for query")?;

        let id = first
            .get("id")
            .and_then(|i| i.as_i64())
            .ok_or("no id in result")?;

        // Check popularity/vote_count to prefer better matches
        let popularity = first
            .get("popularity")
            .and_then(|p| p.as_f64())
            .unwrap_or(0.0);
        let vote_count = first
            .get("vote_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if popularity >= 2.0 || vote_count >= 20 {
            println!("fetch_metadata: found good match with id {}", id);
            best_result = Some(first.clone());
            best_id = Some(id);
            break; // prefer movie over tv if both match
        }
    }

    let first = best_result.ok_or("no suitable results found")?;
    let id = best_id.unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World! You've been greeted from Rust!");
    }

    #[test]
    fn test_scan_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create some test files
        fs::create_dir_all(temp_path.join("subdir")).unwrap();
        fs::write(temp_path.join("video.mp4"), b"fake video").unwrap();
        fs::write(temp_path.join("audio.mp3"), b"fake audio").unwrap();
        fs::write(temp_path.join("text.txt"), b"not media").unwrap();
        fs::write(temp_path.join("subdir/video2.mkv"), b"another video").unwrap();

        let result = scan_directory(&temp_path.to_string_lossy()).unwrap();

        // Should find 3 media files
        assert_eq!(result.len(), 3);

        let video1 = result.iter().find(|m| m.path.contains("video.mp4")).unwrap();
        assert_eq!(video1.media_type, "video");

        let audio = result.iter().find(|m| m.path.contains("audio.mp3")).unwrap();
        assert_eq!(audio.media_type, "audio");

        let video2 = result.iter().find(|m| m.path.contains("video2.mkv")).unwrap();
        assert_eq!(video2.media_type, "video");
    }

    #[test]
    fn test_get_db_path() {
        // This will create a path in the data dir, but for test we can just check it doesn't error
        let path = get_db_path().unwrap();
        assert!(path.ends_with("epikodi.sqlite"));
    }

    #[test]
    fn test_tmdb_best_match_no_key() {
        // Without API key, should return None
        std::env::remove_var("TMDB_API_KEY");
        let result = tmdb_best_match("test").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_persist_results() {
        let mut conn = Connection::open_in_memory().unwrap();
        let results = vec![
            MediaFile { path: "/path/video.mp4".to_string(), media_type: "video".to_string() },
            MediaFile { path: "/path/audio.mp3".to_string(), media_type: "audio".to_string() },
        ];

        persist_results_internal(&mut conn, &results).unwrap();

        // Check if data was inserted
        let result = list_medias_internal(&conn, None, None, None, None, None).unwrap();
        assert_eq!(result.items.len(), 2);
    }

    // For list_medias, get_media, etc., we can create a temp DB and insert data manually.

    #[test]
    fn test_list_medias() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE medias (
                path TEXT PRIMARY KEY,
                title TEXT,
                media_type TEXT,
                last_position INTEGER DEFAULT 0,
                synopsis_json TEXT,
                tmdb_id INTEGER
            )",
            [],
        ).unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO medias (path, title, media_type) VALUES (?1, ?2, ?3)",
            params!["/path/video.mp4", "Video", "video"],
        ).unwrap();
        conn.execute(
            "INSERT INTO medias (path, title, media_type) VALUES (?1, ?2, ?3)",
            params!["/path/audio.mp3", "Audio", "audio"],
        ).unwrap();

        let result = list_medias_internal(&conn, None, None, None, None, None).unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total, 2);

        // Test filtering by media_type
        let result = list_medias_internal(&conn, None, None, Some("video".to_string()), None, None).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].title, "Video");
    }

    // Similarly for other DB functions.

    // For fetch_metadata, we can use mockito to mock the HTTP responses.

    #[test]
    fn test_get_media() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE medias (
                path TEXT PRIMARY KEY,
                title TEXT,
                media_type TEXT,
                last_position INTEGER DEFAULT 0,
                synopsis_json TEXT,
                tmdb_id INTEGER
            )",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO medias (path, title, media_type) VALUES (?1, ?2, ?3)",
            params!["/path/video.mp4", "Video", "video"],
        ).unwrap();

        let result = get_media_internal(&conn, "/path/video.mp4").unwrap();
        assert!(result.is_some());
        let media = result.unwrap();
        assert_eq!(media.title, "Video");

        let result = get_media_internal(&conn, "/path/nonexistent.mp4").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_media() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE medias (
                path TEXT PRIMARY KEY,
                title TEXT,
                media_type TEXT,
                last_position INTEGER DEFAULT 0,
                synopsis_json TEXT,
                tmdb_id INTEGER
            )",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO medias (path, title, media_type) VALUES (?1, ?2, ?3)",
            params!["/path/video.mp4", "Video", "video"],
        ).unwrap();

        update_media_internal(&conn, "/path/video.mp4", Some(100), Some("{\"test\": \"data\"}")).unwrap();

        let result = get_media_internal(&conn, "/path/video.mp4").unwrap().unwrap();
        assert_eq!(result.last_position, 100);
        assert_eq!(result.synopsis_json, Some("{\"test\": \"data\"}".to_string()));
    }

    #[test]
    fn test_delete_media() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE medias (
                path TEXT PRIMARY KEY,
                title TEXT,
                media_type TEXT,
                last_position INTEGER DEFAULT 0,
                synopsis_json TEXT,
                tmdb_id INTEGER
            )",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO medias (path, title, media_type) VALUES (?1, ?2, ?3)",
            params!["/path/video.mp4", "Video", "video"],
        ).unwrap();

        let deleted = delete_media_internal(&conn, "/path/video.mp4").unwrap();
        assert!(deleted);

        let result = get_media_internal(&conn, "/path/video.mp4").unwrap();
        assert!(result.is_none());

        let deleted = delete_media_internal(&conn, "/path/nonexistent.mp4").unwrap();
        assert!(!deleted);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env file for environment variables
    let _ = dotenv::dotenv();

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
