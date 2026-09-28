slint::include_modules!();

use std::rc::Rc;
use std::sync::Arc;
use async_trait::async_trait;
use powertoys_core::{CompositorBridge, Geometry, PowerToyModule};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use tracing::info;

pub struct FancyZonesModule {
    enabled: bool,
    compositor: Option<Arc<dyn CompositorBridge>>,
}

impl FancyZonesModule {
    pub fn new() -> Self {
        Self {
            enabled: true,
            compositor: None,
        }
    }

    fn get_preset_zones(preset: &str) -> Vec<ZoneRect> {
        match preset {
            "2x2 Grid" => vec![
                ZoneRect { id: 1, x_percent: 0.0, y_percent: 0.0, width_percent: 0.5, height_percent: 0.5, label: SharedString::from("Top Left") },
                ZoneRect { id: 2, x_percent: 0.5, y_percent: 0.0, width_percent: 0.5, height_percent: 0.5, label: SharedString::from("Top Right") },
                ZoneRect { id: 3, x_percent: 0.0, y_percent: 0.5, width_percent: 0.5, height_percent: 0.5, label: SharedString::from("Bottom Left") },
                ZoneRect { id: 4, x_percent: 0.5, y_percent: 0.5, width_percent: 0.5, height_percent: 0.5, label: SharedString::from("Bottom Right") },
            ],
            "Priority Grid" => vec![
                ZoneRect { id: 1, x_percent: 0.0, y_percent: 0.0, width_percent: 0.25, height_percent: 1.0, label: SharedString::from("Sidebar Left") },
                ZoneRect { id: 2, x_percent: 0.25, y_percent: 0.0, width_percent: 0.50, height_percent: 1.0, label: SharedString::from("Main Focus") },
                ZoneRect { id: 3, x_percent: 0.75, y_percent: 0.0, width_percent: 0.25, height_percent: 1.0, label: SharedString::from("Sidebar Right") },
            ],
            _ => vec![ // 3 Columns
                ZoneRect { id: 1, x_percent: 0.0, y_percent: 0.0, width_percent: 0.333, height_percent: 1.0, label: SharedString::from("Left (1/3)") },
                ZoneRect { id: 2, x_percent: 0.333, y_percent: 0.0, width_percent: 0.334, height_percent: 1.0, label: SharedString::from("Center (1/3)") },
                ZoneRect { id: 3, x_percent: 0.667, y_percent: 0.0, width_percent: 0.333, height_percent: 1.0, label: SharedString::from("Right (1/3)") },
            ],
        }
    }
}

#[async_trait]
impl PowerToyModule for FancyZonesModule {
    fn id(&self) -> &'static str {
        "fancyzones"
    }

    fn name(&self) -> &'static str {
        "FancyZones"
    }

    fn description(&self) -> &'static str {
        "Window management utility for arranging windows into efficient layouts"
    }

    fn default_hotkey(&self) -> &'static str {
        "Super+Shift+Z"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    async fn init(&mut self, compositor: Arc<dyn CompositorBridge>) -> anyhow::Result<()> {
        self.compositor = Some(compositor);
        info!("FancyZones module initialized");
        Ok(())
    }

    async fn trigger(&mut self) -> anyhow::Result<()> {
        info!("Opening FancyZones visual layout editor");

        let window = FancyZonesEditor::new()?;
        let compositor = self.compositor.clone();

        // Wire preset selector
        let window_weak = window.as_weak();
        window.on_preset_selected(move |preset| {
            if let Some(win) = window_weak.upgrade() {
                let zones = Self::get_preset_zones(preset.as_str());
                win.set_active_preset(preset);
                win.set_zones(ModelRc::from(Rc::new(VecModel::from(zones))));
            }
        });

        // Wire zone snapping action
        let window_weak = window.as_weak();
        let compositor_for_snap = compositor.clone();
        window.on_zone_clicked(move |zone_id| {
            if let Some(win) = window_weak.upgrade() {
                info!("Clicked zone ID: {}", zone_id);
                if let Some(comp) = &compositor_for_snap {
                    let c = comp.clone();
                    let current_zones = win.get_zones();
                    let matching_zone = (0..current_zones.row_count())
                        .filter_map(|i| current_zones.row_data(i))
                        .find(|z| z.id == zone_id);

                    if let Some(z) = matching_zone {
                        tokio::spawn(async move {
                            // Find active window or first client
                            if let Ok(windows) = c.get_windows().await {
                                if let Some(target) = windows.first() {
                                    // Target monitor bounds (default 1920x1080 if not queryable)
                                    let monitors = c.get_monitors().await.unwrap_or_default();
                                    let (mon_w, mon_h) = monitors.first()
                                        .map(|m| (m.width, m.height))
                                        .unwrap_or((1920, 1080));

                                    let snap_geo = Geometry {
                                        x: (mon_w as f32 * z.x_percent) as i32 + 10,
                                        y: (mon_h as f32 * z.y_percent) as i32 + 10,
                                        width: (mon_w as f32 * z.width_percent) as u32 - 20,
                                        height: (mon_h as f32 * z.height_percent) as u32 - 20,
                                    };

                                    info!("Snapping window {} to {:?}", target.id, snap_geo);
                                    let _ = c.snap_window(&target.id, snap_geo).await;
                                }
                            }
                        });
                    }
                }
                let _ = win.hide();
            }
        });

        // Close callback
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
