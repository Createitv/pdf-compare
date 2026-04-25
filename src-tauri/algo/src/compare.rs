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
    keys: HashSet<String>,
    by_key: BTreeMap<String, Vec<FileEntry>>,
    total: usize,
}

fn scan(dir: &Path) -> ScanIndex {
    let mut keys = HashSet::new();
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
        keys.insert(key.clone());
        by_key.entry(key.clone()).or_default().push(FileEntry {
            key,
            full_name: name.to_string(),
            abs_path: entry.path().to_string_lossy().to_string(),
        });
    }

    ScanIndex {
        keys,
        by_key,
        total,
    }
}

pub fn compare_dirs(all_dir: &Path, sent_dir: &Path) -> CompareResult {
    let left = scan(all_dir);
    let right = scan(sent_dir);

    let mut missing: Vec<FileEntry> = Vec::new();
    for (key, files) in &left.by_key {
        if !right.keys.contains(key) {
            for f in files {
                missing.push(f.clone());
            }
        }
    }

    let mut extra: Vec<FileEntry> = Vec::new();
    for (key, files) in &right.by_key {
        if !left.keys.contains(key) {
            for f in files {
                extra.push(f.clone());
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

        touch(&all, "40-03-001--x_Sht_1.PDF");
        touch(&all, "40-03-001--x_Sht_2.PDF");
        touch(&sent, "40-03-001--x_Sht_1.PDF");
        touch(&all, "40-03-002--y_Sht_1.PDF");
        touch(&all, "40-99-001--z_Sht_1.PDF");
        // extra in sent, not in all
        touch(&sent, "40-77-007--rogue_Sht_1.PDF");

        let r = compare_dirs(&all, &sent);
        let missing_keys: HashSet<_> = r.missing.iter().map(|e| e.key.clone()).collect();
        let extra_keys: HashSet<_> = r.extra.iter().map(|e| e.key.clone()).collect();

        assert!(missing_keys.contains("40-03-002"));
        assert!(missing_keys.contains("40-99-001"));
        assert!(!missing_keys.contains("40-03-001"));
        assert_eq!(r.stats.missing_count, 2);

        assert!(extra_keys.contains("40-77-007"));
        assert!(!extra_keys.contains("40-03-001"));
        assert_eq!(r.stats.extra_count, 1);

        assert_eq!(r.stats.total_left, 4);
        assert_eq!(r.stats.total_right, 2);
    }
}
