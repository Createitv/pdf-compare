import { invoke } from "@tauri-apps/api/core";
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
  anomaly_count: number;
}

export interface CompareResult {
  missing: FileEntry[];
  anomalies: FileEntry[];
  stats: Stats;
}

export interface CopyReport {
  copied: number;
  failed: { path: string; error: string }[];
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
