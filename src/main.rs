//! Main entrypoint for pure Rust cmux-native application.

mod agent;
mod config;
mod model;
mod pane;
mod settings;
mod sidebar;
mod split;
mod theme;
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

        // Theme management
        let style_manager = libadwaita::StyleManager::default();
        if config.appearance.theme == "dark" {
            style_manager.set_color_scheme(libadwaita::ColorScheme::PreferDark);
        } else if config.appearance.theme == "light" {
            style_manager.set_color_scheme(libadwaita::ColorScheme::PreferLight);
        } else {
            style_manager.set_color_scheme(libadwaita::ColorScheme::Default);
        }

        let main_win = MainWindow::new(app, config);
        main_win.present();
    });

    app.run();
}
