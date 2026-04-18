pub mod clipboard;
pub mod local;
pub mod markdown;

use crate::capture::Screenshot;
use crate::config::AppConfig;
use std::path::PathBuf;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SaveTarget {
    pub target_type: SaveTargetType,
    pub path: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum SaveTargetType {
    Clipboard,
    LocalFile,
    Markdown,
}

/// Save screenshot to all configured targets
pub fn save_to_all_targets(screenshot: &Screenshot, config: &AppConfig) -> Vec<SaveResult> {
    let mut results = Vec::new();

    for target in &config.save_targets {
        if !target.enabled {
            continue;
        }
        let result = match target.target_type {
            SaveTargetType::Clipboard => {
                match clipboard::copy_to_clipboard(screenshot) {
                    Ok(()) => SaveResult::success("Clipboard"),
                    Err(e) => SaveResult::error("Clipboard", e),
                }
            }
            SaveTargetType::LocalFile => {
                match local::save_to_file(screenshot, config) {
                    Ok(path) => SaveResult::success_with_path("LocalFile", path),
                    Err(e) => SaveResult::error("LocalFile", e),
                }
            }
            SaveTargetType::Markdown => {
                let md_path = match &config.markdown_path {
                    Some(p) => PathBuf::from(p),
                    None => {
                        results.push(SaveResult::error("Markdown", "No markdown path configured".into()));
                        continue;
                    }
                };
                // First save locally, then reference in md
                match local::save_to_file(screenshot, config) {
                    Ok(img_path) => {
                        match markdown::append_to_markdown(&md_path, &img_path, None) {
                            Ok(()) => SaveResult::success_with_path("Markdown", img_path),
                            Err(e) => SaveResult::error("Markdown", e),
                        }
                    }
                    Err(e) => SaveResult::error("Markdown", e),
                }
            }
        };
        results.push(result);
    }

    results
}

#[derive(Debug, serde::Serialize)]
pub struct SaveResult {
    pub target: String,
    pub success: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

impl SaveResult {
    fn success(target: &str) -> Self {
        Self { target: target.into(), success: true, path: None, error: None }
    }
    fn success_with_path(target: &str, path: PathBuf) -> Self {
        Self { target: target.into(), success: true, path: Some(path.to_string_lossy().into()), error: None }
    }
    fn error(target: &str, error: String) -> Self {
        Self { target: target.into(), success: false, path: None, error: Some(error) }
    }
}
