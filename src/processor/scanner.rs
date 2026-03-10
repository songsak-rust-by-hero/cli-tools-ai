use crate::models::FileRecord;
use crate::processor::cleaner;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn scan_project(dir: &str) -> Vec<FileRecord> {
    let mut results = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // สแกนเฉพาะไฟล์ .rs และข้ามโฟลเดอร์ target
        if path.is_file()
            && path.extension().map_or(false, |ext| ext == "rs")
            && !path.to_string_lossy().contains("target")
        {
            if let Ok(raw_content) = fs::read_to_string(path) {
                // 1. รีดไขมันโค้ด
                let cleaned = cleaner::trim_fat(&raw_content);

                // 2. ทำ Hash (ใช้เนื้อหาที่คลีนแล้ว เพื่อดูความต่างของ Logic)
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
    }
    results
}
