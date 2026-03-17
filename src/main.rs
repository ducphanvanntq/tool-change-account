use std::env;
use std::process;

const VERSION: &str = "1.0.0";

fn print_help() {
    println!("tool-change-account v{}", VERSION);
    println!();
    println!("USAGE:");
    println!("    tool-change-account <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    change    Thực hiện thay đổi tài khoản");
    println!("    version   Hiển thị phiên bản");
}

fn cmd_change() {
    println!("🔄 Đang thực hiện thay đổi tài khoản...");
    // TODO: thêm logic change account ở đây
    println!("✅ Hoàn tất!");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        process::exit(0);
    }

    match args[1].as_str() {
        "change" => cmd_change(),
        "version" | "--version" | "-v" => println!("{}", VERSION),
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("Error: Lệnh '{}' không tồn tại.", other);
            eprintln!("Dùng 'tool-change-account help' để xem danh sách lệnh.");
            process::exit(1);
        }
    }
}
