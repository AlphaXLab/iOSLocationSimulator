use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub google_maps_api_key: String,
    pub map_style: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            google_maps_api_key: String::new(),
            map_style: "google".to_string(),
        }
    }
}

fn parse_env_file(path: &Path) -> Option<String> {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim();
                let mut val = v.trim();
                if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                    if val.len() >= 2 {
                        val = &val[1..val.len() - 1];
                    }
                }
                if key == "GOOGLE_MAPS_API_KEY" || key == "MAP_API_KEY" || key == "GOOGLE_API_KEY" {
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

fn find_env_api_key() -> Option<String> {
    // 1. Process environment variables
    for var in &["GOOGLE_MAPS_API_KEY", "MAP_API_KEY", "GOOGLE_API_KEY"] {
        if let Ok(val) = std::env::var(var) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // 2. Search for .env file in current directory and parent paths
    let candidate_dirs = [
        std::env::current_dir().ok(),
        std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())),
        dirs::home_dir(),
    ];

    for dir in candidate_dirs.into_iter().flatten() {
        let mut cur = dir.as_path();
        for _ in 0..4 {
            let env_path = cur.join(".env");
            if env_path.exists() {
                if let Some(key) = parse_env_file(&env_path) {
                    return Some(key);
                }
            }
            if let Some(parent) = cur.parent() {
                cur = parent;
            } else {
                break;
            }
        }
    }

    None
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("LocationSimulator").join("config.json"))
    }

    pub fn load() -> Self {
        let mut cfg = if let Some(path) = Self::config_path() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&data) {
                    cfg
                } else {
                    Self::default()
                }
            } else {
                Self::default()
            }
        } else {
            Self::default()
        };

        // If config file didn't specify a key, or if an environment/.env key is provided, use it
        if cfg.google_maps_api_key.trim().is_empty() {
            if let Some(env_key) = find_env_api_key() {
                cfg.google_maps_api_key = env_key;
            }
        }

        cfg
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(data) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, data);
            }
        }
    }
}
