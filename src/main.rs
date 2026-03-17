use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use base64::{engine::general_purpose, Engine as _};
use rusqlite::Connection;
use serde::Deserialize;

const VERSION: &str = "1.0.0";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

struct OAuthConfig {
    client_id: String,
    client_secret: String,
}

fn load_oauth_config() -> Result<OAuthConfig, String> {
    // 1. Ưu tiên: compile-time env (nhúng lúc build trong CI/CD)
    if let (Some(id), Some(secret)) = (option_env!("CLIENT_ID"), option_env!("CLIENT_SECRET")) {
        return Ok(OAuthConfig {
            client_id: id.to_string(),
            client_secret: secret.to_string(),
        });
    }

    // 2. Fallback: đọc file .env (cho local dev)
    let env_paths = [
        env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join(".env"))),
        Some(PathBuf::from(".env")),
    ];

    for path_opt in &env_paths {
        if let Some(path) = path_opt {
            if path.exists() {
                let content = fs::read_to_string(path)
                    .map_err(|e| format!("Không đọc được .env: {}", e))?;
                let mut client_id = None;
                let mut client_secret = None;
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') { continue; }
                    if let Some((key, value)) = line.split_once('=') {
                        match key.trim() {
                            "CLIENT_ID" => client_id = Some(value.trim().to_string()),
                            "CLIENT_SECRET" => client_secret = Some(value.trim().to_string()),
                            _ => {}
                        }
                    }
                }
                return Ok(OAuthConfig {
                    client_id: client_id.ok_or("CLIENT_ID không có trong .env")?,
                    client_secret: client_secret.ok_or("CLIENT_SECRET không có trong .env")?,
                });
            }
        }
    }

    Err("Không tìm thấy config OAuth. Cần .env file hoặc build với CLIENT_ID/CLIENT_SECRET env.".to_string())
}

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

// ============================================================
// Protobuf Utilities (port từ Antigravity-Manager)
// ============================================================

mod protobuf {
    pub fn read_varint(data: &[u8], offset: usize) -> Result<(u64, usize), String> {
        let mut result = 0u64;
        let mut shift = 0;
        let mut pos = offset;
        loop {
            if pos >= data.len() {
                return Err("incomplete_data".to_string());
            }
            let byte = data[pos];
            result |= ((byte & 0x7F) as u64) << shift;
            pos += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        Ok((result, pos))
    }

    pub fn skip_field(data: &[u8], offset: usize, wire_type: u8) -> Result<usize, String> {
        match wire_type {
            0 => {
                let (_, new_offset) = read_varint(data, offset)?;
                Ok(new_offset)
            }
            1 => Ok(offset + 8),
            2 => {
                let (length, content_offset) = read_varint(data, offset)?;
                Ok(content_offset + length as usize)
            }
            5 => Ok(offset + 4),
            _ => Err(format!("unknown_wire_type: {}", wire_type)),
        }
    }

    pub fn find_field(data: &[u8], target_field: u32) -> Result<Option<Vec<u8>>, String> {
        let mut offset = 0;
        while offset < data.len() {
            let (tag, new_offset) = match read_varint(data, offset) {
                Ok(v) => v,
                Err(_) => break,
            };
            let wire_type = (tag & 7) as u8;
            let field_num = (tag >> 3) as u32;
            if field_num == target_field && wire_type == 2 {
                let (length, content_offset) = read_varint(data, new_offset)?;
                return Ok(Some(
                    data[content_offset..content_offset + length as usize].to_vec(),
                ));
            }
            offset = skip_field(data, new_offset, wire_type)?;
        }
        Ok(None)
    }
}

// ============================================================
// Structs
// ============================================================

struct TokenInfo {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    email: String,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
}

// ============================================================
// Antigravity paths (cross-platform)
// ============================================================

fn get_antigravity_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().unwrap_or_default();
        return home.join("Library/Application Support/Antigravity");
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        return PathBuf::from(appdata).join("Antigravity");
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().unwrap_or_default();
        return home.join(".config/Antigravity");
    }
}

fn get_db_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Không tìm được home directory")?;
        return Ok(
            home.join("Library/Application Support/Antigravity/User/globalStorage/state.vscdb"),
        );
    }

    #[cfg(target_os = "windows")]
    {
        let appdata =
            std::env::var("APPDATA").map_err(|_| "Không lấy được APPDATA".to_string())?;
        return Ok(PathBuf::from(appdata).join("Antigravity\\User\\globalStorage\\state.vscdb"));
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Không tìm được home directory")?;
        return Ok(home.join(".config/Antigravity/User/globalStorage/state.vscdb"));
    }
}

// ============================================================
// Trích xuất Token từ state.vscdb
// ============================================================

fn extract_token_from_db(db_path: &Path) -> Result<TokenInfo, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Không mở được DB: {}", e))?;

    // 1. Thử new format (>= 1.16.5)
    let new_format_data: Option<String> = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            ["antigravityUnifiedStateSync.oauthToken"],
            |row| row.get(0),
        )
        .ok();

    if let Some(outer_b64) = new_format_data {
        let outer_blob = general_purpose::STANDARD
            .decode(&outer_b64)
            .map_err(|e| format!("Outer Base64 decode lỗi: {}", e))?;

        let inner1_blob = protobuf::find_field(&outer_blob, 1)
            .map_err(|e| format!("Parse Outer Field 1 lỗi: {}", e))?
            .ok_or("Outer Field 1 not found")?;

        let inner2_blob = protobuf::find_field(&inner1_blob, 2)
            .map_err(|e| format!("Parse Inner1 Field 2 lỗi: {}", e))?
            .ok_or("Inner1 Field 2 not found")?;

        let oauth_info_bytes = protobuf::find_field(&inner2_blob, 1)
            .map_err(|e| format!("Parse Inner2 Field 1 lỗi: {}", e))?
            .ok_or("Inner2 Field 1 not found")?;

        let oauth_info_b64 =
            String::from_utf8(oauth_info_bytes).map_err(|_| "OAuth Info B64 is not UTF-8")?;

        let oauth_blob = general_purpose::STANDARD
            .decode(&oauth_info_b64)
            .map_err(|e| format!("Inner Base64 decode lỗi: {}", e))?;

        return parse_oauth_info(&oauth_blob);
    }

    // 2. Thử old format (< 1.16.5)
    let current_data: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            ["jetskiStateSync.agentManagerInitState"],
            |row| row.get(0),
        )
        .map_err(|_| "Không tìm thấy token data trong cả 2 format".to_string())?;

    let blob = general_purpose::STANDARD
        .decode(&current_data)
        .map_err(|e| format!("Base64 decode lỗi: {}", e))?;

    let oauth_data = protobuf::find_field(&blob, 6)
        .map_err(|e| format!("Protobuf parse lỗi: {}", e))?
        .ok_or("OAuth data not found (Field 6)")?;

    parse_oauth_info(&oauth_data)
}

fn parse_oauth_info(oauth_blob: &[u8]) -> Result<TokenInfo, String> {
    let access_token = protobuf::find_field(oauth_blob, 1)
        .map_err(|e| format!("Parse access_token lỗi: {}", e))?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();

    let refresh_token = protobuf::find_field(oauth_blob, 3)
        .map_err(|e| format!("Parse refresh_token lỗi: {}", e))?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or("Refresh token not found")?;

    Ok(TokenInfo {
        access_token,
        refresh_token,
    })
}

// ============================================================
// Google API calls
// ============================================================

async fn refresh_access_token(refresh_token: &str) -> Result<TokenResponse, String> {
    let config = load_oauth_config()?;
    let client = reqwest::Client::new();
    let params = [
        ("client_id", config.client_id.as_str()),
        ("client_secret", config.client_secret.as_str()),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Request lỗi: {}", e))?;

    if response.status().is_success() {
        response
            .json::<TokenResponse>()
            .await
            .map_err(|e| format!("Parse response lỗi: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Refresh token thất bại: {}", error_text))
    }
}

async fn get_user_info(access_token: &str) -> Result<UserInfo, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Request lỗi: {}", e))?;

    if response.status().is_success() {
        response
            .json::<UserInfo>()
            .await
            .map_err(|e| format!("Parse response lỗi: {}", e))
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!(
            "Lấy user info thất bại ({}): {}",
            status, error_text
        ))
    }
}

// ============================================================
// Command: info
// ============================================================

async fn cmd_info() {
    // 1. Kiểm tra Antigravity đã cài đặt chưa
    let data_dir = get_antigravity_data_dir();
    if !data_dir.exists() {
        eprintln!("❌ Antigravity chưa được cài đặt hoặc chưa từng mở.");
        eprintln!("   Không tìm thấy thư mục: {}", data_dir.display());
        return;
    }

    // 2. Tìm DB
    let db_path = match get_db_path() {
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

    // 3. Trích xuất token
    let token_info = match extract_token_from_db(&db_path) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("❌ Trích xuất token thất bại: {}", e);
            return;
        }
    };

    // 4. Lấy access_token hợp lệ (refresh nếu cần)
    let access_token = match get_user_info(&token_info.access_token).await {
        Ok(_) => token_info.access_token.clone(),
        Err(_) => match refresh_access_token(&token_info.refresh_token).await {
            Ok(new_token) => new_token.access_token,
            Err(e) => {
                eprintln!("❌ Refresh token thất bại: {}", e);
                return;
            }
        },
    };

    // 5. Lấy và hiển thị account info
    match get_user_info(&access_token).await {
        Ok(info) => {
            println!();
            println!("  👤 Account Info:");
            println!("     Email       : {}", info.email);
            println!(
                "     Name        : {}",
                info.name.as_deref().unwrap_or("N/A")
            );
            println!(
                "     Given Name  : {}",
                info.given_name.as_deref().unwrap_or("N/A")
            );
            println!(
                "     Family Name : {}",
                info.family_name.as_deref().unwrap_or("N/A")
            );
            println!(
                "     Picture     : {}",
                info.picture.as_deref().unwrap_or("N/A")
            );
            println!();
        }
        Err(e) => {
            eprintln!("❌ Lấy user info thất bại: {}", e);
        }
    }
}

// ============================================================
// Main
// ============================================================

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        process::exit(0);
    }

    match args[1].as_str() {
        "info" => cmd_info().await,
        "version" | "--version" | "-v" => println!("{}", VERSION),
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("Error: Lệnh '{}' không tồn tại.", other);
            eprintln!("Dùng 'tool-change-account help' để xem danh sách lệnh.");
            process::exit(1);
        }
    }
}
