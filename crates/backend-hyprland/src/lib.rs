use async_trait::async_trait;
use powertoys_core::{CompositorBridge, Geometry, MonitorInfo, WindowInfo};

pub struct HyprlandBridge;

impl HyprlandBridge {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl CompositorBridge for HyprlandBridge {
    async fn get_windows(&self) -> anyhow::Result<Vec<WindowInfo>> {
        use hyprland::data::Clients;
        use hyprland::shared::HyprData;

        let clients = Clients::get_async().await?;
        let windows = clients
            .into_iter()
            .map(|c| WindowInfo {
                id: c.address.to_string(),
                title: c.title,
                class_name: c.class,
                workspace_id: c.workspace.id,
                geometry: Geometry {
                    x: c.at.0 as i32,
                    y: c.at.1 as i32,
                    width: c.size.0 as u32,
                    height: c.size.1 as u32,
                },
                is_floating: c.floating,
                is_pinned: c.pinned,
            })
            .collect();

        Ok(windows)
    }

    async fn focus_window(&self, window_id: &str) -> anyhow::Result<()> {
        use hyprland::dispatch::{Dispatch, DispatchType, WindowIdentifier};
        use hyprland::shared::Address;
        Dispatch::call_async(DispatchType::FocusWindow(WindowIdentifier::Address(
            Address::new(window_id),
        )))
        .await?;
        Ok(())
    }

    async fn snap_window(&self, window_id: &str, geometry: Geometry) -> anyhow::Result<()> {
        use hyprland::dispatch::{Dispatch, DispatchType, WindowIdentifier};
        use hyprland::shared::Address;

        let addr = WindowIdentifier::Address(Address::new(window_id));
        // Ensure floating first so we can position exactly
        Dispatch::call_async(DispatchType::ToggleFloating(Some(addr))).await?;

        // Move to coordinate
        let move_arg = format!("exact {} {},address:{}", geometry.x, geometry.y, window_id);
        Dispatch::call_async(DispatchType::Custom("movewindowpixel", &move_arg)).await?;

        // Resize to dimensions
        let resize_arg = format!("exact {} {},address:{}", geometry.width, geometry.height, window_id);
        Dispatch::call_async(DispatchType::Custom("resizewindowpixel", &resize_arg)).await?;

        Ok(())
    }

    async fn toggle_pin(&self, window_id: &str) -> anyhow::Result<bool> {
        use hyprland::dispatch::{Dispatch, DispatchType, WindowIdentifier};
        use hyprland::shared::Address;
        Dispatch::call_async(DispatchType::TogglePinWindow(WindowIdentifier::Address(
            Address::new(window_id),
        )))
        .await?;
        Ok(true)
    }

    async fn get_monitors(&self) -> anyhow::Result<Vec<MonitorInfo>> {
        use hyprland::data::Monitors;
        use hyprland::shared::HyprData;

        let monitors = Monitors::get_async().await?;
        let result = monitors
            .into_iter()
            .map(|m| MonitorInfo {
                id: m.id as i32,
                name: m.name,
                width: m.width as u32,
                height: m.height as u32,
                scale: m.scale as f64,
                is_focused: m.focused,
            })
            .collect();

        Ok(result)
    }

    async fn spawn(&self, command: &str) -> anyhow::Result<()> {
        use hyprland::dispatch::{Dispatch, DispatchType};
        Dispatch::call_async(DispatchType::Exec(command)).await?;
        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
#[async_trait]
impl CompositorBridge for HyprlandBridge {
    async fn get_windows(&self) -> anyhow::Result<Vec<WindowInfo>> {
        info!("[HyprlandBridge (Non-Linux Stub)] get_windows called");
        Ok(vec![])
    }

    async fn focus_window(&self, window_id: &str) -> anyhow::Result<()> {
        info!("[HyprlandBridge (Non-Linux Stub)] focus_window: {}", window_id);
        Ok(())
    }

    async fn snap_window(&self, window_id: &str, _geometry: Geometry) -> anyhow::Result<()> {
        info!("[HyprlandBridge (Non-Linux Stub)] snap_window: {}", window_id);
        Ok(())
    }

    async fn toggle_pin(&self, window_id: &str) -> anyhow::Result<bool> {
        info!("[HyprlandBridge (Non-Linux Stub)] toggle_pin: {}", window_id);
        Ok(false)
    }

    async fn get_monitors(&self) -> anyhow::Result<Vec<MonitorInfo>> {
        Ok(vec![])
    }

    async fn spawn(&self, command: &str) -> anyhow::Result<()> {
        info!("[HyprlandBridge (Non-Linux Stub)] spawn: {}", command);
        Ok(())
    }
}
