use std::path::Path;

use base64::{engine::general_purpose, Engine as _};
use rusqlite::Connection;

use crate::protobuf;

pub struct TokenInfo {
    pub access_token: String,
    pub refresh_token: String,
}

pub fn extract_token_from_db(db_path: &Path) -> Result<TokenInfo, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Cannot open DB: {}", e))?;

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
            .map_err(|e| format!("Outer Base64 decode error: {}", e))?;

        let inner1_blob = protobuf::find_field(&outer_blob, 1)
            .map_err(|e| format!("Parse Outer Field 1 error: {}", e))?
            .ok_or("Outer Field 1 not found")?;

        let inner2_blob = protobuf::find_field(&inner1_blob, 2)
            .map_err(|e| format!("Parse Inner1 Field 2 error: {}", e))?
            .ok_or("Inner1 Field 2 not found")?;

        let oauth_info_bytes = protobuf::find_field(&inner2_blob, 1)
            .map_err(|e| format!("Parse Inner2 Field 1 error: {}", e))?
            .ok_or("Inner2 Field 1 not found")?;

        let oauth_info_b64 =
            String::from_utf8(oauth_info_bytes).map_err(|_| "OAuth Info B64 is not UTF-8")?;

        let oauth_blob = general_purpose::STANDARD
            .decode(&oauth_info_b64)
            .map_err(|e| format!("Inner Base64 decode error: {}", e))?;

        return parse_oauth_info(&oauth_blob);
    }

    let current_data: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            ["jetskiStateSync.agentManagerInitState"],
            |row| row.get(0),
        )
        .map_err(|_| "Token data not found in either format".to_string())?;

    let blob = general_purpose::STANDARD
        .decode(&current_data)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    let oauth_data = protobuf::find_field(&blob, 6)
        .map_err(|e| format!("Protobuf parse error: {}", e))?
        .ok_or("OAuth data not found (Field 6)")?;

    parse_oauth_info(&oauth_data)
}

fn parse_oauth_info(oauth_blob: &[u8]) -> Result<TokenInfo, String> {
    let access_token = protobuf::find_field(oauth_blob, 1)
        .map_err(|e| format!("Parse access_token error: {}", e))?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();

    let refresh_token = protobuf::find_field(oauth_blob, 3)
        .map_err(|e| format!("Parse refresh_token error: {}", e))?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or("Refresh token not found")?;

    Ok(TokenInfo {
        access_token,
        refresh_token,
    })
}
