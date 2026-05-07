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
        match manager.path().app_data_dir() {
            Ok(dir) => dir.join("config.json"),
            Err(e) => {
                eprintln!("Warning: failed to get app data dir: {}. Using fallback path.", e);
                PathBuf::from("config.json")
            }
        }
    }

    pub fn load<R: tauri::Runtime>(manager: &impl Manager<R>) -> Self {
        let path = Self::config_path(manager);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Warning: failed to parse config at {:?}: {}. Using defaults.", path, e);
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!("Warning: failed to read config at {:?}: {}. Using defaults.", path, e);
                    Self::default()
                }
            }
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
