<p align="center">
  <img src="data/icons/hicolor/scalable/apps/io.github.j0ck4.Wren.svg" width="120" alt="Wren">
</p>

<h1 align="center">Wren</h1>

<p align="center">
  A native WireGuard client for the GNOME desktop — import, connect, and
  manage your tunnels from a clean GTK4 window, no terminal required.
</p>

<p align="center">
  🇷🇺 Русская версия — <a href="./README.ru.md">README.ru.md</a>
</p>

<p align="center">
  <a href="https://github.com/j0ck4/wren/actions/workflows/ci.yml"><img src="https://github.com/j0ck4/wren/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://www.gnu.org/licenses/gpl-3.0"><img src="https://img.shields.io/badge/License-GPLv3-blue.svg" alt="License: GPL v3"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-stable-orange.svg" alt="Rust"></a>
</p>

<p align="center">
  <img src="data/screenshots/wren.png" width="900" alt="Wren main window">
</p>

## About

Wren manages WireGuard VPN tunnels on Linux through a native GTK4 /
libadwaita interface. It is a front-end for the standard `wg-quick`
tools: import the regular `.conf` file your VPN provider or admin gives
you, then bring the tunnel up or down, watch its status, and edit it —
without touching a terminal.

## Features

- **One-click connect / disconnect** through `wg-quick`, authorized via
  polkit (your password is cached for a few minutes, not asked every
  time).
- **Import any standard `.conf`** with a file picker — no editing by
  hand.
- **Live transfer stats** (received / sent) that refresh every two
  seconds while a tunnel is up.
- **Full detail view** — address, DNS, listen port, MTU, and every peer
  with its allowed IPs, endpoint, and keepalive.
- **Built-in editor** — tweak interface fields, add or remove peers,
  all from the UI.
- **Delete tunnels** safely — a confirmation dialog brings the tunnel
  down first if it is active.
- **Share to mobile via QR code** — scan straight into the WireGuard
  app on Android or iOS.
- **System tray** (StatusNotifierItem) with a per-tunnel toggle, plus
  desktop notifications on connect / disconnect.
- **Start at login** with one toggle.
- **Native everywhere** — GTK4 / libadwaita, Wayland and X11, follows
  the GNOME HIG.

## Status

Early development. The design specification lives in
[`wren-project.md`](./wren-project.md).

## Documentation

For **end users** (download → install → use):
- [USAGE.md](./USAGE.md) — End-user guide (English)
- [USAGE.ru.md](./USAGE.ru.md) — User guide (Russian)

For **developers and power users** (native install, building from
source, polkit, file layout):
- [DEVELOPING.md](./DEVELOPING.md) — Developer guide (English)
- [DEVELOPING.ru.md](./DEVELOPING.ru.md) — Developer guide (Russian)

A quick build reference follows below.

## Installing a prebuilt bundle

For users who just want to try Wren without building it.

```bash
# 1. Add Flathub (skip if already done)
flatpak remote-add --if-not-exists --user flathub \
    https://dl.flathub.org/repo/flathub.flatpakrepo

# 2. Download wren-vX.Y.Z.flatpak from GitHub Releases:
#    https://github.com/j0ck4/wren/releases

# 3. Install
flatpak install --user wren-vX.Y.Z.flatpak

# 4. Run
flatpak run io.github.j0ck4.Wren.Devel
```

The bundle is self-contained and runs on any distribution with Flatpak
(Ubuntu, Fedora, Arch, openSUSE…). Dependencies (the GNOME 50 runtime,
~600 MB) are pulled automatically from Flathub on first install.

You also need **WireGuard tools** on the host — Wren shells out to
`wg-quick`:

```bash
sudo apt install wireguard-tools     # Debian / Ubuntu
```

## Building from source

### Via Flatpak (recommended)

```bash
# One-time: add flathub and install the GNOME SDK 50
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.gnome.Sdk//50 org.gnome.Platform//50 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08 \
  org.freedesktop.Sdk.Extension.llvm20//25.08

# Build and run
flatpak-builder --user --install --force-clean build-dir \
  build-aux/flatpak/io.github.j0ck4.Wren.Devel.json
flatpak run io.github.j0ck4.Wren.Devel
```

### Local build (requires system dev packages)

```bash
sudo apt install meson ninja-build pkg-config \
    libgtk-4-dev libadwaita-1-dev libglib2.0-dev libdbus-1-dev cargo
meson setup builddir --buildtype=release
meson compile -C builddir
sudo meson install -C builddir
```

> Rust 1.85+ is required (edition 2024). If your distro's `cargo` is
> older, install [rustup](https://rustup.rs) or use the Flatpak build
> above. See [DEVELOPING.md](./DEVELOPING.md) for details and for how
> to **uninstall** either flavour.

## License

GPL-3.0-or-later — see [LICENSE](./LICENSE).
