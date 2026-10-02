use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};
use slint::{Color, ComponentHandle, SharedString};

slint::include_modules!();

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    shortcut: String,
    last_color: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcut: "SUPER + SHIFT + C".to_string(),
            last_color: "#B59EE6".to_string(),
        }
    }
}

fn config_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("powertoys").join("config.json")
    } else {
        PathBuf::from("config.json")
    }
}

fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&content) {
                return cfg;
            }
        }
    }
    AppConfig::default()
}

fn save_config(cfg: &AppConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, json);
    }
}

fn parse_hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

fn copy_to_clipboard(text: &str) {
    let _ = Command::new("wl-copy").arg(text).spawn();
}

fn apply_shortcut_to_hyprland(shortcut_str: &str) -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;

    // Parse shortcut string like "SUPER + SHIFT + C" or "SUPER SHIFT, C"
    let cleaned: Vec<String> = shortcut_str
        .split(['+', ',', ' '])
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_uppercase())
        .collect();

    if cleaned.is_empty() {
        return Err("Shortcut cannot be empty".to_string());
    }

    let key = cleaned.last().unwrap();
    let mods = cleaned[..cleaned.len() - 1].join(" ");
    let lua_combo = cleaned.join(" + ");

    // 1. Standard Hyprland config (~/.config/hypr/powertoys.conf)
    let hypr_dir = PathBuf::from(&home).join(".config").join("hypr");
    let powertoys_conf = hypr_dir.join("powertoys.conf");
    let bind_content = format!(
        "# Auto-generated PowerToys Hyprland Keybinds\nbind = {}, {}, exec, hyprpicker -a -n\n",
        if mods.is_empty() { "SUPER".to_string() } else { mods },
        key
    );
    let _ = std::fs::write(&powertoys_conf, bind_content);

    // Ensure source in hyprland.conf if present
    let hypr_conf_path = hypr_dir.join("hyprland.conf");
    if let Ok(hypr_conf_content) = std::fs::read_to_string(&hypr_conf_path) {
        if !hypr_conf_content.contains("powertoys.conf") {
            let updated = format!(
                "{}\n# PowerToys Integration\nsource = ~/.config/hypr/powertoys.conf\n",
                hypr_conf_content.trim_end()
            );
            let _ = std::fs::write(&hypr_conf_path, updated);
        }
    }

    // 2. Caelestia / Midnight Lua Hyprland config (~/.config/powertoys/powertoys.lua)
    let powertoys_dir = PathBuf::from(&home).join(".config").join("powertoys");
    let _ = std::fs::create_dir_all(&powertoys_dir);
    let powertoys_lua = powertoys_dir.join("powertoys.lua");
    let lua_content = format!(
        "-- Auto-generated PowerToys Hyprland Lua Keybinds\nhl.bind(\"{}\", hl.dsp.exec_cmd(\"hyprpicker -a -n\"))\n",
        lua_combo
    );
    let _ = std::fs::write(&powertoys_lua, lua_content);

    let caelestia_user_lua = PathBuf::from(&home).join(".config").join("caelestia").join("hypr-user.lua");
    if let Ok(user_lua_content) = std::fs::read_to_string(&caelestia_user_lua) {
        if !user_lua_content.contains("powertoys.lua") {
            let updated = format!(
                "{}\n\n-- PowerToys Integration\npcall(function()\n    dofile(os.getenv(\"HOME\") .. \"/.config/powertoys/powertoys.lua\")\nend)\n",
                user_lua_content.trim_end()
            );
            let _ = std::fs::write(&caelestia_user_lua, updated);
        }
    }

    // 3. Reload Hyprland
    let _ = Command::new("hyprctl").arg("reload").status();
    Ok(())
}

fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;

    // Fullscreen mode
    app.window().set_fullscreen(true);

    let cfg = load_config();
    let initial_shortcut = cfg.shortcut.clone();
    let initial_hex = cfg.last_color.clone();

    // Set initial UI values
    app.set_shortcut_text(SharedString::from(&initial_shortcut));
    app.set_current_hex(SharedString::from(&initial_hex));

    if let Some((r, g, b)) = parse_hex_to_rgb(&initial_hex) {
        app.set_current_rgb(SharedString::from(format!("rgb({}, {}, {})", r, g, b)));
        app.set_current_color(Color::from_rgb_u8(r, g, b));
    }

    // Ensure Hyprland bind is applied at startup
    let _ = apply_shortcut_to_hyprland(&initial_shortcut);

    // Escape or close callback
    let app_weak = app.as_weak();
    app.on_close_requested(move || {
        if let Some(w) = app_weak.upgrade() {
            let _ = w.hide();
        }
    });

    // Pick color using Hyprpicker
    let app_weak = app.as_weak();
    app.on_pick_color_clicked(move || {
        let weak = app_weak.clone();
        std::thread::spawn(move || {
            if let Ok(output) = Command::new("hyprpicker").arg("-a").arg("-n").output() {
                if output.status.success() {
                    let hex = String::from_utf8_lossy(&output.stdout).trim().to_uppercase();
                    if !hex.is_empty() {
                        let mut c = load_config();
                        c.last_color = hex.clone();
                        save_config(&c);

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(w) = weak.upgrade() {
                                w.set_current_hex(SharedString::from(&hex));
                                if let Some((r, g, b)) = parse_hex_to_rgb(&hex) {
                                    w.set_current_rgb(SharedString::from(format!("rgb({}, {}, {})", r, g, b)));
                                    w.set_current_color(Color::from_rgb_u8(r, g, b));
                                }
                                w.set_status_text(SharedString::from(format!("✓ Picked & copied {} to clipboard", hex)));
                            }
                        });
                    }
                }
            }
        });
    });

    // Copy HEX
    let app_weak = app.as_weak();
    app.on_copy_hex_clicked(move |hex| {
        let text = hex.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_status_text(SharedString::from(format!("✓ Copied {} to clipboard", text)));
        }
    });

    // Copy RGB
    let app_weak = app.as_weak();
    app.on_copy_rgb_clicked(move |rgb| {
        let text = rgb.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_status_text(SharedString::from(format!("✓ Copied {} to clipboard", text)));
        }
    });

    // Preset Swatch clicked
    let app_weak = app.as_weak();
    app.on_swatch_selected(move |hex| {
        let text = hex.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_current_hex(SharedString::from(&text));
            if let Some((r, g, b)) = parse_hex_to_rgb(&text) {
                w.set_current_rgb(SharedString::from(format!("rgb({}, {}, {})", r, g, b)));
                w.set_current_color(Color::from_rgb_u8(r, g, b));
            }
            w.set_status_text(SharedString::from(format!("✓ Swatch {} selected & copied", text)));
        }
    });

    // Save & apply shortcut
    let app_weak = app.as_weak();
    app.on_save_shortcut_clicked(move |shortcut| {
        let s = shortcut.to_string();
        match apply_shortcut_to_hyprland(&s) {
            Ok(_) => {
                let mut c = load_config();
                c.shortcut = s.clone();
                save_config(&c);
                if let Some(w) = app_weak.upgrade() {
                    w.set_shortcut_text(SharedString::from(&s));
                    w.set_status_text(SharedString::from(format!("✓ Bound shortcut '{}' to Hyprland!", s)));
                }
            }
            Err(err) => {
                if let Some(w) = app_weak.upgrade() {
                    w.set_status_text(SharedString::from(format!("✗ Failed to bind: {}", err)));
                }
            }
        }
    });

    app.run()
}
