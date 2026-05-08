# Wren

Нативный GTK4 / libadwaita клиент WireGuard для Ubuntu.

[![CI](https://github.com/j0ck4/wren-project/actions/workflows/ci.yml/badge.svg)](https://github.com/j0ck4/wren-project/actions/workflows/ci.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

## Состояние

Раннее развитие. Спецификация — в [`wren-project.md`](./wren-project.md).

## Сборка из исходников

### Через Flatpak (рекомендуется)

```bash
# Один раз: добавить flathub и установить GNOME SDK 50
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.gnome.Sdk//50 org.gnome.Platform//50 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08 \
  org.freedesktop.Sdk.Extension.llvm20//25.08

# Сборка и запуск
flatpak-builder --user --install --force-clean build-dir \
  build-aux/flatpak/io.github.j0ck4.Wren.Devel.json
flatpak run io.github.j0ck4.Wren.Devel
```

### Локальная сборка (нужны системные dev-пакеты)

```bash
sudo apt install meson ninja-build pkg-config libgtk-4-dev libadwaita-1-dev cargo
meson setup builddir --buildtype=release
meson compile -C builddir
sudo meson install -C builddir
```

### Сборка `.deb`

```bash
sudo apt install debhelper meson ninja-build pkg-config \
    libgtk-4-dev libadwaita-1-dev libglib2.0-dev libssl-dev cargo rustc
dpkg-buildpackage -us -uc -b
sudo apt install ../wren_0.1.0-1_amd64.deb
```

После установки `.deb` polkit-policy кладётся в `/usr/share/polkit-1/actions/`,
и пароль admin запрашивается один раз в 5 минут (`auth_admin_keep`), а не на
каждое подключение.

## Лицензия

GPL-3.0-or-later — см. [LICENSE](./LICENSE).
