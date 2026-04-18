use crate::capture::Screenshot;
use std::process::{Command, Stdio};
use std::io::Write;

/// Copy PNG image to system clipboard
/// Uses macOS `pbcopy` workaround: write PNG to temp, then osascript to set clipboard
pub fn copy_to_clipboard(screenshot: &Screenshot) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos_copy_to_clipboard(&screenshot.data)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = screenshot;
        Err("Clipboard not implemented for this platform".into())
    }
}

#[cfg(target_os = "macos")]
fn macos_copy_to_clipboard(png_data: &[u8]) -> Result<(), String> {
    // Write PNG to temp file, then use osascript to set clipboard image
    let tmp_path = format!("/tmp/capdrop_clip_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S_%3f"));

    std::fs::write(&tmp_path, png_data)
        .map_err(|e| format!("Write temp error: {}", e))?;

    // Use osascript to copy image to clipboard
    let script = format!(
        "set the clipboard to (read (POSIX file \"{}\") as «class PNGf»)",
        tmp_path
    );

    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("osascript error: {}", e))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&tmp_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Clipboard copy failed: {}", stderr));
    }

    Ok(())
}
