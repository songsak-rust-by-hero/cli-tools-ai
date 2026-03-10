// src/models.rs

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub id: Option<i64>,
    pub path: String,
    pub hash: String,
    pub content: String, // โค้ดที่รีดไขมันแล้ว
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub role: String, // "user" หรือ "assistant"
    pub content: String,
    pub timestamp: Option<String>,
}
