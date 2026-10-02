use std::path::PathBuf;
use std::process::Command;

pub struct HyprlandManager;

impl HyprlandManager {
    /// Applies a global shortcut to Hyprland for a given command.
    /// Handles both Caelestia/Lua configurations and standard hyprland.conf setups.
    pub fn register_shortcut(shortcut_str: &str, exec_command: &str) -> Result<(), String> {
        let home = std::env::var("HOME").map_err(|e| e.to_string())?;

        // Parse shortcut tokens, e.g. "SUPER + SHIFT + C" -> ["SUPER", "SHIFT", "C"]
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
            "# Auto-generated PowerToys Hyprland Keybinds\nbind = {}, {}, exec, {}\n",
            if mods.is_empty() { "SUPER".to_string() } else { mods },
            key,
            exec_command
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
            "-- Auto-generated PowerToys Hyprland Lua Keybinds\nhl.bind(\"{}\", hl.dsp.exec_cmd(\"{}\"))\n",
            lua_combo, exec_command
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
        Self::reload();
        Ok(())
    }

    pub fn reload() {
        let _ = Command::new("hyprctl").arg("reload").status();
    }
}
