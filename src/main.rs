use slint::ComponentHandle;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;

    // Request fullscreen mode from the compositor
    app.window().set_fullscreen(true);

    let app_weak = app.as_weak();
    app.on_close_requested(move || {
        if let Some(w) = app_weak.upgrade() {
            let _ = w.hide();
        }
    });

    app.run()
}
