use crate::brain::error_context;
use crate::db::DbManager;
use crate::error::AppError;
use crate::processor::skeleton;

fn find_relevant_files(question: &str, files: &[crate::models::FileRecord]) -> Vec<String> {
    let q = question.to_lowercase();
    let mut relevant: Vec<String> = Vec::new();

    for file in files {
        let filename = file.path.to_lowercase();
        let file_stem = filename
            .split('/')
            .last()
            .unwrap_or("")
            .replace(".rs", "");

        // ตรงชื่อไฟล์
        if q.contains(&file_stem) || q.contains(&file.path.to_lowercase()) {
            relevant.push(format!("=== {} ===\n{}", file.path, file.content));
            continue;
        }

        // ตรงชื่อ fn/struct ในไฟล์ — เช็คทั้ง signature และ code
        let sigs = skeleton::extract_signatures(&file.content);
        let mut matched = false;

        for sig_line in sigs.lines() {
            // ดึงชื่อ fn/struct ออกมา เช่น "pub fn scan_project" → "scan_project"
            let sig = sig_line.trim().trim_end_matches("...");
            let words: Vec<&str> = sig.split_whitespace().collect();
            for word in &words {
                let w = word.to_lowercase();
                // ข้าม keyword สั้นๆ
                if w.len() <= 2 || matches!(w.as_str(), "fn" | "pub" | "let" | "mut" | "impl" | "struct" | "enum") {
                    continue;
                }
                if q.contains(&w) {
                    matched = true;
                    break;
                }
            }
            if matched {
                break;
            }
        }

        if matched {
            relevant.push(format!("=== {} ===\n{}", file.path, file.content));
        }
    }

    relevant
}

pub fn prepare_context(
    db: &mut DbManager,
    window_size: usize,
    task: Option<&str>,
    project_dir: Option<&str>,
    question: Option<&str>,
) -> Result<String, AppError> {
    let mut prompt = String::new();

    if let Some(t) = task {
        prompt.push_str(&format!("### CURRENT TASK ###\n{}\n\n", t));
    }

    if let Some(dir) = project_dir {
        let err_ctx = error_context::run_cargo_check(dir);
        prompt.push_str(&error_context::format_for_context(&err_ctx));
        prompt.push('\n');
    }

    let summary: String = db
        .get_latest_summary()?
        .unwrap_or_else(|| "No previous summary.".to_string());
    prompt.push_str(&format!("### OVERALL PROJECT SUMMARY ###\n{}\n\n", summary));

    let all_files = db.get_all_files()?;

    if let Some(q) = question {
        let relevant = find_relevant_files(q, &all_files);
        if !relevant.is_empty() {
            prompt.push_str("### RELEVANT FILE CONTENTS ###\n");
            for content in &relevant {
                prompt.push_str(content);
                prompt.push_str("\n\n");
            }
        }
    }

    prompt.push_str("### PROJECT STRUCTURE (SKELETON) ###\n");
    for file in &all_files {
        let sigs = skeleton::extract_signatures(&file.content);
        if !sigs.is_empty() {
            prompt.push_str(&format!("File: {}\n{}\n", file.path, sigs));
        }
    }
    prompt.push('\n');

    if window_size > 0 {
        prompt.push_str("### RECENT CONVERSATION ###\n");
        let chats = db.get_recent_chats(window_size)?;
        for chat in chats.iter().rev() {
            prompt.push_str(&format!("{}: {}\n", chat.role, chat.content));
        }
    }

    Ok(prompt)
}
