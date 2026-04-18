#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

pub struct Screenshot {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub trait CaptureEngine {
    fn capture_full_screen(&self) -> Result<Screenshot, String>;
    fn capture_region(&self, x: u32, y: u32, w: u32, h: u32) -> Result<Screenshot, String>;
}
