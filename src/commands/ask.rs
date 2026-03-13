use crate::brain::{api, memory};
use crate::db::DbManager;
use crate::error::AppError;
use crate::processor::scanner;

pub async fn run(
    db: &mut DbManager,
    api_key: &str,
    question: &str,
    project_dir: &str,
    task: Option<&str>,
) -> Result<(), AppError> {
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
        println!("📂 Updated {} changed file(s)", changed.len());
        db.upsert_files(&changed)?;
    }

    db.add_chat("user", question)?;

    let context = memory::prepare_context(
        db,
        5,
        task,
        Some(project_dir),
        Some(question),  // ← ส่ง question เพื่อหา relevant files
    )?;
    let final_prompt = format!("Context:\n{}\n\nQuestion: {}", context, question);

    println!("🤖 Thinking...");
    let response = api::call_ai(&final_prompt, api_key).await?;
    println!("\nAssistant:");
    termimad::print_text(&response);

    db.add_chat("assistant", &response)?;
    Ok(())
}
