pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

pub fn load_oauth_config() -> Result<OAuthConfig, String> {
    if let (Some(id), Some(secret)) = (option_env!("CLIENT_ID"), option_env!("CLIENT_SECRET")) {
        return Ok(OAuthConfig {
            client_id: id.to_string(),
            client_secret: secret.to_string(),
        });
    }

    let _ = dotenvy::dotenv();

    let client_id = std::env::var("CLIENT_ID")
        .map_err(|_| "CLIENT_ID not found. Need .env file or build with CLIENT_ID env.")?;
    let client_secret = std::env::var("CLIENT_SECRET")
        .map_err(|_| "CLIENT_SECRET not found. Need .env file or build with CLIENT_SECRET env.")?;

    Ok(OAuthConfig {
        client_id,
        client_secret,
    })
}
