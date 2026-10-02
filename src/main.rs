mod core;
mod modules;

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use slint::{Color, ComponentHandle, ModelRc, SharedString, VecModel};

use core::config::AppConfig;
use core::hyprland::HyprlandManager;
use core::ipc::{send_ipc_message, start_ipc_listener};
use modules::color_picker::model::{format_color, hex_to_rgb};
use modules::color_picker::{copy_to_clipboard, notify, pick_color_sync};

slint::include_modules!();

fn update_history_ui(app: &MainWindow, history_hexes: &[String]) {
    let items: Vec<HistoryColor> = history_hexes
        .iter()
        .map(|hex| {
            let (r, g, b) = hex_to_rgb(hex).unwrap_or((181, 158, 230));
            HistoryColor {
                hex: SharedString::from(hex),
                col: Color::from_rgb_u8(r, g, b),
            }
        })
        .collect();
    let model = Rc::new(VecModel::from(items));
    app.set_history(ModelRc::from(model));
}

fn apply_color_to_ui(app: &MainWindow, hex: &str) {
    if let Some(formats) = format_color(hex) {
        app.set_current_hex(SharedString::from(&formats.hex));
        app.set_current_rgb(SharedString::from(&formats.rgb));
        app.set_current_hsl(SharedString::from(&formats.hsl));
        app.set_current_cmyk(SharedString::from(&formats.cmyk));
        app.set_current_color(Color::from_rgb_u8(formats.r, formats.g, formats.b));
    }
}

fn handle_cli_args() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--pick" || args[1] == "-p") {
        if let Some(hex) = pick_color_sync() {
            // Try to notify the running GUI instance via IPC
            let ipc_msg = format!("PICK_COLOR:{}", hex);
            if send_ipc_message(&ipc_msg).is_err() {
                // GUI is not running; persist to config and notify
                let mut cfg = AppConfig::load();
                cfg.add_history_color(&hex);
                notify("PowerToys Color Picker", &format!("Picked {} (Copied to clipboard)", hex));
            }
        }
        return true;
    }
    false
}

fn main() -> Result<(), slint::PlatformError> {
    // Check if invoked via global shortcut / CLI flag
    if handle_cli_args() {
        return Ok(());
    }

    let app = MainWindow::new()?;
    let cfg = Arc::new(Mutex::new(AppConfig::load()));

    // Initial setup from configuration
    {
        let config = cfg.lock().unwrap();
        app.set_shortcut_text(SharedString::from(&config.color_picker.shortcut));
        apply_color_to_ui(&app, &config.color_picker.last_color);
        update_history_ui(&app, &config.color_picker.history);

        // Ensure shortcut is registered in Hyprland to invoke `powertoys --pick`
        let _ = HyprlandManager::register_shortcut(&config.color_picker.shortcut, "powertoys --pick");
    }

    // Set up IPC Server listener to receive color updates from global shortcuts
    let app_weak_ipc = app.as_weak();
    let cfg_ipc = Arc::clone(&cfg);
    start_ipc_listener(move |msg| {
        if let Some(hex) = msg.strip_prefix("PICK_COLOR:") {
            let clean_hex = hex.trim().to_uppercase();
            let weak = app_weak_ipc.clone();
            let cfg_clone = Arc::clone(&cfg_ipc);

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = weak.upgrade() {
                    apply_color_to_ui(&w, &clean_hex);

                    let mut config = cfg_clone.lock().unwrap();
                    config.add_history_color(&clean_hex);
                    update_history_ui(&w, &config.color_picker.history);

                    w.set_status_text(SharedString::from(format!("✓ Picked & copied {} to clipboard", clean_hex)));
                }
            });
        }
    });

    // Close requested callback
    let app_weak = app.as_weak();
    app.on_close_requested(move || {
        if let Some(w) = app_weak.upgrade() {
            let _ = w.hide();
        }
    });

    // Pick color button clicked in GUI
    let app_weak = app.as_weak();
    let cfg_clone = Arc::clone(&cfg);
    app.on_pick_color_clicked(move || {
        let weak = app_weak.clone();
        let cfg_inner = Arc::clone(&cfg_clone);

        std::thread::spawn(move || {
            if let Some(hex) = pick_color_sync() {
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = weak.upgrade() {
                        apply_color_to_ui(&w, &hex);

                        let mut config = cfg_inner.lock().unwrap();
                        config.add_history_color(&hex);
                        update_history_ui(&w, &config.color_picker.history);

                        w.set_status_text(SharedString::from(format!("✓ Picked & copied {} to clipboard", hex)));
                    }
                });
            }
        });
    });

    // Copy HEX
    let app_weak = app.as_weak();
    app.on_copy_hex_clicked(move |hex| {
        let text = hex.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_status_text(SharedString::from(format!("✓ Copied {} to clipboard", text)));
        }
    });

    // Copy RGB
    let app_weak = app.as_weak();
    app.on_copy_rgb_clicked(move |rgb| {
        let text = rgb.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_status_text(SharedString::from(format!("✓ Copied {} to clipboard", text)));
        }
    });

    // Copy HSL
    let app_weak = app.as_weak();
    app.on_copy_hsl_clicked(move |hsl| {
        let text = hsl.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_status_text(SharedString::from(format!("✓ Copied {} to clipboard", text)));
        }
    });

    // Copy CMYK
    let app_weak = app.as_weak();
    app.on_copy_cmyk_clicked(move |cmyk| {
        let text = cmyk.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_status_text(SharedString::from(format!("✓ Copied {} to clipboard", text)));
        }
    });

    // History swatch selected
    let app_weak = app.as_weak();
    let cfg_clone = Arc::clone(&cfg);
    app.on_swatch_selected(move |hex| {
        let text = hex.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            apply_color_to_ui(&w, &text);
            let mut config = cfg_clone.lock().unwrap();
            config.color_picker.last_color = text.clone();
            config.save();
            w.set_status_text(SharedString::from(format!("✓ Swatch {} selected & copied", text)));
        }
    });

    // Save & apply shortcut
    let app_weak = app.as_weak();
    let cfg_clone = Arc::clone(&cfg);
    app.on_save_shortcut_clicked(move |shortcut| {
        let s = shortcut.to_string();
        match HyprlandManager::register_shortcut(&s, "powertoys --pick") {
            Ok(_) => {
                let mut config = cfg_clone.lock().unwrap();
                config.color_picker.shortcut = s.clone();
                config.save();

                if let Some(w) = app_weak.upgrade() {
                    w.set_shortcut_text(SharedString::from(&s));
                    w.set_status_text(SharedString::from(format!("✓ Bound shortcut '{}' to Hyprland!", s)));
                }
            }
            Err(err) => {
                if let Some(w) = app_weak.upgrade() {
                    w.set_status_text(SharedString::from(format!("✗ Failed to bind: {}", err)));
                }
            }
        }
    });

    app.run()
}
