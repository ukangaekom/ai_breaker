use colored::*;
use std::io::{self, Write};
use std::time::Instant;
use crate::database::{ChatMessage, ChatLog, DatabaseManager, ModelInfo};
use crate::model::exec_model_chat;

pub async fn start() {
    println!("\n{}", "=== AI Breaker Interactive Session Setup ===".cyan().bold());

    // Initialize Database
    let db = match DatabaseManager::open_default() {
        Ok(manager) => {
            println!("{} Connected to embedded redb database: {}", "[db]".green().bold(), manager.path().display());
            Some(manager)
        }
        Err(err) => {
            println!("{} Failed to open redb database: {}", "[warning]".yellow().bold(), err);
            None
        }
    };

    // 1. Select Model Name
    print!("{} ", "Enter target model name [default: gemini-1.5-flash]:".yellow().bold());
    io::stdout().flush().unwrap();
    let mut model_input = String::new();
    io::stdin().read_line(&mut model_input).unwrap();
    let model_name = model_input.trim();
    let model_name = if model_name.is_empty() {
        "gemini-1.5-flash"
    } else {
        model_name
    };

    // Register model info in DB if DB is active
    if let Some(ref db_mgr) = db {
        let provider = model_name
            .split('/')
            .next()
            .unwrap_or(model_name)
            .to_string();

        let model_info = ModelInfo {
            name: model_name.to_string(),
            provider,
            description: Some("Interactive prompt evaluation target".into()),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        if let Err(e) = db_mgr.save_model(&model_info) {
            println!("{} Failed to save model info: {}", "[db error]".red(), e);
        }
    }

    // 2. Set System Prompt / Instructions
    print!("{} ", "Enter temporary System Prompt/Instruction (leave empty for none):".yellow().bold());
    io::stdout().flush().unwrap();
    let mut sys_input = String::new();
    io::stdin().read_line(&mut sys_input).unwrap();
    let sys_prompt_str = sys_input.trim();
    let system_instruction = if sys_prompt_str.is_empty() {
        None
    } else {
        Some(sys_prompt_str.to_string())
    };

    println!("\n{}", "------------------------------------------------------------".bright_black());
    println!("{} {}", "Model Target:".bold(), model_name.green());
    if let Some(ref sys) = system_instruction {
        println!("{} {}", "System Instruction:".bold(), sys.magenta());
    } else {
        println!("{} {}", "System Instruction:".bold(), "<None>".bright_black());
    }
    println!("Type {} or {} to end the evaluation session.\n", "'exit'".yellow(), "'quit'".yellow());

    let mut message_history: Vec<ChatMessage> = Vec::new();
    let mut input = String::new();

    loop {
        print!("{}", "Breaker > ".green().bold());
        io::stdout().flush().unwrap();

        input.clear();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let prompt = input.trim();

                if prompt.eq_ignore_ascii_case("exit") || prompt.eq_ignore_ascii_case("quit") {
                    println!("\n{}", "Shutting down evaluation session...".bright_black());
                    break;
                }

                if prompt.is_empty() {
                    continue;
                }

                println!("{}", "Model > Thinking..........".bright_yellow());

                let start_time = Instant::now();
                let result = exec_model_chat(
                    model_name,
                    system_instruction.as_deref(),
                    &message_history,
                    prompt,
                ).await;
                let latency_ms = start_time.elapsed().as_millis() as u64;

                let response_text = match result {
                    Ok(text) => text,
                    Err(err) => {
                        let err_msg = format!("[Error executing model chat: {}]", err);
                        println!("{} {}\n", "Model Error >".red().bold(), err_msg);
                        err_msg
                    }
                };

                println!("{} {}\n", "Model Response >".yellow().bold(), response_text.cyan());

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Append to message history
                message_history.push(ChatMessage {
                    role: "user".into(),
                    content: prompt.to_string(),
                    timestamp: now,
                });
                message_history.push(ChatMessage {
                    role: "assistant".into(),
                    content: response_text.clone(),
                    timestamp: now,
                });

                // Persist chat log to redb database
                if let Some(ref db_mgr) = db {
                    let log = ChatLog::new(
                        model_name,
                        system_instruction.clone(),
                        prompt,
                        &response_text,
                        message_history.clone(),
                        Some(latency_ms),
                    );

                    match db_mgr.save_chat_log(&log) {
                        Ok(_) => {
                            println!("{} Session log recorded in redb [ID: {} | Latency: {}ms]",
                                "[redb log]".bright_blue(), log.id.yellow(), latency_ms);
                        }
                        Err(e) => {
                            println!("{} Failed to persist log: {}", "[redb error]".red(), e);
                        }
                    }
                }
            }

            Err(error) => {
                println!("Error reading input: {}", error);
                break;
            }
        }
    }
}
