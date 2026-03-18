pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub api_url: String,
}

const DEFAULT_API_URL: &str = "http://localhost:3000";

pub fn load_config() -> Result<AppConfig, String> {
    let _ = dotenvy::dotenv();

    let client_id = option_env!("CLIENT_ID")
        .map(String::from)
        .or_else(|| std::env::var("CLIENT_ID").ok())
        .ok_or("CLIENT_ID not found. Need .env file or build with CLIENT_ID env.")?;

    let client_secret = option_env!("CLIENT_SECRET")
        .map(String::from)
        .or_else(|| std::env::var("CLIENT_SECRET").ok())
        .ok_or("CLIENT_SECRET not found. Need .env file or build with CLIENT_SECRET env.")?;

    let api_url = option_env!("API_URL")
        .map(String::from)
        .or_else(|| std::env::var("API_URL").ok())
        .unwrap_or_else(|| DEFAULT_API_URL.to_string());

    Ok(AppConfig {
        client_id,
        client_secret,
        api_url,
    })
}
