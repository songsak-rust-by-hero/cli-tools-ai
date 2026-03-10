use crate::error::AppError;
use crate::models::{ChatMessage, FileRecord};
use rusqlite::{Connection, Result, params};

pub struct DbManager {
    conn: Connection,
}

impl DbManager {
    pub fn new(path: &str) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(DbManager { conn })
    }

    fn init_schema(conn: &Connection) -> Result<(), AppError> {
        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                hash TEXT NOT NULL,
                content TEXT
            );
            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS summaries (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL
            );
            COMMIT;",
        )?;
        Ok(())
    }

    pub fn get_recent_chats(&self, limit: usize) -> Result<Vec<ChatMessage>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, role, content, timestamp FROM conversations ORDER BY id DESC LIMIT ?1",
        )?;

        let msgs = stmt
            .query_map([limit as i64], |row| {
                Ok(ChatMessage {
                    id: row.get(0)?,
                    role: row.get(1)?,
                    content: row.get(2)?,
                    timestamp: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(msgs)
    }

    // ✅ แก้ไข: ตัด [path] ที่เกินออก และใช้รูปแบบที่ถูกต้อง
    pub fn get_file_hash(&self, path: &str) -> Result<Option<String>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT hash FROM files WHERE path = ?1")?;

        // query_row รับพารามิเตอร์แค่ 2 ตัวคือ params และ closure
        let hash: Option<String> = stmt.query_row([path], |row| row.get(0)).ok();
        Ok(hash)
    }

    pub fn upsert_files(&mut self, files: &[FileRecord]) -> Result<(), AppError> {
        let tx = self.conn.transaction()?;

        for file in files {
            tx.execute(
                "INSERT OR REPLACE INTO files (path, hash, content) VALUES (?1, ?2, ?3)",
                params![file.path, file.hash, file.content],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_latest_summary(&self) -> Result<Option<String>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT content FROM summaries ORDER BY id DESC LIMIT 1")?;
        let summary: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
        Ok(summary)
    }

    pub fn save_summary(&self, content: &str) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO summaries (id, content) VALUES (1, ?1)",
            params![content],
        )?;
        Ok(())
    }

    pub fn get_all_files(&self) -> Result<Vec<FileRecord>, AppError> {
        let mut stmt = self.conn.prepare("SELECT path, hash, content FROM files")?;
        let rows = stmt.query_map([], |row| {
            Ok(FileRecord {
                id: None,
                path: row.get(0)?,
                hash: row.get(1)?,
                content: row.get(2)?,
            })
        })?;

        let mut files = Vec::new();
        for row in rows {
            files.push(row?);
        }
        Ok(files)
    }

    pub fn add_chat(&self, role: &str, content: &str) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO conversations (role, content) VALUES (?1, ?2)",
            params![role, content],
        )?;
        Ok(())
    }

}
