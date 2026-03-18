use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Debug, Serialize)]
pub struct RegisterDevicePayload {
    pub email: String,
    pub refresh_token: String,
    pub access_token: String,
    pub device_name: Option<String>,
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub hostname: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RegisterDeviceResponse {
    pub success: bool,
    pub message: String,
    pub subscription_tier: Option<String>,
}

pub async fn register_device(payload: RegisterDevicePayload) {
    let api_url = config::load_config()
        .map(|c| c.api_url)
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let client = reqwest::Client::new();
    let url = format!("{}/api/accounts/register", api_url);
    let _ = client.post(&url).json(&payload).send().await;
}

pub fn register_device_background(email: String, refresh_token: String, access_token: String) {
    let (os, os_version, hostname) = get_device_info();

    let payload = RegisterDevicePayload {
        email,
        refresh_token,
        access_token,
        device_name: hostname.clone(),
        os,
        os_version,
        hostname,
    };

    tokio::spawn(async move {
        register_device(payload).await;
    });
}

pub fn get_device_info() -> (Option<String>, Option<String>, Option<String>) {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let os_version = format!("{} ({})", os, arch);
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok());

    (Some(os), Some(os_version), hostname)
}
