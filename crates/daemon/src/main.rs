use std::sync::Arc;
use powertoys_core::{CompositorBridge, PowerToyModule};
use powertoys_backend_mock::MockCompositorBridge;
use powertoys_backend_hyprland::HyprlandBridge;
use powertoys_module_run::RunModule;
use powertoys_module_fancyzones::FancyZonesModule;
use powertoys_module_settings::SettingsModule;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let args: Vec<String> = std::env::args().collect();
    let module_name = args.get(1).map(|s| s.as_str()).unwrap_or("settings");

    info!("Starting Linux PowerToys (CachyOS High-Performance Edition)");
    info!("Requested module: '{}'", module_name);

    // Pick backend: Hyprland on Linux, Mock on Windows/Dev
    let compositor: Arc<dyn CompositorBridge> = if cfg!(target_os = "linux") {
        info!("Using native Hyprland IPC backend");
        Arc::new(HyprlandBridge::new())
    } else {
        info!("Non-Linux detected. Using Mock Compositor Bridge for local development");
        Arc::new(MockCompositorBridge::new())
    };

    match module_name {
        "fancyzones" | "zones" => {
            let mut fz = FancyZonesModule::new();
            fz.init(compositor).await?;
            fz.trigger().await?;
        }
        "run" => {
            let mut run = RunModule::new();
            run.init(compositor).await?;
            run.trigger().await?;
        }
        "settings" | "dashboard" | "ui" | _ => {
            let mut settings = SettingsModule::new();
            settings.init(compositor).await?;
            settings.trigger().await?;
        }
    }

    Ok(())
}
