// src/brain/memory.rs
use crate::db::DbManager;
use crate::error::AppError;
use crate::processor::skeleton;

pub fn prepare_context(db: &mut DbManager, window_size: usize) -> Result<String, AppError> {
    let mut prompt = String::new();

    // 1. ดึงสรุปความจำระยะยาว (Long-term Summary)
    // สมมติว่าเก็บไว้ในตาราง summaries id=1
    let summary: String = db
        .get_latest_summary()?
        .unwrap_or_else(|| "No previous summary.".to_string());
    prompt.push_str(&format!("### OVERALL PROJECT SUMMARY ###\n{}\n\n", summary));

    // 2. ดึงแผนที่โปรเจกต์ (Project Map / Skeleton)
    // เพื่อให้ AI เห็นว่าเรามีไฟล์อะไรบ้าง และมีฟังก์ชันอะไรบ้าง
    prompt.push_str("### PROJECT STRUCTURE (SKELETON) ###\n");
    let all_files = db.get_all_files()?; // เราต้องไปเพิ่มฟังก์ชันนี้ใน db.rs
    for file in all_files {
        let sigs = skeleton::extract_signatures(&file.content);
        prompt.push_str(&format!("File: {}\n{}\n", file.path, sigs));
    }
    prompt.push_str("\n");

    // 3. ดึงประวัติการคุยล่าสุด (Sliding Window)
    prompt.push_str("### RECENT CONVERSATION ###\n");
    let chats = db.get_recent_chats(window_size)?;
    // กลับด้านจากเก่าไปใหม่
    for chat in chats.iter().rev() {
        prompt.push_str(&format!("{}: {}\n", chat.role, chat.content));
    }

    Ok(prompt)
}
