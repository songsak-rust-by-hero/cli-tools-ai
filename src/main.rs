mod brain;
mod db;
mod error;
mod models;
mod processor;

use clap::{Parser, Subcommand};
use db::DbManager;
use std::env;
use std::io::Write;

#[derive(Parser)]
#[command(name = "brain")]
#[command(about = "AI Code Assistant CLI for Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// สแกนโปรเจกต์และอัปเดตฐานข้อมูล
    Sync,
    /// ถามคำถาม AI เกี่ยวกับโปรเจกต์ (จบในคำสั่งเดียว)
    Ask { question: String },
    /// เข้าสู่โหมดแชทโต้ตอบ
    Chat,
    /// แสดงโครงสร้างโปรเจกต์ (Skeleton) ที่ระบบเห็น
    Map,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut db = DbManager::new("brain.db")?;
    let api_key = env::var("AI_API_KEY").unwrap_or_else(|_| "your_key_here".to_string());

    match cli.command {
        Commands::Sync => {
            println!("🔍 Scanning files...");
            let files = processor::scanner::scan_project("./src");
            db.upsert_files(&files)?;
            println!("✅ Updated {} files in database.", files.len());
        }

        Commands::Ask { question } => {
            // 1. Sync อัตโนมัติก่อนถาม
            let files = processor::scanner::scan_project("./src");
            db.upsert_files(&files)?;

            // 2. บันทึกคำถาม
            db.add_chat("user", &question)?;

            // 3. เตรียม Context และเรียก AI
            let context = brain::memory::prepare_context(&mut db, 5)?;
            let final_prompt = format!("Context:\n{}\n\nQuestion: {}", context, question);

            println!("🤖 Thinking...");
            let response = brain::api::call_ai(&final_prompt, &api_key).await?;

            println!("\nAssistant: {}", response);
            db.add_chat("assistant", &response)?;
        }

        // ใน src/main.rs ส่วน match cli.command
        Commands::Chat => {
            println!("💬 Entering interactive mode (type 'exit' to quit)");
            let mut input = String::new();
            loop {
                print!("\nYou: ");
                std::io::Write::flush(&mut std::io::stdout())?;
                input.clear();
                std::io::stdin().read_line(&mut input)?;
                let msg = input.trim();

                if msg == "exit" {
                    // บันทึก summary ก่อนออก
                    println!("💾 Saving summary...");
                    let history = db.get_recent_chats(20)?
                        .iter().rev()
                        .map(|c| format!("{}: {}", c.role, c.content))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !history.is_empty() {
                        match brain::api::summarize_chats(&history, &api_key).await {
                            Ok(summary) => { db.save_summary(&summary)?; }
                            Err(e) => eprintln!("⚠️ Could not save summary: {}", e),
                        }
                    }
                    break;
                }

                // ทำกระบวนการเดียวกับ Ask แต่ทำซ้ำใน Loop
                let files = processor::scanner::scan_project("./src");
                db.upsert_files(&files)?;
                db.add_chat("user", msg)?;

                let context = brain::memory::prepare_context(&mut db, 5)?;
                let final_prompt = format!("Context:\n{}\n\nQuestion: {}", context, msg);

                println!("🤖 Thinking...");
                match brain::api::call_ai(&final_prompt, &api_key).await {
                    Ok(response) => {
                        println!("\nAssistant: {}", response);
                        db.add_chat("assistant", &response)?;
                    }
                    Err(e) => eprintln!("❌ Error: {}", e),
                }
            }
        }

        Commands::Map => {
            let context = brain::memory::prepare_context(&mut db, 0)?;
            println!("{}", context);
        }
    }

    Ok(())
}
