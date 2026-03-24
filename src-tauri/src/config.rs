use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub scan_dirs: Vec<String>,
    pub pinned: Vec<String>,
    pub recents: Vec<RecentEntry>,
    pub launch_flags: String,
    pub max_recents: usize,
    pub scan_depth: usize,
    pub terminal: String, // "auto", "terminal", "iterm2", "warp", "alacritty", "kitty"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEntry {
    pub path: String,
    pub timestamp: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut scan_dirs = vec![];
        if let Some(home) = dirs::home_dir() {
            let desktop = home.join("Desktop");
            if desktop.exists() {
                scan_dirs.push(desktop.to_string_lossy().to_string());
            }
            let projects = home.join("Projects");
            if projects.exists() {
                scan_dirs.push(projects.to_string_lossy().to_string());
            }
        }
        Self {
            scan_dirs,
            pinned: vec![],
            recents: vec![],
            launch_flags: String::new(),
            max_recents: 20,
            scan_depth: 3,
            terminal: "auto".to_string(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".cc-launcher")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if !path.exists() {
        let config = AppConfig::default();
        let _ = save_config(&config);
        return config;
    }
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;

    let path = config_path();
    let tmp_path = path.with_extension("json.tmp");
    let contents =
        serde_json::to_string_pretty(config).map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&tmp_path, &contents).map_err(|e| format!("Failed to write config: {}", e))?;
    fs::rename(&tmp_path, &path).map_err(|e| format!("Failed to save config: {}", e))?;
    Ok(())
}

pub fn add_recent(config: &mut AppConfig, path: &str) {
    config.recents.retain(|r| r.path != path);
    config.recents.insert(
        0,
        RecentEntry {
            path: path.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    );
    if config.recents.len() > config.max_recents {
        config.recents.truncate(config.max_recents);
    }
}
