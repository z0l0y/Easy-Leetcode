use serde::Deserialize;
use std::path::PathBuf;

/// Global configuration loaded from `.lhconfig.toml`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    /// Path to the JDK installation (for javac/java).
    #[serde(default)]
    #[allow(dead_code)]
    pub jdk_path: Option<String>,

    /// Directory for trace cache files.
    #[serde(default)]
    pub cache_dir: Option<String>,

    /// Default theme file path.
    #[serde(default)]
    pub default_theme: Option<String>,

    /// Default trace mode: "tui" or "text".
    #[serde(default)]
    #[allow(dead_code)]
    pub trace_mode: Option<String>,
}

/// Search for `.lhconfig.toml` starting from the current directory,
/// walking up to parent directories, and finally checking the user's home directory.
pub fn load_config() -> Config {
    // 1. Start from current directory
    let mut current = std::env::current_dir().ok();

    while let Some(dir) = current {
        let candidate = dir.join(".lhconfig.toml");
        if candidate.exists()
            && let Ok(content) = std::fs::read_to_string(&candidate)
        {
            if let Ok(config) = toml::from_str::<Config>(&content) {
                return config;
            }
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }

    // 2. Check home directory
    if let Some(home) = dirs_home() {
        let candidate = home.join(".lhconfig.toml");
        if candidate.exists()
            && let Ok(content) = std::fs::read_to_string(&candidate)
        {
            if let Ok(config) = toml::from_str::<Config>(&content) {
                return config;
            }
        }
    }

    Config::default()
}

fn dirs_home() -> Option<PathBuf> {
    // Use HOME on Unix, USERPROFILE on Windows
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(home));
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return Some(PathBuf::from(profile));
    }
    None
}
