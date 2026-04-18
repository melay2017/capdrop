use crate::capture::{CaptureEngine, Screenshot};

pub struct WindowsCaptureEngine;

impl WindowsCaptureEngine {
    pub fn new() -> Self {
        Self
    }
}

impl CaptureEngine for WindowsCaptureEngine {
    fn capture_full_screen(&self) -> Result<Screenshot, String> {
        // Use Windows Graphics Capture API via windows crate
        // For MVP, we use the PrintScreen approach via clipboard
        let png_data = capture_via_print_screen()?;
        let img = image::load_from_memory(&png_data)
            .map_err(|e| format!("Decode screenshot error: {}", e))?;
        Ok(Screenshot {
            data: png_data,
            width: img.width(),
            height: img.height(),
        })
    }

    fn capture_region(&self, x: u32, y: u32, w: u32, h: u32) -> Result<Screenshot, String> {
        // Capture full screen then crop
        let full = self.capture_full_screen()?;
        let img = image::load_from_memory(&full.data)
            .map_err(|e| format!("Decode for crop: {}", e))?;

        let cropped = img.crop_imm(x, y, w, h);
        let mut png_data = Vec::new();
        cropped.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
            .map_err(|e| format!("PNG encode crop: {}", e))?;

        Ok(Screenshot {
            data: png_data,
            width: cropped.width(),
            height: cropped.height(),
        })
    }
}

/// Capture screenshot using BitBlt (GDI) - works on all Windows versions
fn capture_via_print_screen() -> Result<Vec<u8>, String> {
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::WinRT::*;

    unsafe {
        // Get screen dimensions
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        if screen_w <= 0 || screen_h <= 0 {
            return Err("Invalid screen dimensions".into());
        }

        let w = screen_w as u32;
        let h = screen_h as u32;

        // Get DC for the entire screen
        let screen_dc = GetDC(None);
        if screen_dc.is_invalid() {
            return Err("Failed to get screen DC".into());
        }

        // Create compatible DC and bitmap
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        let bitmap = CreateCompatibleBitmap(screen_dc, screen_w, screen_h);
        let old_bitmap = SelectObject(mem_dc, bitmap);

        // BitBlt from screen to memory DC
        let result = BitBlt(
            mem_dc,
            0, 0,
            screen_w, screen_h,
            screen_dc,
            0, 0,
            SRCCOPY | CAPTUREBLT,
        );

        if result.is_err() {
            SelectObject(mem_dc, old_bitmap);
            DeleteObject(bitmap);
            DeleteDC(mem_dc);
            ReleaseDC(None, screen_dc);
            return Err("BitBlt failed".into());
        }

        // Get bitmap data as RGBA
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = screen_w;
        bmi.bmiHeader.biHeight = -screen_h; // top-down
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB.0;

        let pixel_count = (w * h) as usize;
        let mut pixels: Vec<u8> = vec![0u8; pixel_count * 4];

        let rows = GetDIBits(
            mem_dc,
            bitmap,
            0,
            h as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Cleanup GDI resources
        SelectObject(mem_dc, old_bitmap);
        DeleteObject(bitmap);
        DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);

        if rows == 0 {
            return Err("GetDIBits failed".into());
        }

        // Convert BGRA → RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // swap B and R
        }

        // Encode as PNG
        let img = image::RgbaImage::from_raw(w, h, pixels)
            .ok_or("Failed to create image buffer")?;

        let mut png_data = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
            .map_err(|e| format!("PNG encode error: {}", e))?;

        Ok(png_data)
    }
}

/// Public helper: capture fullscreen and return base64 PNG
pub fn capture_screen() -> Result<String, String> {
    use base64::Engine;
    let engine = WindowsCaptureEngine::new();
    let screenshot = engine.capture_full_screen()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&screenshot.data);
    Ok(b64)
}

/// Public helper: capture region and return base64 PNG
pub fn capture_region_b64(x: u32, y: u32, w: u32, h: u32) -> Result<String, String> {
    use base64::Engine;
    let engine = WindowsCaptureEngine::new();
    let screenshot = engine.capture_region(x, y, w, h)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&screenshot.data);
    Ok(b64)
}
