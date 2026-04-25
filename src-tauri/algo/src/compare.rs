use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

static KEY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(40-(\d+)-\d+)").unwrap());

#[derive(Debug, Serialize, Clone)]
pub struct FileEntry {
    pub key: String,
    pub full_name: String,
    pub abs_path: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Stats {
    pub total_left: usize,
    pub total_right: usize,
    pub missing_count: usize,
    pub extra_count: usize,
}

#[derive(Debug, Serialize)]
pub struct CompareResult {
    pub missing: Vec<FileEntry>,
    pub extra: Vec<FileEntry>,
    pub stats: Stats,
}

pub fn extract_key(filename: &str) -> Option<String> {
    KEY_RE.captures(filename).map(|c| c[1].to_string())
}

fn is_pdf(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".pdf")
}

struct ScanIndex {
    by_key: BTreeMap<String, Vec<FileEntry>>,
    total: usize,
}

fn scan(dir: &Path) -> ScanIndex {
    let mut by_key: BTreeMap<String, Vec<FileEntry>> = BTreeMap::new();
    let mut total = 0usize;

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = match entry.file_name().to_str() {
            Some(s) => s,
            None => continue,
        };
        if !is_pdf(name) {
            continue;
        }
        total += 1;
        let Some(key) = extract_key(name) else {
            continue;
        };
        by_key.entry(key.clone()).or_default().push(FileEntry {
            key,
            full_name: name.to_string(),
            abs_path: entry.path().to_string_lossy().to_string(),
        });
    }

    ScanIndex { by_key, total }
}

/// 两层匹配：
/// 1. 优先以前三段 key (`40-XX-YYY`) 比对。
/// 2. 当左侧某 key 下有多个文件（多 sheet）时，回退到完整文件名比对，
///    避免「右侧只发了部分 sheet」时误判为已全部发送。
pub fn compare_dirs(all_dir: &Path, sent_dir: &Path) -> CompareResult {
    let left = scan(all_dir);
    let right = scan(sent_dir);

    let mut missing: Vec<FileEntry> = Vec::new();
    let mut extra: Vec<FileEntry> = Vec::new();

    // 缺失：左有右无（优先 key，多 sheet 时按全名）
    for (key, all_files) in &left.by_key {
        match right.by_key.get(key) {
            None => {
                for f in all_files {
                    missing.push(f.clone());
                }
            }
            Some(sent_files) => {
                if all_files.len() > 1 {
                    let sent_names: HashSet<&str> =
                        sent_files.iter().map(|f| f.full_name.as_str()).collect();
                    for f in all_files {
                        if !sent_names.contains(f.full_name.as_str()) {
                            missing.push(f.clone());
                        }
                    }
                }
            }
        }
    }

    // 多余：右有左无（同样两层规则）
    for (key, sent_files) in &right.by_key {
        match left.by_key.get(key) {
            None => {
                for f in sent_files {
                    extra.push(f.clone());
                }
            }
            Some(all_files) => {
                if all_files.len() > 1 {
                    let all_names: HashSet<&str> =
                        all_files.iter().map(|f| f.full_name.as_str()).collect();
                    for f in sent_files {
                        if !all_names.contains(f.full_name.as_str()) {
                            extra.push(f.clone());
                        }
                    }
                }
            }
        }
    }

    let stats = Stats {
        total_left: left.total,
        total_right: right.total,
        missing_count: missing.len(),
        extra_count: extra.len(),
    };

    CompareResult {
        missing,
        extra,
        stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_key_basic() {
        assert_eq!(
            extract_key("40-03-001--32120-100-CWR-23016-H1A20-N(031)_Sht_1.PDF"),
            Some("40-03-001".to_string())
        );
    }

    #[test]
    fn extract_key_no_match() {
        assert_eq!(extract_key("readme.pdf"), None);
        assert_eq!(extract_key("39-03-001-x.pdf"), None);
    }

    #[test]
    fn compare_finds_only_missing() {
        let tmp = std::env::temp_dir().join(format!("pdfcmp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let all = tmp.join("all");
        let sent = tmp.join("sent");
        std::fs::create_dir_all(&all).unwrap();
        std::fs::create_dir_all(&sent).unwrap();

        let touch = |dir: &Path, name: &str| {
            std::fs::write(dir.join(name), b"%PDF").unwrap();
        };

        // 多 sheet 同 key — 按全名 fallback：sent 只发了 Sht_1，Sht_2 应为缺失
        touch(&all, "40-03-001--x_Sht_1.PDF");
        touch(&all, "40-03-001--x_Sht_2.PDF");
        touch(&sent, "40-03-001--x_Sht_1.PDF");

        // 单 sheet 同 key — key 命中即可，sheet 名差异也算匹配（其实这里相同）
        touch(&all, "40-03-009--single_Sht_1.PDF");
        touch(&sent, "40-03-009--single_Sht_1.PDF");

        // key 完全不在右侧 — 全部缺失
        touch(&all, "40-03-002--y_Sht_1.PDF");
        touch(&all, "40-99-001--z_Sht_1.PDF");

        // 右侧多余（key 不在左）
        touch(&sent, "40-77-007--rogue_Sht_1.PDF");

        let r = compare_dirs(&all, &sent);
        let missing_names: HashSet<_> =
            r.missing.iter().map(|e| e.full_name.clone()).collect();
        let extra_names: HashSet<_> =
            r.extra.iter().map(|e| e.full_name.clone()).collect();

        // 应缺失：001_Sht_2 + 002 + 99-001 = 3
        assert!(missing_names.contains("40-03-001--x_Sht_2.PDF"));
        assert!(missing_names.contains("40-03-002--y_Sht_1.PDF"));
        assert!(missing_names.contains("40-99-001--z_Sht_1.PDF"));
        assert!(!missing_names.contains("40-03-001--x_Sht_1.PDF"));
        assert!(!missing_names.contains("40-03-009--single_Sht_1.PDF"));
        assert_eq!(r.stats.missing_count, 3);

        assert!(extra_names.contains("40-77-007--rogue_Sht_1.PDF"));
        assert_eq!(r.stats.extra_count, 1);

        assert_eq!(r.stats.total_left, 5);
        assert_eq!(r.stats.total_right, 3);
    }

    #[test]
    fn key_match_sufficient_when_single_sheet_in_left_even_if_full_names_differ() {
        // 左侧只有 1 个，key 命中即视为匹配，即使全名不同
        let tmp = std::env::temp_dir().join(format!("pdfcmp-single-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let all = tmp.join("all");
        let sent = tmp.join("sent");
        std::fs::create_dir_all(&all).unwrap();
        std::fs::create_dir_all(&sent).unwrap();
        std::fs::write(all.join("40-03-100--A.PDF"), b"x").unwrap();
        std::fs::write(sent.join("40-03-100--B.PDF"), b"x").unwrap();

        let r = compare_dirs(&all, &sent);
        assert_eq!(r.stats.missing_count, 0);
        assert_eq!(r.stats.extra_count, 0);
    }
}
