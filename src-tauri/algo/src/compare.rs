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
    pub anomaly_count: usize,
}

#[derive(Debug, Serialize)]
pub struct CompareResult {
    pub missing: Vec<FileEntry>,
    pub anomalies: Vec<FileEntry>,
    pub stats: Stats,
}

pub fn extract_key(filename: &str) -> Option<String> {
    KEY_RE.captures(filename).map(|c| c[1].to_string())
}

pub fn extract_prefix(key: &str) -> Option<String> {
    let mut iter = key.splitn(3, '-');
    let a = iter.next()?;
    let b = iter.next()?;
    Some(format!("{a}-{b}"))
}

fn is_pdf(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".pdf")
}

struct ScanIndex {
    keys: HashSet<String>,
    prefixes: HashSet<String>,
    by_key: BTreeMap<String, Vec<FileEntry>>,
    total: usize,
}

fn scan(dir: &Path) -> ScanIndex {
    let mut keys = HashSet::new();
    let mut prefixes = HashSet::new();
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
        if let Some(prefix) = extract_prefix(&key) {
            prefixes.insert(prefix);
        }
        keys.insert(key.clone());
        by_key.entry(key.clone()).or_default().push(FileEntry {
            key,
            full_name: name.to_string(),
            abs_path: entry.path().to_string_lossy().to_string(),
        });
    }

    ScanIndex {
        keys,
        prefixes,
        by_key,
        total,
    }
}

pub fn compare_dirs(all_dir: &Path, sent_dir: &Path) -> CompareResult {
    let left = scan(all_dir);
    let right = scan(sent_dir);

    let mut missing: Vec<FileEntry> = Vec::new();
    let mut anomalies: Vec<FileEntry> = Vec::new();

    for (key, files) in &left.by_key {
        let prefix = extract_prefix(key);
        let prefix_seen = prefix
            .as_ref()
            .map(|p| right.prefixes.contains(p))
            .unwrap_or(false);
        let key_seen = right.keys.contains(key);

        if !prefix_seen {
            for f in files {
                anomalies.push(f.clone());
            }
        } else if !key_seen {
            for f in files {
                missing.push(f.clone());
            }
        }
    }

    let stats = Stats {
        total_left: left.total,
        total_right: right.total,
        missing_count: missing.len(),
        anomaly_count: anomalies.len(),
    };

    CompareResult {
        missing,
        anomalies,
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
        assert_eq!(
            extract_key("40-03-018--32120-50-LD-22030-A21K-CJ40(031)_Sht_3.PDF"),
            Some("40-03-018".to_string())
        );
    }

    #[test]
    fn extract_key_no_match() {
        assert_eq!(extract_key("readme.pdf"), None);
        assert_eq!(extract_key("39-03-001-x.pdf"), None);
    }

    #[test]
    fn extract_prefix_basic() {
        assert_eq!(extract_prefix("40-03-001"), Some("40-03".to_string()));
        assert_eq!(extract_prefix("40-99-12"), Some("40-99".to_string()));
    }

    #[test]
    fn compare_finds_missing_and_anomalies() {
        let tmp = tempdir();
        let all = tmp.join("all");
        let sent = tmp.join("sent");
        std::fs::create_dir_all(&all).unwrap();
        std::fs::create_dir_all(&sent).unwrap();

        touch(&all, "40-03-001--x_Sht_1.PDF");
        touch(&all, "40-03-001--x_Sht_2.PDF");
        touch(&sent, "40-03-001--x_Sht_1.PDF");

        touch(&all, "40-03-002--y_Sht_1.PDF");
        touch(&all, "40-99-001--z_Sht_1.PDF");

        let r = compare_dirs(&all, &sent);
        let missing_keys: HashSet<_> = r.missing.iter().map(|e| e.key.clone()).collect();
        let anomaly_keys: HashSet<_> = r.anomalies.iter().map(|e| e.key.clone()).collect();

        assert!(missing_keys.contains("40-03-002"));
        assert!(!missing_keys.contains("40-03-001"));
        assert!(anomaly_keys.contains("40-99-001"));
        assert_eq!(r.stats.total_left, 4);
        assert_eq!(r.stats.total_right, 1);
    }

    fn tempdir() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("pdfcmp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn touch(dir: &std::path::Path, name: &str) {
        std::fs::write(dir.join(name), b"%PDF-1.4\n%%EOF").unwrap();
    }
}
