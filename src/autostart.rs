//! Manages the user's autostart entry under
//! `~/.config/autostart/<app-id>.desktop`.
//!
//! Inside Flatpak, the host autostart directory is reachable
//! because the manifest grants
//! `--filesystem=xdg-config/autostart:create`. We deliberately
//! construct the path from `$HOME` rather than from
//! `glib::user_config_dir()`, since the latter points at the
//! sandboxed config dir which `gnome-session` doesn't read.

use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};

use crate::config;

pub fn is_enabled() -> bool {
    autostart_file().exists()
}

pub fn enable() -> Result<()> {
    let path = autostart_file();
    let dir = path
        .parent()
        .context("autostart file has no parent directory")?;
    fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    fs::write(&path, content()).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

pub fn disable() -> Result<()> {
    let path = autostart_file();
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
    }
    Ok(())
}

fn autostart_file() -> PathBuf {
    let home = env::var_os("HOME").map_or_else(|| PathBuf::from("/"), PathBuf::from);
    home.join(".config")
        .join("autostart")
        .join(format!("{}.desktop", config::APP_ID))
}

fn content() -> String {
    // When running under Flatpak, the host launcher must call
    // `flatpak run <app-id>`; for a native install the binary
    // is just `wren` on PATH.
    let exec = if env::var_os("FLATPAK_ID").is_some() {
        format!("flatpak run {}", config::APP_ID)
    } else {
        "wren".to_string()
    };
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Wren\n\
         Comment=Native GTK4 WireGuard VPN client\n\
         Exec={exec}\n\
         Icon={app_id}\n\
         X-GNOME-Autostart-enabled=true\n\
         NoDisplay=false\n",
        app_id = config::APP_ID,
    )
}
