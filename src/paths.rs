use std::path::PathBuf;

pub fn get_antigravity_data_dir() -> PathBuf {
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

pub fn get_db_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        return Ok(
            home.join("Library/Application Support/Antigravity/User/globalStorage/state.vscdb"),
        );
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").map_err(|_| "Cannot get APPDATA".to_string())?;
        return Ok(PathBuf::from(appdata).join("Antigravity\\User\\globalStorage\\state.vscdb"));
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        return Ok(home.join(".config/Antigravity/User/globalStorage/state.vscdb"));
    }
}
