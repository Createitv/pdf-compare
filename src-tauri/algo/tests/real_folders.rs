//! Integration test against the real-world `全部/` and `已发送/` sample folders
//! at the repo root. Verifies the two-tier matching logic end-to-end.
//!
//! 运行：`cd src-tauri/algo && cargo test --test real_folders -- --nocapture`

use pdfcmp_algo::compare::compare_dirs;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = .../pdf-compare/src-tauri/algo
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn current_dataset_matches_expected_counts() {
    let root = project_root();
    let all = root.join("全部");
    let sent = root.join("已发送");

    if !all.is_dir() || !sent.is_dir() {
        eprintln!(
            "skip: 缺少测试数据目录 {} / {}",
            all.display(),
            sent.display()
        );
        return;
    }

    let r = compare_dirs(&all, &sent);

    println!(
        "\n[real-data] 左 {} / 右 {} / 缺失 {} / 多余 {}",
        r.stats.total_left, r.stats.total_right, r.stats.missing_count, r.stats.extra_count
    );
    println!("--- 缺失 ---");
    for f in &r.missing {
        println!("  {}", f.full_name);
    }
    if !r.extra.is_empty() {
        println!("--- 多余 ---");
        for f in &r.extra {
            println!("  {}", f.full_name);
        }
    }

    // 期望：当前样本 34 / 9 / 25 / 0
    assert_eq!(r.stats.total_left, 34, "left total mismatch");
    assert_eq!(r.stats.total_right, 9, "right total mismatch");
    assert_eq!(r.stats.extra_count, 0, "已发送 必须是 全部 的子集");
    assert_eq!(r.stats.missing_count, 25, "缺失数量与预期不符");

    // 不变量：matched = total_left - missing_count == total_right - extra_count
    let matched_left = r.stats.total_left - r.stats.missing_count;
    let matched_right = r.stats.total_right - r.stats.extra_count;
    assert_eq!(
        matched_left, matched_right,
        "匹配数对不上：左侧匹配 {matched_left} ≠ 右侧匹配 {matched_right}"
    );

    // 已发送的每个文件都应该精确出现在 全部 里（按完整文件名）
    use std::collections::HashSet;
    let mut all_names: HashSet<String> = HashSet::new();
    for entry in walkdir::WalkDir::new(&all)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(n) = entry.file_name().to_str() {
                all_names.insert(n.to_string());
            }
        }
    }
    for entry in walkdir::WalkDir::new(&sent)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(n) = entry.file_name().to_str() {
                assert!(
                    all_names.contains(n),
                    "已发送的 {n} 不在 全部 里 —— 子集关系被破坏"
                );
            }
        }
    }
}
