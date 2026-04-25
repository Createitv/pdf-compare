use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize, Clone)]
pub struct CopyFailure {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct CopyReport {
    pub copied: usize,
    pub skipped: usize,
    pub failed: Vec<CopyFailure>,
}

pub enum CopyOutcome {
    Copied,
    Skipped,
    Failed(String),
}

/// Copy a single file into out_dir, skipping if a same-named file already exists.
pub fn copy_one(src: &str, out_dir: &Path) -> CopyOutcome {
    let src_path = Path::new(src);
    let name = match src_path.file_name().and_then(|s| s.to_str()) {
        Some(n) => n,
        None => return CopyOutcome::Failed("invalid filename".into()),
    };
    let dest = out_dir.join(name);
    if dest.exists() {
        return CopyOutcome::Skipped;
    }
    match std::fs::copy(src_path, &dest) {
        Ok(_) => CopyOutcome::Copied,
        Err(e) => CopyOutcome::Failed(e.to_string()),
    }
}

/// Copy multiple files. Calls `on_progress(done, total, current_name)` after each file.
pub fn copy_files<F: FnMut(usize, usize, &str)>(
    files: &[String],
    out_dir: &Path,
    mut on_progress: F,
) -> std::io::Result<CopyReport> {
    std::fs::create_dir_all(out_dir)?;

    let total = files.len();
    let mut copied = 0usize;
    let mut skipped = 0usize;
    let mut failed: Vec<CopyFailure> = Vec::new();

    for (i, src) in files.iter().enumerate() {
        let name = Path::new(src)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        match copy_one(src, out_dir) {
            CopyOutcome::Copied => copied += 1,
            CopyOutcome::Skipped => skipped += 1,
            CopyOutcome::Failed(err) => failed.push(CopyFailure {
                path: src.clone(),
                error: err,
            }),
        }
        on_progress(i + 1, total, name);
    }

    Ok(CopyReport {
        copied,
        skipped,
        failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_skips_existing() {
        let tmp = std::env::temp_dir().join(format!("pdfcmp-copy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let src_dir = tmp.join("src");
        let out_dir = tmp.join("out");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&out_dir).unwrap();

        let a = src_dir.join("a.PDF");
        let b = src_dir.join("b.PDF");
        std::fs::write(&a, b"x").unwrap();
        std::fs::write(&b, b"y").unwrap();
        std::fs::write(out_dir.join("a.PDF"), b"existing").unwrap();

        let mut events = Vec::new();
        let report = copy_files(
            &[
                a.to_string_lossy().into_owned(),
                b.to_string_lossy().into_owned(),
            ],
            &out_dir,
            |done, total, name| events.push((done, total, name.to_string())),
        )
        .unwrap();

        assert_eq!(report.copied, 1);
        assert_eq!(report.skipped, 1);
        assert_eq!(report.failed.len(), 0);
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].0, 2);
        assert_eq!(events[1].1, 2);

        let preserved = std::fs::read(out_dir.join("a.PDF")).unwrap();
        assert_eq!(preserved, b"existing");
    }
}
