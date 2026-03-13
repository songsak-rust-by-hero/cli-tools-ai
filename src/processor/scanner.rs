use crate::models::FileRecord;
use crate::processor::cleaner;
use sha2::{Digest, Sha256};
use std::fs;
use walkdir::WalkDir;

pub fn scan_project(dir: &str) -> Vec<FileRecord> {
    let mut results: Vec<FileRecord> = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().map_or(true, |ext| ext != "rs") {
            continue;
        }
        if path.to_string_lossy().contains("target") {
            continue;
        }

        if let Ok(raw_content) = fs::read_to_string(path) {
            let cleaned = cleaner::trim_fat(&raw_content);

            let mut hasher = Sha256::new();
            hasher.update(cleaned.as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            results.push(FileRecord {
                id: None,
                path: path.to_string_lossy().into_owned(),
                hash,
                content: cleaned,
            });
        }
    }

    results
}

// เช็ค hash ก่อน — return เฉพาะไฟล์ที่เปลี่ยนแปลง
pub fn scan_changed<F>(dir: &str, is_changed: F) -> Vec<FileRecord>
where
    F: Fn(&str, &str) -> bool,
{
    scan_project(dir)
        .into_iter()
        .filter(|f| is_changed(&f.path, &f.hash))
        .collect()
}
