use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub expose_network: bool,
    pub hostname: String,
    pub username: String,
    pub password: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 4096,
            expose_network: false,
            hostname: "0.0.0.0".to_string(),
            username: String::new(),
            password: String::new(),
        }
    }
}

impl Config {
    fn config_path<R: tauri::Runtime>(manager: &impl Manager<R>) -> PathBuf {
        manager
            .path()
            .app_data_dir()
            .expect("failed to get app data dir")
            .join("config.json")
    }

    pub fn load<R: tauri::Runtime>(manager: &impl Manager<R>) -> Self {
        let path = Self::config_path(manager);
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save<R: tauri::Runtime>(&self, manager: &impl Manager<R>) -> Result<(), String> {
        let path = Self::config_path(manager);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&path, serde_json::to_string_pretty(self).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
