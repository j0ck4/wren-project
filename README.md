# Wren

A native GTK4 / libadwaita WireGuard client for Ubuntu.

🇷🇺 Русская версия — [README.ru.md](./README.ru.md).

[![CI](https://github.com/j0ck4/wren-project/actions/workflows/ci.yml/badge.svg)](https://github.com/j0ck4/wren-project/actions/workflows/ci.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

## Status

Early development. The specification lives in
[`wren-project.md`](./wren-project.md).

## Documentation

For **end users** (download → install → use):
- 📖 [USAGE.md](./USAGE.md) — End-user guide (English)
- 📖 [USAGE.ru.md](./USAGE.ru.md) — User guide (Russian)

For **developers and power users** (native install, building from
source, polkit, file layout):
- 🛠 [DEVELOPING.md](./DEVELOPING.md) — Developer guide (English)
- 🛠 [DEVELOPING.ru.md](./DEVELOPING.ru.md) — Developer guide (Russian)

A quick build reference follows below.

## Installing a prebuilt bundle

For users who just want to try Wren without building it.

```bash
# 1. Add Flathub (skip if already done)
flatpak remote-add --if-not-exists --user flathub \
    https://dl.flathub.org/repo/flathub.flatpakrepo

# 2. Download wren-vX.Y.Z.flatpak from GitHub Releases:
#    https://github.com/j0ck4/wren-project/releases

# 3. Install
flatpak install --user wren-vX.Y.Z.flatpak

# 4. Run
flatpak run io.github.j0ck4.Wren.Devel
```

The bundle is self-contained and runs on any distribution with Flatpak
(Ubuntu, Fedora, Arch, openSUSE…). Dependencies (the GNOME 50 runtime,
~600 MB) are pulled automatically from Flathub on first install.

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
sudo apt install meson ninja-build pkg-config libgtk-4-dev libadwaita-1-dev cargo
meson setup builddir --buildtype=release
meson compile -C builddir
sudo meson install -C builddir
```

### Building a `.deb`

```bash
sudo apt install debhelper meson ninja-build pkg-config \
    libgtk-4-dev libadwaita-1-dev libglib2.0-dev libssl-dev cargo rustc
dpkg-buildpackage -us -uc -b
sudo apt install ../wren_0.1.0-1_amd64.deb
```

Installing the `.deb` places the polkit policy in
`/usr/share/polkit-1/actions/`, so the admin password is requested once
every 5 minutes (`auth_admin_keep`) rather than on every connection.

## License

GPL-3.0-or-later — see [LICENSE](./LICENSE).
