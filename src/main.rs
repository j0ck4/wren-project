mod application;
mod autostart;
mod config;
mod detail;
mod models;
mod qr_dialog;
mod storage;
mod tray;
mod wg;
mod window;

use std::path::Path;

use gtk::{gio, glib, prelude::*};
use tracing_subscriber::EnvFilter;

use crate::application::WrenApplication;

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    glib::set_application_name("Wren");

    let resources_path =
        Path::new(config::PKGDATADIR).join(format!("{}.gresource", env!("CARGO_PKG_NAME")));
    let resources = gio::Resource::load(&resources_path).unwrap_or_else(|e| {
        panic!(
            "Failed to load gresource at {}: {e}",
            resources_path.display()
        )
    });
    gio::resources_register(&resources);

    WrenApplication::new().run()
}
