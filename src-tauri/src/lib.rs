use pdfcmp_algo::{compare, copy};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn compare(all_dir: String, sent_dir: String) -> Result<compare::CompareResult, String> {
    let a = PathBuf::from(&all_dir);
    let b = PathBuf::from(&sent_dir);
    if !a.is_dir() {
        return Err(format!("路径不是文件夹: {all_dir}"));
    }
    if !b.is_dir() {
        return Err(format!("路径不是文件夹: {sent_dir}"));
    }
    Ok(compare::compare_dirs(&a, &b))
}

#[derive(Clone, Serialize)]
struct CopyProgress {
    done: usize,
    total: usize,
    current: String,
}

#[tauri::command]
fn copy_missing(
    app: AppHandle,
    files: Vec<String>,
    out_dir: String,
) -> Result<copy::CopyReport, String> {
    let out = PathBuf::from(&out_dir);
    let app_for_cb = app.clone();
    copy::copy_files(&files, &out, move |done, total, current| {
        let _ = app_for_cb.emit(
            "copy-progress",
            CopyProgress {
                done,
                total,
                current: current.to_string(),
            },
        );
    })
    .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![compare, copy_missing])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
