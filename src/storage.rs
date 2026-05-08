//! Tunnel persistence under `~/.config/wren/tunnels/`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use gtk::glib;

use crate::{models::Tunnel, wg::parser};

pub fn config_dir() -> PathBuf {
    glib::user_config_dir().join(env!("CARGO_PKG_NAME"))
}

pub fn tunnels_dir() -> PathBuf {
    config_dir().join("tunnels")
}

/// Copies a `.conf` file into the tunnels directory and parses it.
/// The destination filename is derived from the source's stem; if a
/// tunnel with that name already exists it is overwritten.
pub fn import(src: &Path) -> Result<Tunnel> {
    let stem = src
        .file_stem()
        .ok_or_else(|| anyhow!("source path has no filename: {}", src.display()))?
        .to_string_lossy()
        .into_owned();
    let name = sanitize_name(&stem)?;

    let dir = tunnels_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("creating tunnel dir {}", dir.display()))?;

    let dest = dir.join(format!("{name}.conf"));
    let text = fs::read_to_string(src)
        .with_context(|| format!("reading {}", src.display()))?;
    let config = parser::parse(&text)
        .with_context(|| format!("parsing WireGuard config {}", src.display()))?;

    fs::write(&dest, &text)
        .with_context(|| format!("writing {}", dest.display()))?;

    Ok(Tunnel { name, config_path: dest, config })
}

/// Lists all tunnels currently stored in the config directory.
pub fn list() -> Result<Vec<Tunnel>> {
    let dir = tunnels_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut tunnels = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().is_none_or(|ext| ext != "conf") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("skipping {}: {e}", path.display());
                continue;
            }
        };
        let config = match parser::parse(&text) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("skipping {}: parse error: {e}", path.display());
                continue;
            }
        };
        tunnels.push(Tunnel {
            name:        stem.to_string(),
            config_path: path,
            config,
        });
    }

    tunnels.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tunnels)
}

fn sanitize_name(name: &str) -> Result<String> {
    // WireGuard interface names must match `^[a-zA-Z0-9_=+.-]{1,15}$`,
    // but as a stored filename we are slightly more permissive: only
    // strip path separators and control chars, then truncate.
    let cleaned: String = name
        .chars()
        .filter(|c| !c.is_control() && *c != '/' && *c != '\\')
        .collect();
    if cleaned.is_empty() {
        return Err(anyhow!("tunnel name is empty after sanitisation"));
    }
    Ok(cleaned)
}
