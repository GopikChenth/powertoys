use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub class_name: String,
    pub workspace_id: i32,
    pub geometry: Geometry,
    pub is_floating: bool,
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: i32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub is_focused: bool,
}

/// Abstract bridge for compositor / window manager operations.
/// Implemented by `powertoys-backend-hyprland` on Linux and `powertoys-backend-mock` on Windows.
#[async_trait::async_trait]
pub trait CompositorBridge: Send + Sync {
    /// Get the list of all active windows/clients
    async fn get_windows(&self) -> anyhow::Result<Vec<WindowInfo>>;

    /// Focus/jump to a specific window by its ID
    async fn focus_window(&self, window_id: &str) -> anyhow::Result<()>;

    /// Move and resize a window into a custom zone geometry
    async fn snap_window(&self, window_id: &str, geometry: Geometry) -> anyhow::Result<()>;

    /// Toggle window pin (Always on Top)
    async fn toggle_pin(&self, window_id: &str) -> anyhow::Result<bool>;

    /// Get connected monitors
    async fn get_monitors(&self) -> anyhow::Result<Vec<MonitorInfo>>;

    /// Execute a command through the desktop environment
    async fn spawn(&self, command: &str) -> anyhow::Result<()>;
}
