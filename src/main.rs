mod commands;
mod config;
mod oauth;
mod paths;
mod protobuf;
mod token;

use clap::{Parser, Subcommand};

const VERSION: &str = "1.0.0";

#[derive(Parser)]
#[command(name = "tool-change-account", version = VERSION, about = "Tool quản lý account Antigravity")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Hiển thị thông tin account hiện tại
    Info,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info => commands::cmd_info().await,
    }
}
