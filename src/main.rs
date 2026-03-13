mod brain;
mod commands;
mod db;
mod error;
mod models;
mod processor;

use clap::{Parser, Subcommand};
use db::DbManager;
use dotenv::dotenv;
use std::env;

#[derive(Parser)]
#[command(name = "brain")]
#[command(about = "AI Code Assistant CLI for Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to project directory (default: current dir)
    #[arg(short, long, default_value = ".")]
    project: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan and sync project files to DB
    Sync,
    /// Ask a one-shot question
    Ask { question: String },
    /// Enter interactive chat mode
    Chat,
    /// Show current project map/context
    Map,
    /// Run cargo check and show errors
    Check,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let cli = Cli::parse();
    let mut db = DbManager::new("brain.db")?;

    let api_key = env::var("AI_API_KEY").unwrap_or_else(|_| {
        eprintln!("❌ AI_API_KEY not set. Add it to your .env file.");
        std::process::exit(1);
    });

    let project_dir = &cli.project;

    match cli.command {
        Commands::Sync => {
            println!("🔍 Scanning files...");
            let files = processor::scanner::scan_project(project_dir);
            db.upsert_files(&files)?;
            println!("✅ Updated {} file(s).", files.len());
        }

        Commands::Ask { question } => {
            commands::ask::run(&mut db, &api_key, &question, project_dir, None).await?;
        }

        Commands::Chat => {
            commands::chat::run(&mut db, &api_key, project_dir).await?;
        }

        Commands::Map => {
            let context = brain::memory::prepare_context(&mut db, 0, None, None,None)?;
            println!("{}", context);
        }

        Commands::Check => {
            println!("🔧 Running cargo check...");
            let ctx = brain::error_context::run_cargo_check(project_dir);
            println!("{}", brain::error_context::format_for_context(&ctx));
        }
    }

    Ok(())
}
