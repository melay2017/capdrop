use crate::capture::{CaptureEngine, Screenshot};
use base64::Engine;
use std::process::Command;

pub struct MacCaptureEngine;

impl MacCaptureEngine {
    pub fn new() -> Self {
        Self
    }
}

impl CaptureEngine for MacCaptureEngine {
    fn capture_full_screen(&self) -> Result<Screenshot, String> {
        let png_data = capture_via_screencapture(None)?;
        let img = image::load_from_memory(&png_data)
            .map_err(|e| format!("Decode screenshot error: {}", e))?;
        Ok(Screenshot {
            data: png_data,
            width: img.width(),
            height: img.height(),
        })
    }

    fn capture_region(&self, x: u32, y: u32, w: u32, h: u32) -> Result<Screenshot, String> {
        let rect = format!("{},{},{},{}", x, y, w, h);
        let png_data = capture_via_screencapture(Some(&rect))?;
        let img = image::load_from_memory(&png_data)
            .map_err(|e| format!("Decode region error: {}", e))?;
        Ok(Screenshot {
            data: png_data,
            width: img.width(),
            height: img.height(),
        })
    }
}

/// Use macOS `screencapture` CLI tool - most reliable method
/// - Full screen: `screencapture -x -t png /tmp/capdrop_xxx.png`
/// - Region: `screencapture -x -R x,y,w,h -t png /tmp/capdrop_xxx.png`
fn capture_via_screencapture(rect: Option<&str>) -> Result<Vec<u8>, String> {
    let tmp_path = format!("/tmp/capdrop_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S_%3f"));

    let mut cmd = Command::new("/usr/sbin/screencapture");
    cmd.arg("-x")  // no sound
        .arg("-t").arg("png");

    if let Some(r) = rect {
        cmd.arg("-R").arg(r);
    }

    cmd.arg(&tmp_path);

    let output = cmd.output()
        .map_err(|e| format!("screencapture exec error: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("screencapture failed: {}", stderr));
    }

    let data = std::fs::read(&tmp_path)
        .map_err(|e| format!("Read screenshot file error: {}", e))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&tmp_path);

    Ok(data)
}

/// Public helper: capture fullscreen and return base64 PNG
pub fn capture_screen() -> Result<String, String> {
    let engine = MacCaptureEngine::new();
    let screenshot = engine.capture_full_screen()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&screenshot.data);
    Ok(b64)
}

/// Public helper: capture region and return base64 PNG
pub fn capture_region_b64(x: u32, y: u32, w: u32, h: u32) -> Result<String, String> {
    let engine = MacCaptureEngine::new();
    let screenshot = engine.capture_region(x, y, w, h)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&screenshot.data);
    Ok(b64)
}
