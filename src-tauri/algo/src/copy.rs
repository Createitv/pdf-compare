use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct CopyFailure {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct CopyReport {
    pub copied: usize,
    pub failed: Vec<CopyFailure>,
}

fn unique_dest(out_dir: &Path, file_name: &str) -> PathBuf {
    let candidate = out_dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    for n in 2..10_000 {
        let alt = out_dir.join(format!("{stem}_{n}{ext}"));
        if !alt.exists() {
            return alt;
        }
    }
    candidate
}

pub fn copy_files(files: &[String], out_dir: &Path) -> std::io::Result<CopyReport> {
    std::fs::create_dir_all(out_dir)?;

    let mut copied = 0usize;
    let mut failed: Vec<CopyFailure> = Vec::new();

    for src in files {
        let src_path = Path::new(src);
        let name = match src_path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => {
                failed.push(CopyFailure {
                    path: src.clone(),
                    error: "invalid filename".into(),
                });
                continue;
            }
        };
        let dest = unique_dest(out_dir, name);
        match std::fs::copy(src_path, &dest) {
            Ok(_) => copied += 1,
            Err(e) => failed.push(CopyFailure {
                path: src.clone(),
                error: e.to_string(),
            }),
        }
    }

    Ok(CopyReport { copied, failed })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_with_collision_rename() {
        let tmp = std::env::temp_dir().join(format!("pdfcmp-copy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let src_dir = tmp.join("src");
        let out_dir = tmp.join("out");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&out_dir).unwrap();

        let a = src_dir.join("a.PDF");
        std::fs::write(&a, b"x").unwrap();
        std::fs::write(out_dir.join("a.PDF"), b"existing").unwrap();

        let report = copy_files(&[a.to_string_lossy().into_owned()], &out_dir).unwrap();
        assert_eq!(report.copied, 1);
        assert!(out_dir.join("a_2.PDF").exists());
    }
}
