mod core;
mod modules;

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use slint::{Color, ComponentHandle, ModelRc, SharedString, VecModel};

use core::config::AppConfig;
use core::hyprland::HyprlandManager;
use core::ipc::{send_ipc_message, start_ipc_listener};
use modules::color_picker::model::{format_color, hex_to_rgb};
use modules::color_picker::{copy_to_clipboard, pick_color_sync};

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
        app.set_current_hex_clean(SharedString::from(&formats.hex_clean));
        app.set_color_name(SharedString::from(&formats.color_name));
        app.set_current_rgb(SharedString::from(&formats.rgb));
        app.set_current_hsl(SharedString::from(&formats.hsl));
        app.set_current_cmyk(SharedString::from(&formats.cmyk));
        app.set_current_color(Color::from_rgb_u8(formats.r, formats.g, formats.b));
    }
}

fn apply_popup_color(popup: &PickerPopupWindow, hex: &str, history_hexes: &[String]) {
    if let Some(formats) = format_color(hex) {
        popup.set_hex(SharedString::from(&formats.hex_clean));
        popup.set_rgb(SharedString::from(&formats.rgb));
        popup.set_hsl(SharedString::from(&formats.hsl));
        popup.set_hsv(SharedString::from(&formats.hsv));
        popup.set_current_color(Color::from_rgb_u8(formats.r, formats.g, formats.b));

        // Stepped shades
        let shade_colors: Vec<Color> = formats
            .shades
            .iter()
            .map(|&(sr, sg, sb)| Color::from_rgb_u8(sr, sg, sb))
            .collect();
        popup.set_shades(ModelRc::from(Rc::new(VecModel::from(shade_colors))));

        // History dots
        let history_items: Vec<HistoryColor> = history_hexes
            .iter()
            .take(8)
            .map(|h| {
                let (hr, hg, hb) = hex_to_rgb(h).unwrap_or((181, 158, 230));
                HistoryColor {
                    hex: SharedString::from(h),
                    col: Color::from_rgb_u8(hr, hg, hb),
                }
            })
            .collect();
        popup.set_history(ModelRc::from(Rc::new(VecModel::from(history_items))));
        popup.set_show_copied_toast(true);
    }
}

fn setup_popup_callbacks(
    popup: &PickerPopupWindow,
    active_hex: Arc<Mutex<String>>,
    shades_data: Arc<Mutex<Vec<(u8, u8, u8)>>>,
    app_weak: Option<slint::Weak<MainWindow>>,
    cfg: Arc<Mutex<AppConfig>>,
) {
    // Close
    let p_weak = popup.as_weak();
    popup.on_close_requested(move || {
        if let Some(p) = p_weak.upgrade() {
            let _ = p.hide();
        }
    });

    // Settings
    let p_weak = popup.as_weak();
    let aw = app_weak.clone();
    popup.on_settings_requested(move || {
        if let Some(p) = p_weak.upgrade() {
            let _ = p.hide();
        }
        if let Some(ref w_app) = aw {
            if let Some(w) = w_app.upgrade() {
                let _ = w.show();
                return;
            }
        }
        let _ = std::process::Command::new("powertoys").arg("--gui").spawn();
    });

    // Pick again
    let p_weak = popup.as_weak();
    let a_hex = Arc::clone(&active_hex);
    let s_data = Arc::clone(&shades_data);
    let aw_pick = app_weak.clone();
    let cfg_pick = Arc::clone(&cfg);
    popup.on_pick_requested(move || {
        let pw = p_weak.clone();
        let ah = Arc::clone(&a_hex);
        let sd = Arc::clone(&s_data);
        let aw_inner = aw_pick.clone();
        let cfg_inner = Arc::clone(&cfg_pick);

        if let Some(p) = pw.upgrade() {
            let _ = p.hide();
        }

        std::thread::spawn(move || {
            if let Some(new_hex) = pick_color_sync() {
                let history = {
                    let mut config = cfg_inner.lock().unwrap();
                    config.add_history_color(&new_hex);
                    config.color_picker.last_color = new_hex.clone();
                    config.save();
                    config.color_picker.history.clone()
                };

                let nh_copy = new_hex.clone();
                copy_to_clipboard(&new_hex);

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(p) = pw.upgrade() {
                        *ah.lock().unwrap() = nh_copy.clone();
                        if let Some(f) = format_color(&nh_copy) {
                            *sd.lock().unwrap() = f.shades;
                        }
                        apply_popup_color(&p, &nh_copy, &history);
                        let _ = p.show();
                    }
                    if let Some(ref w_app) = aw_inner {
                        if let Some(w) = w_app.upgrade() {
                            apply_color_to_ui(&w, &nh_copy);
                            update_history_ui(&w, &history);
                        }
                    }
                });
            }
        });
    });

    // Copy HEX
    let p_weak = popup.as_weak();
    let a_hex = Arc::clone(&active_hex);
    popup.on_copy_hex(move || {
        let h = a_hex.lock().unwrap().clone();
        copy_to_clipboard(&h);
        if let Some(p) = p_weak.upgrade() {
            p.set_show_copied_toast(true);
        }
    });

    // Copy RGB
    let p_weak = popup.as_weak();
    let a_hex = Arc::clone(&active_hex);
    popup.on_copy_rgb(move || {
        let h = a_hex.lock().unwrap().clone();
        if let Some(f) = format_color(&h) {
            copy_to_clipboard(&f.rgb);
        }
        if let Some(p) = p_weak.upgrade() {
            p.set_show_copied_toast(true);
        }
    });

    // Copy HSL
    let p_weak = popup.as_weak();
    let a_hex = Arc::clone(&active_hex);
    popup.on_copy_hsl(move || {
        let h = a_hex.lock().unwrap().clone();
        if let Some(f) = format_color(&h) {
            copy_to_clipboard(&f.hsl);
        }
        if let Some(p) = p_weak.upgrade() {
            p.set_show_copied_toast(true);
        }
    });

    // Copy HSV
    let p_weak = popup.as_weak();
    let a_hex = Arc::clone(&active_hex);
    popup.on_copy_hsv(move || {
        let h = a_hex.lock().unwrap().clone();
        if let Some(f) = format_color(&h) {
            copy_to_clipboard(&f.hsv);
        }
        if let Some(p) = p_weak.upgrade() {
            p.set_show_copied_toast(true);
        }
    });

    // Select Swatch from dots
    let p_weak = popup.as_weak();
    let a_hex = Arc::clone(&active_hex);
    let s_data = Arc::clone(&shades_data);
    let aw_swatch = app_weak.clone();
    let cfg_swatch = Arc::clone(&cfg);
    popup.on_select_swatch(move |hex_str| {
        let text = hex_str.to_string();
        let mut config = cfg_swatch.lock().unwrap();
        config.color_picker.last_color = text.clone();
        config.save();

        *a_hex.lock().unwrap() = text.clone();
        if let Some(f) = format_color(&text) {
            *s_data.lock().unwrap() = f.shades;
        }
        copy_to_clipboard(&text);
        if let Some(p) = p_weak.upgrade() {
            apply_popup_color(&p, &text, &config.color_picker.history);
        }
        if let Some(ref w_app) = aw_swatch {
            if let Some(w) = w_app.upgrade() {
                apply_color_to_ui(&w, &text);
            }
        }
    });

    // Select Shade from strip
    let p_weak = popup.as_weak();
    let a_hex = Arc::clone(&active_hex);
    let s_data = Arc::clone(&shades_data);
    let aw_shade = app_weak.clone();
    let cfg_shade = Arc::clone(&cfg);
    popup.on_select_shade(move |idx| {
        let shades = s_data.lock().unwrap().clone();
        if let Some(&(sr, sg, sb)) = shades.get(idx as usize) {
            let hex_str = format!("#{:02X}{:02X}{:02X}", sr, sg, sb);
            *a_hex.lock().unwrap() = hex_str.clone();
            copy_to_clipboard(&hex_str);
            let mut config = cfg_shade.lock().unwrap();
            config.color_picker.last_color = hex_str.clone();
            config.save();

            if let Some(p) = p_weak.upgrade() {
                apply_popup_color(&p, &hex_str, &config.color_picker.history);
            }
            if let Some(ref w_app) = aw_shade {
                if let Some(w) = w_app.upgrade() {
                    apply_color_to_ui(&w, &hex_str);
                }
            }
        }
    });
}

fn show_popup_window(initial_hex: &str) {
    if let Ok(popup) = PickerPopupWindow::new() {
        let mut config = AppConfig::load();
        config.add_history_color(initial_hex);
        config.color_picker.last_color = initial_hex.to_string();
        config.save();

        let cfg = Arc::new(Mutex::new(config.clone()));
        let active_hex = Arc::new(Mutex::new(initial_hex.to_string()));
        let shades_data = Arc::new(Mutex::new(
            format_color(initial_hex).map(|f| f.shades).unwrap_or_default(),
        ));

        setup_popup_callbacks(&popup, Arc::clone(&active_hex), Arc::clone(&shades_data), None, Arc::clone(&cfg));

        apply_popup_color(&popup, initial_hex, &config.color_picker.history);
        copy_to_clipboard(initial_hex);

        let _ = popup.run();
    }
}

fn handle_cli_args() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if args[1] == "--popup" && args.len() > 2 {
            let hex = &args[2];
            // If background daemon is running, dispatch to it for instant 0ms display
            if send_ipc_message(&format!("PICK_COLOR:{}", hex)).is_ok() {
                return true;
            }
            show_popup_window(hex);
            return true;
        }

        if args[1] == "--pick" || args[1] == "-p" {
            if let Some(hex) = pick_color_sync() {
                // If background daemon is running, dispatch to it for instant 0ms display
                if send_ipc_message(&format!("PICK_COLOR:{}", hex)).is_ok() {
                    return true;
                }
                show_popup_window(&hex);
            }
            return true;
        }

        if args[1] == "--gui" {
            return false;
        }
    }
    false
}

fn main() -> Result<(), slint::PlatformError> {
    // Check if invoked via global shortcut / CLI flag
    if handle_cli_args() {
        return Ok(());
    }

    let app = MainWindow::new()?;
    let popup = PickerPopupWindow::new()?;
    let cfg = Arc::new(Mutex::new(AppConfig::load()));

    let active_hex = Arc::new(Mutex::new(String::new()));
    let shades_data = Arc::new(Mutex::new(Vec::new()));

    setup_popup_callbacks(
        &popup,
        Arc::clone(&active_hex),
        Arc::clone(&shades_data),
        Some(app.as_weak()),
        Arc::clone(&cfg),
    );

    // Initial setup from configuration
    {
        let config = cfg.lock().unwrap();
        app.set_shortcut_text(SharedString::from(&config.color_picker.shortcut));
        apply_color_to_ui(&app, &config.color_picker.last_color);
        update_history_ui(&app, &config.color_picker.history);

        // Ensure shortcut is registered in Hyprland to invoke `powertoys --pick`
        let _ = HyprlandManager::register_shortcut(&config.color_picker.shortcut, "powertoys --pick");
    }

    // Set up IPC Server listener to receive color updates and show popup instantly
    let app_weak_ipc = app.as_weak();
    let popup_weak_ipc = popup.as_weak();
    let cfg_ipc = Arc::clone(&cfg);
    let active_hex_ipc = Arc::clone(&active_hex);
    let shades_ipc = Arc::clone(&shades_data);

    start_ipc_listener(move |msg| {
        if let Some(hex) = msg.strip_prefix("PICK_COLOR:") {
            let clean_hex = hex.trim().to_uppercase();
            let weak = app_weak_ipc.clone();
            let pop_weak = popup_weak_ipc.clone();
            let cfg_clone = Arc::clone(&cfg_ipc);
            let ah = Arc::clone(&active_hex_ipc);
            let sd = Arc::clone(&shades_ipc);

            let _ = slint::invoke_from_event_loop(move || {
                let mut config = cfg_clone.lock().unwrap();
                config.add_history_color(&clean_hex);
                config.color_picker.last_color = clean_hex.clone();
                config.save();

                if let Some(w) = weak.upgrade() {
                    apply_color_to_ui(&w, &clean_hex);
                    update_history_ui(&w, &config.color_picker.history);

                    let color_name = modules::color_picker::model::format_color(&clean_hex)
                        .map(|f| f.color_name)
                        .unwrap_or_else(|| "Color".to_string());
                    w.set_status_text(SharedString::from(format!("Picked {} ({})", clean_hex, color_name)));
                }

                if let Some(p) = pop_weak.upgrade() {
                    *ah.lock().unwrap() = clean_hex.clone();
                    if let Some(f) = format_color(&clean_hex) {
                        *sd.lock().unwrap() = f.shades;
                    }
                    apply_popup_color(&p, &clean_hex, &config.color_picker.history);
                    copy_to_clipboard(&clean_hex);
                    let _ = p.show();
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

    // Pick color button clicked in GUI -> display popup instantly in-process!
    let app_weak_btn = app.as_weak();
    let popup_weak_btn = popup.as_weak();
    let cfg_btn = Arc::clone(&cfg);
    let ah_btn = Arc::clone(&active_hex);
    let sd_btn = Arc::clone(&shades_data);

    app.on_pick_color_clicked(move || {
        let weak = app_weak_btn.clone();
        let pop_weak = popup_weak_btn.clone();
        let cfg_inner = Arc::clone(&cfg_btn);
        let ah = Arc::clone(&ah_btn);
        let sd = Arc::clone(&sd_btn);

        std::thread::spawn(move || {
            if let Some(hex) = pick_color_sync() {
                let hex_copy = hex.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let mut config = cfg_inner.lock().unwrap();
                    config.add_history_color(&hex_copy);
                    config.color_picker.last_color = hex_copy.clone();
                    config.save();

                    if let Some(w) = weak.upgrade() {
                        apply_color_to_ui(&w, &hex_copy);
                        update_history_ui(&w, &config.color_picker.history);

                        let color_name = modules::color_picker::model::format_color(&hex_copy)
                            .map(|f| f.color_name)
                            .unwrap_or_else(|| "Color".to_string());
                        w.set_status_text(SharedString::from(format!("Picked {} ({})", hex_copy, color_name)));
                    }

                    if let Some(p) = pop_weak.upgrade() {
                        *ah.lock().unwrap() = hex_copy.clone();
                        if let Some(f) = format_color(&hex_copy) {
                            *sd.lock().unwrap() = f.shades;
                        }
                        apply_popup_color(&p, &hex_copy, &config.color_picker.history);
                        copy_to_clipboard(&hex_copy);
                        let _ = p.show();
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
            w.set_status_text(SharedString::from(format!("Copied {} to clipboard", text)));
        }
    });

    // Copy RGB
    let app_weak = app.as_weak();
    app.on_copy_rgb_clicked(move |rgb| {
        let text = rgb.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_status_text(SharedString::from(format!("Copied {} to clipboard", text)));
        }
    });

    // Copy HSL
    let app_weak = app.as_weak();
    app.on_copy_hsl_clicked(move |hsl| {
        let text = hsl.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_status_text(SharedString::from(format!("Copied {} to clipboard", text)));
        }
    });

    // Copy CMYK
    let app_weak = app.as_weak();
    app.on_copy_cmyk_clicked(move |cmyk| {
        let text = cmyk.to_string();
        copy_to_clipboard(&text);
        if let Some(w) = app_weak.upgrade() {
            w.set_status_text(SharedString::from(format!("Copied {} to clipboard", text)));
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

            let color_name = modules::color_picker::model::format_color(&text)
                .map(|f| f.color_name)
                .unwrap_or_else(|| "Color".to_string());
            w.set_status_text(SharedString::from(format!("Selected {} ({})", text, color_name)));
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
                    w.set_status_text(SharedString::from(format!("Bound shortcut '{}' to Hyprland", s)));
                }
            }
            Err(err) => {
                if let Some(w) = app_weak.upgrade() {
                    w.set_status_text(SharedString::from(format!("Failed to bind: {}", err)));
                }
            }
        }
    });

    app.run()
}
