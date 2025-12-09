import { invoke } from "@tauri-apps/api/tauri";

export async function scanDirectory(path: string) {
  return invoke("scan_directory", { path });
}

export async function fetchMetadata(title: string, mediaType: "movie" | "series") {
  return invoke("fetch_metadata", { title, media_type: mediaType });
}
