use std::sync::Arc;
use crate::compositor::CompositorBridge;

/// Every module (Run, FancyZones, OCR, ColorPicker, etc.) implements this trait.
/// This guarantees complete modularity and independent testing.
#[async_trait::async_trait]
pub trait PowerToyModule: Send + Sync {
    /// Unique identifier of the module (e.g. "run", "fancyzones")
    fn id(&self) -> &'static str;

    /// Human-friendly display name (e.g. "PowerToys Run", "FancyZones")
    fn name(&self) -> &'static str;

    /// Description of what the module does
    fn description(&self) -> &'static str;

    /// Default activation shortcut (e.g. "Super+Space", "Super+Shift+`")
    fn default_hotkey(&self) -> &'static str;

    /// Check if the module is enabled in configuration
    fn is_enabled(&self) -> bool;

    /// Set enabled state
    fn set_enabled(&mut self, enabled: bool);

    /// Initialize the module with a reference to the active compositor bridge
    async fn init(&mut self, compositor: Arc<dyn CompositorBridge>) -> anyhow::Result<()>;

    /// Invoked when the user triggers the module's primary action/overlay
    async fn trigger(&mut self) -> anyhow::Result<()>;

    /// Shutdown and clean up any persistent listeners or overlays
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
