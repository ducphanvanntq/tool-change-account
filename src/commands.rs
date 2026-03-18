use crate::{oauth, paths, token};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

fn create_spinner(msg: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

pub async fn cmd_info() {
    // 1. Check data directory
    let sp = create_spinner("Kiểm tra thư mục dữ liệu...");
    let data_dir = paths::get_antigravity_data_dir();
    if !data_dir.exists() {
        sp.finish_and_clear();
        eprintln!("{} Antigravity chưa được cài đặt hoặc chưa từng mở.", "✘".red().bold());
        eprintln!("  {} Không tìm thấy thư mục: {}", "→".dimmed(), data_dir.display());
        return;
    }
    sp.finish_and_clear();

    // 2. Check DB
    let sp = create_spinner("Kiểm tra database...");
    let db_path = match paths::get_db_path() {
        Ok(p) if p.exists() => p,
        Ok(_) => {
            sp.finish_and_clear();
            eprintln!("{} Antigravity đã cài nhưng chưa đăng nhập (state.vscdb không tồn tại).", "✘".red().bold());
            return;
        }
        Err(e) => {
            sp.finish_and_clear();
            eprintln!("{} {}", "✘".red().bold(), e);
            return;
        }
    };
    sp.finish_and_clear();

    // 3. Extract token
    let sp = create_spinner("Trích xuất token...");
    let token_info = match token::extract_token_from_db(&db_path) {
        Ok(info) => info,
        Err(e) => {
            sp.finish_and_clear();
            eprintln!("{} Trích xuất token thất bại: {}", "✘".red().bold(), e);
            return;
        }
    };
    sp.finish_and_clear();

    // 4. Verify / refresh token
    let sp = create_spinner("Xác thực token với Google...");
    let access_token = match oauth::get_user_info(&token_info.access_token).await {
        Ok(_) => {
            sp.finish_and_clear();
            token_info.access_token.clone()
        }
        Err(_) => {
            sp.set_message("Token hết hạn, đang refresh...".to_string());
            match oauth::refresh_access_token(&token_info.refresh_token).await {
                Ok(new_token) => {
                    sp.finish_and_clear();
                    new_token.access_token
                }
                Err(e) => {
                    sp.finish_and_clear();
                    eprintln!("{} Refresh token thất bại: {}", "✘".red().bold(), e);
                    return;
                }
            }
        }
    };

    // 5. Fetch user info
    let sp = create_spinner("Đang lấy thông tin user...");
    match oauth::get_user_info(&access_token).await {
        Ok(info) => {
            sp.finish_and_clear();
            println!();
            println!("  {} {}", "👤".bold(), "Account Info".bold().cyan());
            println!("  {}", "─".repeat(40).dimmed());
            println!("     {} : {}", "Email      ".bold(), info.email.green());
            println!("     {} : {}", "Name       ".bold(), info.name.as_deref().unwrap_or("N/A").to_string().white());
            println!("     {} : {}", "Given Name ".bold(), info.given_name.as_deref().unwrap_or("N/A").to_string().white());
            println!("     {} : {}", "Family Name".bold(), info.family_name.as_deref().unwrap_or("N/A").to_string().white());
            println!("     {} : {}", "Picture    ".bold(), info.picture.as_deref().unwrap_or("N/A").to_string().dimmed());
            println!();
        }
        Err(e) => {
            sp.finish_and_clear();
            eprintln!("{} Lấy user info thất bại: {}", "✘".red().bold(), e);
        }
    }
}
