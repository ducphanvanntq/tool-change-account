use crate::{oauth, paths, token};

pub async fn cmd_info() {
    let data_dir = paths::get_antigravity_data_dir();
    if !data_dir.exists() {
        eprintln!("❌ Antigravity chưa được cài đặt hoặc chưa từng mở.");
        eprintln!("   Không tìm thấy thư mục: {}", data_dir.display());
        return;
    }

    let db_path = match paths::get_db_path() {
        Ok(p) if p.exists() => p,
        Ok(_) => {
            eprintln!("❌ Antigravity đã cài nhưng chưa đăng nhập (state.vscdb không tồn tại).");
            return;
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            return;
        }
    };

    let token_info = match token::extract_token_from_db(&db_path) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("❌ Trích xuất token thất bại: {}", e);
            return;
        }
    };

    let access_token = match oauth::get_user_info(&token_info.access_token).await {
        Ok(_) => token_info.access_token.clone(),
        Err(_) => match oauth::refresh_access_token(&token_info.refresh_token).await {
            Ok(new_token) => new_token.access_token,
            Err(e) => {
                eprintln!("❌ Refresh token thất bại: {}", e);
                return;
            }
        },
    };

    match oauth::get_user_info(&access_token).await {
        Ok(info) => {
            println!();
            println!("  👤 Account Info:");
            println!("     Email       : {}", info.email);
            println!("     Name        : {}", info.name.as_deref().unwrap_or("N/A"));
            println!("     Given Name  : {}", info.given_name.as_deref().unwrap_or("N/A"));
            println!("     Family Name : {}", info.family_name.as_deref().unwrap_or("N/A"));
            println!("     Picture     : {}", info.picture.as_deref().unwrap_or("N/A"));
            println!();
        }
        Err(e) => {
            eprintln!("❌ Lấy user info thất bại: {}", e);
        }
    }
}
