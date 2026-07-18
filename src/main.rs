mod chat;

use clap::{Parser, Subcommand};
use colored::*; // For colored output
use crate::chat::chat::start;




#[derive(Parser)]
#[command(
    name = "ai_breaker",
    author = "Ekomabasi Ukanga", 
    version = "1.0.0",
    about = "A tool ",
    long_about = None)
]


struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
}


#[derive(Subcommand)]
enum Commands {
    Settings,
    Chat
}



#[tokio::main]
async fn main() {

    println!("{}", "Prompt Instructions".white().bold());
    println!("{}", "Hello, world!".red().bold());
    println!("{}","AI response will be".yellow().bold());


    let cli = Cli::parse();

    match &cli.command{

        Some(Commands::Settings) => {
            println!("Settings command selected");


        },
        Some(Commands::Chat) => {
            println!("Chat command selected");

            start().await;
        },

        _=> {
            println!("Default command selected --help")
        }

    }


}
