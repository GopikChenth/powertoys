pub mod model;

use std::process::Command;
use model::extract_hex;

pub fn copy_to_clipboard(text: &str) {
    let _ = Command::new("wl-copy").arg(text).spawn();
}

pub fn notify(summary: &str, body: &str) {
    let _ = Command::new("notify-send")
        .arg("-a")
        .arg("PowerToys")
        .arg(summary)
        .arg(body)
        .spawn();
}

/// Invokes hyprpicker, extracts clean hex, and copies to clipboard
pub fn pick_color_sync() -> Option<String> {
    // -a: autocopy, -b: no-fancy (clean hex without ANSI escape codes)
    let output = Command::new("hyprpicker")
        .arg("-a")
        .arg("-b")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let hex = extract_hex(&stdout_str)?;

    // Ensure clipboard has the clean hex code
    copy_to_clipboard(&hex);

    Some(hex)
}
