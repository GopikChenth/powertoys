use async_trait::async_trait;
use powertoys_core::{CompositorBridge, Geometry, MonitorInfo, WindowInfo};
use std::sync::Mutex;
use tracing::info;

pub struct MockCompositorBridge {
    windows: Mutex<Vec<WindowInfo>>,
    monitors: Vec<MonitorInfo>,
}

impl MockCompositorBridge {
    pub fn new() -> Self {
        // Pre-populate with realistic mock windows for Windows local development
        let windows = vec![
            WindowInfo {
                id: "client-1".to_string(),
                title: "Visual Studio Code - PowerToys".to_string(),
                class_name: "code".to_string(),
                workspace_id: 1,
                geometry: Geometry { x: 100, y: 100, width: 1200, height: 800 },
                is_floating: false,
                is_pinned: false,
            },
            WindowInfo {
                id: "client-2".to_string(),
                title: "Firefox Developer Edition".to_string(),
                class_name: "firefox".to_string(),
                workspace_id: 2,
                geometry: Geometry { x: 50, y: 50, width: 1400, height: 900 },
                is_floating: false,
                is_pinned: false,
            },
            WindowInfo {
                id: "client-3".to_string(),
                title: "Kitty Terminal - cachyos@hyprland".to_string(),
                class_name: "kitty".to_string(),
                workspace_id: 1,
                geometry: Geometry { x: 200, y: 200, width: 900, height: 600 },
                is_floating: true,
                is_pinned: true,
            },
            WindowInfo {
                id: "client-4".to_string(),
                title: "Spotify Premium".to_string(),
                class_name: "spotify".to_string(),
                workspace_id: 3,
                geometry: Geometry { x: 0, y: 0, width: 1000, height: 700 },
                is_floating: false,
                is_pinned: false,
            },
        ];

        let monitors = vec![MonitorInfo {
            id: 0,
            name: "eDP-1".to_string(),
            width: 1920,
            height: 1080,
            scale: 1.0,
            is_focused: true,
        }];

        Self {
            windows: Mutex::new(windows),
            monitors,
        }
    }
}

#[async_trait]
impl CompositorBridge for MockCompositorBridge {
    async fn get_windows(&self) -> anyhow::Result<Vec<WindowInfo>> {
        let lock = self.windows.lock().unwrap();
        Ok(lock.clone())
    }

    async fn focus_window(&self, window_id: &str) -> anyhow::Result<()> {
        info!("[MockCompositor] Focusing window id: {}", window_id);
        Ok(())
    }

    async fn snap_window(&self, window_id: &str, geometry: Geometry) -> anyhow::Result<()> {
        info!(
            "[MockCompositor] Snapping window '{}' to x:{}, y:{}, w:{}, h:{}",
            window_id, geometry.x, geometry.y, geometry.width, geometry.height
        );
        let mut lock = self.windows.lock().unwrap();
        if let Some(w) = lock.iter_mut().find(|w| w.id == window_id) {
            w.geometry = geometry;
            w.is_floating = true;
        }
        Ok(())
    }

    async fn toggle_pin(&self, window_id: &str) -> anyhow::Result<bool> {
        let mut lock = self.windows.lock().unwrap();
        if let Some(w) = lock.iter_mut().find(|w| w.id == window_id) {
            w.is_pinned = !w.is_pinned;
            info!("[MockCompositor] Toggled pin on window '{}': {}", window_id, w.is_pinned);
            return Ok(w.is_pinned);
        }
        Ok(false)
    }

    async fn get_monitors(&self) -> anyhow::Result<Vec<MonitorInfo>> {
        Ok(self.monitors.clone())
    }

    async fn spawn(&self, command: &str) -> anyhow::Result<()> {
        info!("[MockCompositor] Spawning command: {}", command);
        Ok(())
    }
}
