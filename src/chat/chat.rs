use colored::*;
use std::io::{self, Write};

pub async fn start() {
    let mut input = String::new();

    loop {
        print!("{}", "Tester >".yellow().bold());
        io::stdout().flush().unwrap();

        input.clear();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let text = input.trim();

                if text.eq_ignore_ascii_case("exit") || text.eq_ignore_ascii_case("quit") {
                    println!("\n\n{}", "shutting down agent runtime.....".bright_black());
                    break;
                }

                if text.is_empty() {
                    continue;
                }

                println!("{}", "Model > Thinking..........".bright_yellow());

                // Color the user's input and the model's response separately
                println!("{} {}\n\n", "You >".green().bold(), text.green());
                println!("{} {}\n\n", "Model >".yellow().bold(), text.cyan());
            }

            Err(error) => {
                println!("Error reading inputs: {}", error);
                break;
            }
        }
    }
}
