mod chat;
mod database;
mod model;

use clap::{Parser, Subcommand};
use colored::*;
use crate::chat::chat::start;
use crate::database::DatabaseManager;

#[derive(Parser)]
#[command(
    name = "ai_breaker",
    author = "Ekomabasi Ukanga",
    version = "1.0.0",
    about = "Embedded LLM Security Evaluation & Jailbreak Testing Framework",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive jailbreak chat session with model & system prompt
    Chat,
    /// List all registered model names in redb database
    Models,
    /// View recorded chat logs and metadata stored in redb database
    Logs,
    /// View application settings
    Settings,
}

#[tokio::main]
async fn main() {
    println!("{}", "==================================================".cyan());
    println!("{}", "               AI BREAKER FRAMEWORK               ".cyan().bold());
    println!("{}", "==================================================".cyan());

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Settings) => {
            println!("{}", "Settings selected".green().bold());
            if let Ok(db) = DatabaseManager::open_default() {
                println!("Embedded Database Path: {}", db.path().display());
            }
        }
        Some(Commands::Chat) => {
            println!("{}", "Launching Chat Evaluation Engine...".green().bold());
            start().await;
        }
        Some(Commands::Models) => {
            println!("{}", "=== Registered Models in redb ===".yellow().bold());
            match DatabaseManager::open_default() {
                Ok(db) => match db.list_models() {
                    Ok(models) => {
                        if models.is_empty() {
                            println!("No models registered yet.");
                        } else {
                            for m in models {
                                println!(
                                    "• {} [{}] - {}",
                                    m.name.green().bold(),
                                    m.provider.magenta(),
                                    m.description.unwrap_or_default().bright_black()
                                );
                            }
                        }
                    }
                    Err(e) => println!("Error listing models: {}", e),
                },
                Err(e) => println!("Error opening database: {}", e),
            }
        }
        Some(Commands::Logs) => {
            println!("{}", "=== Chat History Logs in redb ===".yellow().bold());
            match DatabaseManager::open_default() {
                Ok(db) => match db.list_chat_logs() {
                    Ok(logs) => {
                        if logs.is_empty() {
                            println!("No chat logs recorded yet.");
                        } else {
                            for log in logs {
                                println!(
                                    "[{}] Model: {} | Prompt: \"{}\" | Latency: {}ms",
                                    log.id.cyan(),
                                    log.model_name.green(),
                                    log.prompt,
                                    log.latency_ms.unwrap_or(0)
                                );
                                if let Some(sys) = log.system_prompt {
                                    println!("   System Instruction: {}", sys.magenta());
                                }
                                println!("   Response: {}\n", log.response.bright_white());
                            }
                        }
                    }
                    Err(e) => println!("Error listing chat logs: {}", e),
                },
                Err(e) => println!("Error opening database: {}", e),
            }
        }
        None => {
            println!("{}", "Defaulting to Chat Evaluation Session...".yellow());
            println!("(Use --help to see all available commands)");
            start().await;
        }
    }
}
