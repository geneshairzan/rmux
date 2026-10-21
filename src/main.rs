//! Main entrypoint for pure Rust cmux-native application.

mod agent;
mod config;
mod model;
mod pane;
mod settings;
mod sidebar;
mod split;
mod theme;
mod tray;
mod vte;
mod window;

use gtk4::prelude::*;

use crate::config::AppConfig;
use crate::window::MainWindow;

fn main() {
    let app = libadwaita::Application::builder()
        .application_id("org.rmux.Terminal")
        .build();

    app.connect_startup(|_| {
        libadwaita::init().unwrap();

        // Load custom CSS
        let provider = gtk4::CssProvider::new();
        provider.load_from_string(crate::window::CUSTOM_CSS);
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });

    app.connect_activate(|app| {
        let config = AppConfig::load();

        // Top-bar indicator owns Show / New Workspace / Settings / Quit.
        // Tray callbacks cross into GTK over a std channel drained by a
        // poller in MainWindow. If no StatusNotifier host exists, the app
        // still runs without it.
        let (tray_tx, tray_rx) = std::sync::mpsc::channel();
        if let Err(e) = crate::tray::RmuxTray::spawn(tray_tx) {
            eprintln!("rmux: top-bar indicator unavailable: {e}");
        }

        // All built-in presets are dark palettes — keep the libadwaita
        // chrome on dark so headerbar/sidebar never flash light.
        let style_manager = libadwaita::StyleManager::default();
        style_manager.set_color_scheme(libadwaita::ColorScheme::PreferDark);

        let main_win = MainWindow::new(app, config, tray_rx);
        main_win.present();
    });

    app.run();
}
