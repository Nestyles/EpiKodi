import { invoke } from "@tauri-apps/api/core";

export async function scanDirectory(path: string) {
  return invoke("scan_directory", { path });
}

export async function fetchMetadata(title: string) {
  return invoke("fetch_metadata", { title });
}

export async function listMedias(options?: {
  page?: number;
  per_page?: number;
  media_type?: string | null;
  query?: string | null;
}) {
  const { page = 1, per_page = 50, media_type = null, query = null } = options || {};
  return invoke("list_medias", { page, per_page, media_type, query });
}

export async function getMedia(path: string) {
  return invoke("get_media", { path });
}

export async function updateMedia(args: { path: string; last_position?: number | null; synopsis_json?: string | null; }) {
  return invoke("update_media", { path: args.path, last_position: args.last_position ?? null, synopsis_json: args.synopsis_json ?? null });
}

export async function deleteMedia(path: string) {
  return invoke("delete_media", { path });
}
