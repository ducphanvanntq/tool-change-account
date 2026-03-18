mod commands;
mod config;
mod oauth;
mod paths;
mod protobuf;
mod token;

const VERSION: &str = "1.0.0";

fn print_help() {
    println!("tool-change-account v{}", VERSION);
    println!();
    println!("USAGE:");
    println!("    tool-change-account <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    info      Hiển thị thông tin account hiện tại");
    println!("    version   Hiển thị phiên bản");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(0);
    }

    match args[1].as_str() {
        "info" => commands::cmd_info().await,
        "version" | "--version" | "-v" => println!("{}", VERSION),
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("Error: '{}' is not a valid command.", other);
            eprintln!("Use 'tool-change-account help' to see available commands.");
            std::process::exit(1);
        }
    }
}
