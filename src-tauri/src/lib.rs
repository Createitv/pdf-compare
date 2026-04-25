use pdfcmp_algo::{compare, copy};
use std::path::PathBuf;

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

#[tauri::command]
fn copy_missing(files: Vec<String>, out_dir: String) -> Result<copy::CopyReport, String> {
    let out = PathBuf::from(&out_dir);
    copy::copy_files(&files, &out).map_err(|e| e.to_string())
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
