//! Tunnel persistence under `~/.config/wren/tunnels/`.

use std::{
    fs,
    os::unix::fs::PermissionsExt,
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
    // Tunnel configs hold private keys; tighten permissions to
    // user-only so wg-quick stops warning about world access.
    fs::set_permissions(&dest, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("chmod 0600 {}", dest.display()))?;

    Ok(Tunnel { name, config_path: dest, config })
}

/// Lists all tunnels currently stored in the config directory.
/// Files whose stem isn't a valid WireGuard interface name are
/// renamed in place before being loaded.
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

        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));

        let path = match canonical_path(&path, stem) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("skipping {}: {e:#}", path.display());
                continue;
            }
        };
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("canonical_path returns a valid stem")
            .to_string();

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
        tunnels.push(Tunnel { name, config_path: path, config });
    }

    tunnels.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tunnels)
}

/// Returns `path` unchanged if its stem is already a valid
/// WireGuard interface name, or renames the file in place to use
/// the sanitised name and returns the new path.
fn canonical_path(path: &Path, stem: &str) -> Result<PathBuf> {
    let sanitised = sanitize_name(stem)?;
    if sanitised == stem {
        return Ok(path.to_path_buf());
    }
    let new_path = path.with_file_name(format!("{sanitised}.conf"));
    if new_path.exists() && new_path != path {
        return Err(anyhow!(
            "cannot rename to {}: target already exists",
            new_path.display()
        ));
    }
    fs::rename(path, &new_path)
        .with_context(|| format!("renaming {} → {}", path.display(), new_path.display()))?;
    tracing::info!("Renamed tunnel: {stem} → {sanitised}");
    Ok(new_path)
}

/// Sanitises an arbitrary string into a valid WireGuard interface
/// name (`^[a-zA-Z0-9_=+.-]{1,15}$`). Invalid characters are
/// replaced with `-`, runs of `-` are collapsed, and the result is
/// truncated to 15 characters.
fn sanitize_name(name: &str) -> Result<String> {
    let mut cleaned = String::with_capacity(name.len());
    let mut last_dash = false;
    for ch in name.chars() {
        let allowed = ch.is_ascii_alphanumeric() || matches!(ch, '_' | '=' | '+' | '.' | '-');
        if allowed {
            cleaned.push(ch);
            last_dash = ch == '-';
        } else if !last_dash {
            cleaned.push('-');
            last_dash = true;
        }
    }
    let trimmed: String = cleaned.trim_matches('-').chars().take(15).collect();
    let trimmed = trimmed.trim_end_matches('-').to_string();
    if trimmed.is_empty() {
        return Err(anyhow!("tunnel name '{name}' has no valid characters"));
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::sanitize_name;

    #[test]
    fn truncates_to_fifteen() {
        assert_eq!(sanitize_name("home-office-laptop").unwrap(), "home-office-lap");
    }

    #[test]
    fn replaces_spaces_and_parens() {
        assert_eq!(
            sanitize_name("work laptop.vpn (1)").unwrap(),
            "work-laptop.vpn"
        );
    }

    #[test]
    fn rejects_empty_after_clean() {
        assert!(sanitize_name("///").is_err());
    }
}

