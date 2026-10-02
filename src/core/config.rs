use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPickerConfig {
    pub shortcut: String,
    pub last_color: String,
    pub history: Vec<String>,
}

impl Default for ColorPickerConfig {
    fn default() -> Self {
        Self {
            shortcut: "SUPER + SHIFT + C".to_string(),
            last_color: "#B59EE6".to_string(),
            history: vec![
                "#B59EE6".to_string(),
                "#F38BA8".to_string(),
                "#A6E3A1".to_string(),
                "#89B4FA".to_string(),
                "#F9E2AF".to_string(),
                "#FAB387".to_string(),
                "#CBA6F7".to_string(),
                "#94E2D5".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub color_picker: ColorPickerConfig,
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("powertoys").join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // First try standard modern format
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
                // Fallback for legacy flat format
                #[derive(Deserialize)]
                struct LegacyConfig {
                    shortcut: Option<String>,
                    last_color: Option<String>,
                }
                if let Ok(legacy) = serde_json::from_str::<LegacyConfig>(&content) {
                    let mut cfg = AppConfig::default();
                    if let Some(s) = legacy.shortcut {
                        cfg.color_picker.shortcut = s;
                    }
                    if let Some(c) = legacy.last_color {
                        cfg.color_picker.last_color = c.clone();
                        if !cfg.color_picker.history.contains(&c) {
                            cfg.color_picker.history.insert(0, c);
                        }
                    }
                    return cfg;
                }
            }
        }
        AppConfig::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn add_history_color(&mut self, hex: &str) {
        let upper = hex.to_uppercase();
        // Remove if already exists so it moves to front
        self.color_picker.history.retain(|h| !h.eq_ignore_ascii_case(&upper));
        self.color_picker.history.insert(0, upper.clone());
        // Cap at 10 items
        if self.color_picker.history.len() > 10 {
            self.color_picker.history.truncate(10);
        }
        self.color_picker.last_color = upper;
        self.save();
    }
}
