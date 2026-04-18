use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::storage::SaveTarget;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub hotkey: String,
    pub save_targets: Vec<SaveTarget>,
    pub default_save_path: String,
    pub filename_template: String,
    /// Path to a Markdown file for auto-inserting screenshots
    pub markdown_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "Alt+Shift+S".into(),
            save_targets: vec![
                SaveTarget {
                    target_type: crate::storage::SaveTargetType::Clipboard,
                    path: None,
                    enabled: true,
                },
                SaveTarget {
                    target_type: crate::storage::SaveTargetType::LocalFile,
                    path: Some("~/Pictures/CapDrop".into()),
                    enabled: true,
                },
            ],
            default_save_path: "~/Pictures/CapDrop".into(),
            filename_template: "screenshot_{timestamp}".into(),
            markdown_path: None,
        }
    }
}

impl AppConfig {
    /// Get config file path: ~/.capdrop/config.json
    pub fn config_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        Ok(home.join(".capdrop").join("config.json"))
    }

    /// Load config from disk, or return default if not found
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path()?;
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Read config error: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Parse config error: {}", e))
    }

    /// Save config to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Create config dir error: {}", e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialize config error: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("Write config error: {}", e))
    }

    /// Update config and persist
    pub fn update(&mut self, new_config: AppConfig) -> Result<(), String> {
        *self = new_config;
        self.save()
    }
}
