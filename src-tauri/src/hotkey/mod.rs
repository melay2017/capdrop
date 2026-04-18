use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// Register global shortcut for screenshot trigger
pub fn register_hotkey(app: &AppHandle, hotkey_str: &str) -> Result<(), String> {
    let shortcut = parse_shortcut(hotkey_str)?;

    app.global_shortcut()
        .on_shortcut(shortcut, move |app_handle, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = app_handle.emit("hotkey:screenshot", ());
            }
        })
        .map_err(|e| format!("Register shortcut error: {}", e))?;

    Ok(())
}

/// Unregister all shortcuts
pub fn unregister_all(app: &AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("Unregister shortcuts error: {}", e))
}

/// Parse shortcut string like "Alt+Shift+S" into Shortcut
fn parse_shortcut(s: &str) -> Result<Shortcut, String> {
    let mut modifiers = Modifiers::empty();
    let mut key: Option<Code> = None;

    for part in s.split('+') {
        let part = part.trim();
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "cmd" | "command" => modifiers |= Modifiers::SUPER,
            _ => {
                // Try to parse as key code
                let code = parse_key_code(part)?;
                key = Some(code);
            }
        }
    }

    let code = key.ok_or_else(|| format!("No key specified in shortcut: {}", s))?;
    Ok(Shortcut::new(Some(modifiers), code))
}

fn parse_key_code(s: &str) -> Result<Code, String> {
    match s.to_lowercase().as_str() {
        "a" => Ok(Code::KeyA),
        "b" => Ok(Code::KeyB),
        "c" => Ok(Code::KeyC),
        "d" => Ok(Code::KeyD),
        "e" => Ok(Code::KeyE),
        "f" => Ok(Code::KeyF),
        "g" => Ok(Code::KeyG),
        "h" => Ok(Code::KeyH),
        "i" => Ok(Code::KeyI),
        "j" => Ok(Code::KeyJ),
        "k" => Ok(Code::KeyK),
        "l" => Ok(Code::KeyL),
        "m" => Ok(Code::KeyM),
        "n" => Ok(Code::KeyN),
        "o" => Ok(Code::KeyO),
        "p" => Ok(Code::KeyP),
        "q" => Ok(Code::KeyQ),
        "r" => Ok(Code::KeyR),
        "s" => Ok(Code::KeyS),
        "t" => Ok(Code::KeyT),
        "u" => Ok(Code::KeyU),
        "v" => Ok(Code::KeyV),
        "w" => Ok(Code::KeyW),
        "x" => Ok(Code::KeyX),
        "y" => Ok(Code::KeyY),
        "z" => Ok(Code::KeyZ),
        "0" => Ok(Code::Digit0),
        "1" => Ok(Code::Digit1),
        "2" => Ok(Code::Digit2),
        "3" => Ok(Code::Digit3),
        "4" => Ok(Code::Digit4),
        "5" => Ok(Code::Digit5),
        "6" => Ok(Code::Digit6),
        "7" => Ok(Code::Digit7),
        "8" => Ok(Code::Digit8),
        "9" => Ok(Code::Digit9),
        "f1" => Ok(Code::F1),
        "f2" => Ok(Code::F2),
        "f3" => Ok(Code::F3),
        "f4" => Ok(Code::F4),
        "f5" => Ok(Code::F5),
        "f6" => Ok(Code::F6),
        "f7" => Ok(Code::F7),
        "f8" => Ok(Code::F8),
        "f9" => Ok(Code::F9),
        "f10" => Ok(Code::F10),
        "f11" => Ok(Code::F11),
        "f12" => Ok(Code::F12),
        "space" => Ok(Code::Space),
        "enter" | "return" => Ok(Code::Enter),
        "escape" | "esc" => Ok(Code::Escape),
        "tab" => Ok(Code::Tab),
        "backquote" | "`" => Ok(Code::Backquote),
        _ => Err(format!("Unknown key: {}", s)),
    }
}
