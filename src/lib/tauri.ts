import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

export interface FileEntry {
  key: string;
  full_name: string;
  abs_path: string;
}

export interface Stats {
  total_left: number;
  total_right: number;
  missing_count: number;
}

export interface CompareResult {
  missing: FileEntry[];
  stats: Stats;
}

export interface CopyFailure {
  path: string;
  error: string;
}

export interface CopyReport {
  copied: number;
  skipped: number;
  failed: CopyFailure[];
}

export interface CopyProgress {
  done: number;
  total: number;
  current: string;
}

export async function pickDirectory(): Promise<string | null> {
  const result = await open({ directory: true, multiple: false });
  if (result == null) return null;
  return Array.isArray(result) ? (result[0] ?? null) : result;
}

export function compare(allDir: string, sentDir: string) {
  return invoke<CompareResult>("compare", { allDir, sentDir });
}

export function copyMissing(files: string[], outDir: string) {
  return invoke<CopyReport>("copy_missing", { files, outDir });
}

export function onCopyProgress(cb: (p: CopyProgress) => void): Promise<UnlistenFn> {
  return listen<CopyProgress>("copy-progress", (e) => cb(e.payload));
}
