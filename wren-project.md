# Wren — WireGuard GUI for Ubuntu

> A native GTK4 / libadwaita client for managing WireGuard tunnels.  
> Language: Rust | Platforms: Ubuntu 24.04 LTS, 26.04 LTS

🇷🇺 Русская версия — [wren-project.ru.md](./wren-project.ru.md).

---

## Context

At the moment there is no native GTK4/libadwaita WireGuard client for Linux. The closest analogue is **Wrenrd** (Go + GTK3, ~960 ⭐), last updated January 2024, with no support for Ubuntu 24.04+. The remaining projects are either abandoned or use web wrappers (Tauri + Electron).

This project fills an empty niche and follows the modern GNOME stack.

---

## Project goals

- Provide a native, visually modern WireGuard client for Ubuntu
- Support Ubuntu 24.04 LTS and 26.04 LTS out of the box
- Distribute as both a `.deb` package and a Flatpak
- Avoid constant password prompts (polkit / pkexec)

---

## Technology stack

| Component       | Choice                       | Rationale                                |
|-----------------|------------------------------|------------------------------------------|
| Language        | Rust                         | Safety, performance, Ubuntu 26.04        |
| UI framework    | gtk4-rs + libadwaita         | Native GNOME stack                       |
| Async runtime   | tokio                        | Background tasks, stats polling          |
| System tray     | ksni                         | StatusNotifierItem (the modern standard) |
| UI construction | Blueprint (optional)         | Declarative XML for GTK4                 |
| Privileges      | polkit / pkexec              | Ubuntu standard for admin rights         |
| Data storage    | glib::user_config_dir        | `~/.config/wren/`                      |
| `.conf` parsing | ini crate + serde            | WireGuard's INI-like format              |

---

## Functional requirements

### MVP (v0.1)

- [ ] Import a `.conf` file via a file-open dialog
- [ ] List of tunnels in the sidebar
- [ ] Connect / disconnect a tunnel with one button
- [ ] Tunnel status (connected / disconnected / error)
- [ ] System tray with a status indicator and menu

### v0.2

- [ ] View tunnel details (IP, DNS, peers, allowed IPs)
- [ ] Traffic statistics (RX / TX) refreshed every 2 s
- [ ] GNOME notifications on connect / disconnect
- [ ] Autostart at login

### v0.3

- [ ] Manage multiple peers (peer list)
- [ ] Edit tunnel configuration in the UI
- [ ] Export config / QR code for mobile devices
- [ ] Flatpak package

---

## Non-functional requirements

- Application startup time: < 300 ms
- Background (tray) memory usage: < 20 MB
- No runtime dependency on libssl, Node.js, or Python
- Wayland and X11 support
- Compliance with the GNOME HIG (Human Interface Guidelines)

---

## Project architecture

```
wren/
├── src/
│   ├── main.rs                 # Entry point, GTK Application
│   ├── app.rs                  # ApplicationWindow, global state
│   ├── ui/
│   │   ├── window.rs           # Main window (AdwApplicationWindow)
│   │   ├── tunnel_list.rs      # Sidebar with the tunnel list
│   │   ├── tunnel_detail.rs    # Selected tunnel details + statistics
│   │   ├── import_dialog.rs    # .conf import dialog
│   │   └── tray.rs             # System tray (ksni)
│   ├── wg/
│   │   ├── manager.rs          # wg-quick up/down via pkexec
│   │   ├── parser.rs           # .conf file parsing
│   │   ├── stats.rs            # Reading statistics (wg show transfer)
│   │   └── monitor.rs          # Background polling of tunnel status
│   └── models/
│       ├── tunnel.rs           # Tunnel, Peer, TunnelStatus structs
│       └── config.rs           # Save/load application settings
├── data/
│   ├── io.github.wren.desktop        # .desktop file
│   ├── io.github.wren.metainfo.xml   # AppStream metadata
│   ├── io.github.wren.gschema.xml    # GSettings schema
│   └── icons/
│       ├── hicolor/scalable/apps/wren.svg
│       └── hicolor/symbolic/apps/wren-symbolic.svg
├── polkit/
│   └── io.github.wren.policy         # Polkit policy for wg-quick
├── packaging/
│   ├── debian/                          # .deb package
│   └── io.github.wren.yml            # Flatpak manifest
└── Cargo.toml
```

---

## Key dependencies (Cargo.toml)

```toml
[dependencies]
gtk          = { version = "0.9",  package = "gtk4",       features = ["v4_10"] }
adw          = { version = "0.7",  package = "libadwaita", features = ["v1_4"]  }
ksni         = "0.2"
tokio        = { version = "1",    features = ["full"] }
serde        = { version = "1",    features = ["derive"] }
toml         = "0.8"
ini          = "1.3"
anyhow       = "1"
once_cell    = "1"
glib         = "0.20"
```

---

## Interaction with WireGuard

The application manages tunnels through system tools, not directly through the kernel:

```
UI Action          →  pkexec wg-quick up   <interface>   # bring the tunnel up
UI Action          →  pkexec wg-quick down <interface>   # bring the tunnel down
Background thread  →  wg show <interface> transfer       # RX/TX statistics
Background thread  →  wg show interfaces                 # list of active ones
```

On import, configuration files are copied into `/etc/wireguard/` via pkexec.

---

## Polkit policy

The file `/usr/share/polkit-1/actions/io.github.wren.policy` allows running `wg-quick` without a constant password prompt for users in the `sudo` group:

```xml
<action id="io.github.wren.manage-tunnels">
  <description>Manage WireGuard tunnels</description>
  <defaults>
    <allow_active>auth_admin_keep</allow_active>
  </defaults>
  <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/wg-quick</annotate>
</action>
```

---

## Roadmap

| Stage | Version | Timeline   | Contents                                        |
|-------|---------|------------|-------------------------------------------------|
| 1     | v0.1    | 4 weeks    | MVP: import, list, connect/disconnect, tray     |
| 2     | v0.2    | +3 weeks   | Statistics, notifications, autostart            |
| 3     | v0.3    | +4 weeks   | Config editor, multi-peer, Flatpak              |
| 4     | v1.0    | +2 weeks   | UI polish, tests, GitHub publication            |

---

## Competitive analysis

| Project           | Language | UI        | Ubuntu 24.04 | Active  | Statistics |
|-------------------|----------|-----------|:------------:|:-------:|:----------:|
| Wrenrd         | Go       | GTK3      | ⚠️ no       | ❌      | ❌         |
| wireguard-gui     | JS+Tauri | WebView   | ✅           | ⚠️      | ❌         |
| Wren           | ?        | ?         | ❌           | ❌      | ❌         |
| **wren (ours)** | **Rust** | **GTK4**  | **✅**       | **✅**  | **✅**     |

---

## System dependencies (runtime)

```
wireguard-tools   # wg, wg-quick
resolvconf        # DNS management
polkit            # privilege prompts
libgtk-4-1        # GTK4
libadwaita-1-0    # Adwaita widgets
```

---

*Document created: May 2026*
