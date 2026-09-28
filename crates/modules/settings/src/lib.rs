slint::include_modules!();

use std::sync::Arc;
use async_trait::async_trait;
use powertoys_core::{AppConfig, CompositorBridge, PowerToyModule};
use slint::{ComponentHandle, SharedString};
use tokio::sync::Mutex;
use tracing::info;

pub struct SettingsModule {
    config: Arc<Mutex<AppConfig>>,
    compositor: Option<Arc<dyn CompositorBridge>>,
}

impl SettingsModule {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(AppConfig::load())),
            compositor: None,
        }
    }
}

#[async_trait]
impl PowerToyModule for SettingsModule {
    fn id(&self) -> &'static str {
        "settings"
    }

    fn name(&self) -> &'static str {
        "PowerToys Settings & Dashboard"
    }

    fn description(&self) -> &'static str {
        "Central dashboard and configuration manager for PowerToys utilities"
    }

    fn default_hotkey(&self) -> &'static str {
        "Super+Shift+P"
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn set_enabled(&mut self, _enabled: bool) {}

    async fn init(&mut self, compositor: Arc<dyn CompositorBridge>) -> anyhow::Result<()> {
        self.compositor = Some(compositor);
        info!("PowerToys Settings & Dashboard module initialized");
        Ok(())
    }

    async fn trigger(&mut self) -> anyhow::Result<()> {
        info!("Opening PowerToys Settings & Dashboard UI");

        let cfg = AppConfig::load();
        *self.config.lock().await = cfg.clone();

        let window = SettingsWindow::new()?;

        // Initial values from config
        let run_enabled = cfg.is_module_enabled("run");
        let fz_enabled = cfg.is_module_enabled("fancyzones");

        let run_calc_enabled = cfg
            .modules
            .get("run")
            .and_then(|m| m.extra.get("enable_calculator"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let run_windows_enabled = cfg
            .modules
            .get("run")
            .and_then(|m| m.extra.get("search_windows"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        window.set_run_enabled(run_enabled);
        window.set_fancyzones_enabled(fz_enabled);
        window.set_run_calc_enabled(run_calc_enabled);
        window.set_run_windows_enabled(run_windows_enabled);
        window.set_active_preset(SharedString::from("3 Columns"));

        // Callbacks
        let config_arc = self.config.clone();
        window.on_toggle_run(move |val| {
            let cfg = config_arc.clone();
            tokio::spawn(async move {
                let mut guard = cfg.lock().await;
                guard.set_module_enabled("run", val);
                let _ = guard.save();
                info!("PowerToys Run enabled toggled to: {}", val);
            });
        });

        let config_arc = self.config.clone();
        window.on_toggle_fancyzones(move |val| {
            let cfg = config_arc.clone();
            tokio::spawn(async move {
                let mut guard = cfg.lock().await;
                guard.set_module_enabled("fancyzones", val);
                let _ = guard.save();
                info!("FancyZones enabled toggled to: {}", val);
            });
        });

        let config_arc = self.config.clone();
        window.on_toggle_run_calc(move |val| {
            let cfg = config_arc.clone();
            tokio::spawn(async move {
                let mut guard = cfg.lock().await;
                if let Some(m) = guard.modules.get_mut("run") {
                    if let Some(obj) = m.extra.as_object_mut() {
                        obj.insert("enable_calculator".to_string(), serde_json::Value::Bool(val));
                    }
                }
                let _ = guard.save();
            });
        });

        let config_arc = self.config.clone();
        window.on_toggle_run_windows(move |val| {
            let cfg = config_arc.clone();
            tokio::spawn(async move {
                let mut guard = cfg.lock().await;
                if let Some(m) = guard.modules.get_mut("run") {
                    if let Some(obj) = m.extra.as_object_mut() {
                        obj.insert("search_windows".to_string(), serde_json::Value::Bool(val));
                    }
                }
                let _ = guard.save();
            });
        });

        let config_arc = self.config.clone();
        window.on_select_preset(move |preset| {
            let p = preset.to_string();
            let cfg = config_arc.clone();
            tokio::spawn(async move {
                let mut guard = cfg.lock().await;
                if let Some(m) = guard.modules.get_mut("fancyzones") {
                    if let Some(obj) = m.extra.as_object_mut() {
                        obj.insert("default_preset".to_string(), serde_json::Value::String(p));
                    }
                }
                let _ = guard.save();
            });
        });

        let config_arc = self.config.clone();
        window.on_save_config(move || {
            let cfg = config_arc.clone();
            tokio::spawn(async move {
                let guard = cfg.lock().await;
                if let Err(e) = guard.save() {
                    tracing::error!("Failed to save config: {}", e);
                } else {
                    info!("Configuration saved to {}", AppConfig::config_path().display());
                }
            });
        });

        let config_arc = self.config.clone();
        let window_weak = window.as_weak();
        window.on_reset_config(move || {
            let cfg = config_arc.clone();
            let weak = window_weak.clone();
            tokio::spawn(async move {
                let mut guard = cfg.lock().await;
                *guard = AppConfig::default();
                let _ = guard.save();
                info!("Configuration reset to defaults");

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(win) = weak.upgrade() {
                        win.set_run_enabled(true);
                        win.set_fancyzones_enabled(true);
                        win.set_run_calc_enabled(true);
                        win.set_run_windows_enabled(true);
                        win.set_active_preset(SharedString::from("3 Columns"));
                    }
                });
            });
        });

        window.on_launch_module(move |module_name| {
            let mod_str = module_name.to_string();
            info!("Launching utility module: {}", mod_str);
            let exe_path = std::env::current_exe().unwrap_or_else(|_| "powertoys".into());
            tokio::spawn(async move {
                let _ = std::process::Command::new(exe_path)
                    .arg(&mod_str)
                    .spawn();
            });
        });

        let window_weak = window.as_weak();
        window.on_close_requested(move || {
            if let Some(win) = window_weak.upgrade() {
                let _ = win.hide();
            }
        });

        window.run()?;
        Ok(())
    }
}
