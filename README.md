# Wren

Нативный GTK4 / libadwaita клиент WireGuard для Ubuntu.

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
sudo apt install meson ninja-build pkg-config libgtk-4-dev libadwaita-1-dev
meson setup builddir
meson compile -C builddir
meson install -C builddir
```

## Лицензия

GPL-3.0-or-later — см. [LICENSE](./LICENSE).
