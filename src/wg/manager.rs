//! Bridge to the host's `wireguard-tools`.
//!
//! Wren runs inside a Flatpak sandbox, so every command is executed
//! on the host via `flatpak-spawn --host`. Privileged commands go
//! through `pkexec`, which presents a polkit prompt.

use std::{collections::HashSet, ffi::OsStr, path::Path};

use anyhow::{Context, Result, bail};
use gtk::{gio, prelude::*};

pub async fn up(config_path: &Path) -> Result<()> {
    let path = config_path.to_string_lossy();
    run(&["pkexec", "wg-quick", "up", &path]).await
}

pub async fn down(config_path: &Path) -> Result<()> {
    let path = config_path.to_string_lossy();
    run(&["pkexec", "wg-quick", "down", &path]).await
}

/// Returns the set of network interface names that currently exist
/// on the host. We treat a tunnel as "active" iff its name is in
/// this set. This avoids needing CAP_NET_ADMIN to call `wg show`.
pub async fn active_interfaces() -> Result<HashSet<String>> {
    let out = capture(&["ls", "/sys/class/net"]).await?;
    Ok(out.split_whitespace().map(String::from).collect())
}

async fn run(args: &[&str]) -> Result<()> {
    let argv = host_argv(args);
    let proc = gio::Subprocess::newv(&argv, flags()).context("spawning host command")?;
    let (_, err) = proc
        .communicate_utf8_future(None)
        .await
        .context("waiting for host command")?;
    if !proc.is_successful() {
        bail!(
            "{} failed: {}",
            args.join(" "),
            err.map(|s| s.to_string()).unwrap_or_default().trim()
        );
    }
    Ok(())
}

async fn capture(args: &[&str]) -> Result<String> {
    let argv = host_argv(args);
    let proc = gio::Subprocess::newv(&argv, flags()).context("spawning host command")?;
    let (out, err) = proc
        .communicate_utf8_future(None)
        .await
        .context("waiting for host command")?;
    if !proc.is_successful() {
        bail!(
            "{} failed: {}",
            args.join(" "),
            err.map(|s| s.to_string()).unwrap_or_default().trim()
        );
    }
    Ok(out.map(|s| s.to_string()).unwrap_or_default())
}

fn host_argv<'a>(args: &'a [&'a str]) -> Vec<&'a OsStr> {
    let mut argv: Vec<&OsStr> = Vec::with_capacity(args.len() + 2);
    argv.push(OsStr::new("flatpak-spawn"));
    argv.push(OsStr::new("--host"));
    argv.extend(args.iter().map(|s| OsStr::new(*s)));
    argv
}

fn flags() -> gio::SubprocessFlags {
    gio::SubprocessFlags::STDOUT_PIPE | gio::SubprocessFlags::STDERR_PIPE
}
