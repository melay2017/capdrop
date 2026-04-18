use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Append screenshot reference to a Markdown file
/// Inserts: ![CapDrop_{timestamp}](relative_path)
pub fn append_to_markdown(md_path: &PathBuf, image_path: &PathBuf, alt_text: Option<&str>) -> Result<(), String> {
    // Ensure the markdown file exists
    if !md_path.exists() {
        fs::write(md_path, "# CapDrop Screenshots\n\n")
            .map_err(|e| format!("Create md error: {}", e))?;
    }

    // Calculate relative path from md file's directory to image
    let rel_path = compute_relative_path(md_path, image_path);

    let alt_default = format!("CapDrop_{}", chrono::Local::now().format("%H%M%S"));
    let alt = alt_text.unwrap_or(&alt_default);
    let line = format!("![{}]({})\n\n", alt, rel_path);

    let mut file = OpenOptions::new()
        .append(true)
        .open(md_path)
        .map_err(|e| format!("Open md error: {}", e))?;

    file.write_all(line.as_bytes())
        .map_err(|e| format!("Write md error: {}", e))?;

    Ok(())
}

/// Compute relative path from md file directory to image file
fn compute_relative_path(from: &PathBuf, to: &PathBuf) -> String {
    if let Some(from_dir) = from.parent() {
        if let Ok(rel) = to.strip_prefix(from_dir) {
            return rel.to_string_lossy().to_string();
        }
    }
    to.to_string_lossy().to_string()
}
