use crate::capture::Screenshot;
use crate::config::AppConfig;
use std::fs;
use std::path::PathBuf;

/// Save screenshot to local folder with timestamp naming
pub fn save_to_file(screenshot: &Screenshot, config: &AppConfig) -> Result<PathBuf, String> {
    let save_dir = expand_path(&config.default_save_path)?;
    fs::create_dir_all(&save_dir).map_err(|e| format!("Create dir error: {}", e))?;

    let filename = generate_filename(&config.filename_template);
    let filepath = save_dir.join(format!("{}.png", filename));

    fs::write(&filepath, &screenshot.data)
        .map_err(|e| format!("Write file error: {}", e))?;

    Ok(filepath)
}

/// Save raw PNG data to a specific path
pub fn save_png_to_path(data: &[u8], path: &PathBuf) -> Result<PathBuf, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create dir error: {}", e))?;
    }
    fs::write(path, data).map_err(|e| format!("Write error: {}", e))?;
    Ok(path.clone())
}

/// Generate filename from template
/// Supports: {timestamp}, {date}, {time}, {seq}
fn generate_filename(template: &str) -> String {
    let now = chrono::Local::now();
    let result = template
        .replace("{timestamp}", &now.format("%Y%m%d_%H%M%S").to_string())
        .replace("{date}", &now.format("%Y-%m-%d").to_string())
        .replace("{time}", &now.format("%H-%M-%S").to_string());

    // Add milliseconds for uniqueness
    let millis = now.format("%3f").to_string();
    result.replace("{ms}", &millis)
}

/// Expand ~ to home directory
fn expand_path(path: &str) -> Result<PathBuf, String> {
    if path.starts_with('~') {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        let rest = path.trim_start_matches('~').trim_start_matches('/');
        Ok(home.join(rest))
    } else {
        Ok(PathBuf::from(path))
    }
}
