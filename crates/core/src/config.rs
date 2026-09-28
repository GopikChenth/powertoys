use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub accent_color: String,
    pub modules: HashMap<String, ModuleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub enabled: bool,
    pub hotkey: String,
    #[serde(default)]
    pub extra: serde_json::Value,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut modules = HashMap::new();
        modules.insert(
            "run".to_string(),
            ModuleConfig {
                enabled: true,
                hotkey: "Super+Space".to_string(),
                extra: serde_json::json!({
                    "max_results": 8,
                    "search_files": true,
                    "enable_calculator": true,
                }),
            },
        );
        modules.insert(
            "fancyzones".to_string(),
            ModuleConfig {
                enabled: true,
                hotkey: "Super+Shift+Z".to_string(),
                extra: serde_json::json!({
                    "zones_count": 3,
                    "zone_padding": 8,
                }),
            },
        );

        Self {
            theme: "caelestia-dark".to_string(),
            accent_color: "#b4befe".to_string(), // Catppuccin Lavender
            modules,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> std::path::PathBuf {
        if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            std::path::PathBuf::from(config_home).join("powertoys").join("config.json")
        } else if let Ok(home) = std::env::var("HOME") {
            std::path::PathBuf::from(home).join(".config").join("powertoys").join("config.json")
        } else {
            std::path::PathBuf::from("config.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&content) {
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn is_module_enabled(&self, id: &str) -> bool {
        self.modules.get(id).map(|m| m.enabled).unwrap_or(false)
    }

    pub fn set_module_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(m) = self.modules.get_mut(id) {
            m.enabled = enabled;
        } else {
            self.modules.insert(
                id.to_string(),
                ModuleConfig {
                    enabled,
                    hotkey: String::new(),
                    extra: serde_json::Value::Null,
                },
            );
        }
    }
}

