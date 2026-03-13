use crate::brain::{api, memory};
use crate::db::DbManager;
use crate::error::AppError;
use crate::processor::scanner;
use std::io::Write;

pub async fn run(
    db: &mut DbManager,
    api_key: &str,
    project_dir: &str,
) -> Result<(), AppError> {
    println!("💬 Chat mode (type 'exit' to quit, 'task: <desc>' to set current task)");

    let mut input = String::new();
    let mut current_task: Option<String> = None;

    loop {
        print!("\nYou: ");
        std::io::stdout().flush()?;
        input.clear();
        std::io::stdin().read_line(&mut input)?;
        let msg = input.trim();

        if msg == "exit" {
            println!("💾 Saving summary...");
            let history = db
                .get_recent_chats(20)?
                .iter()
                .rev()
                .map(|c| format!("{}: {}", c.role, c.content))
                .collect::<Vec<_>>()
                .join("\n");

            if !history.is_empty() {
                match api::summarize_chats(&history, api_key).await {
                    Ok(summary) => db.save_summary(&summary)?,
                    Err(e) => eprintln!("⚠️ Could not save summary: {}", e),
                }
            }
            break;
        }

        // ── pin task ──────────────────────────────────────────────────────
        if let Some(task_desc) = msg.strip_prefix("task:") {
            current_task = Some(task_desc.trim().to_string());
            println!("📌 Task set: {}", task_desc.trim());
            continue;
        }

        // ── smart scan ────────────────────────────────────────────────────
        let all = scanner::scan_project(project_dir);
        let changed: Vec<_> = all
            .into_iter()
            .filter(|f| {
                db.get_file_hash(&f.path)
                    .ok()
                    .flatten()
                    .map_or(true, |h| h != f.hash)
            })
            .collect();

        if !changed.is_empty() {
            db.upsert_files(&changed)?;
        }

        db.add_chat("user", msg)?;

        let context = memory::prepare_context(
            db,
            5,
            current_task.as_deref(),
            Some(project_dir),
            Some(msg),  // ← ส่ง question เพื่อหา relevant files
        )?;
        let final_prompt = format!("Context:\n{}\n\nQuestion: {}", context, msg);

        println!("🤖 Thinking...");
        match api::call_ai(&final_prompt, api_key).await {
            Ok(response) => {
                println!("\nAssistant:");
                termimad::print_text(&response);
                db.add_chat("assistant", &response)?;
            }
            Err(e) => eprintln!("❌ Error: {}", e),
        }
    }

    Ok(())
}
