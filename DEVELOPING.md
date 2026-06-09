# Wren — Developer & Power-User Guide

The full version of the Wren documentation. Compared to the
end-user-focused [`USAGE.md`](./USAGE.md), this document also
covers native installation, building from source, file layout
in detail, and the polkit policy.

🇷🇺 Русская версия — [DEVELOPING.ru.md](./DEVELOPING.ru.md).

---

## 1. Installation

### Option A — Flatpak bundle (recommended)

Works on any Linux with Flatpak: Ubuntu, Fedora, Arch, openSUSE, …

```bash
# 1. Add the Flathub remote (skip if already added)
flatpak remote-add --if-not-exists --user flathub \
    https://dl.flathub.org/repo/flathub.flatpakrepo

# 2. Download the latest .flatpak from
#    https://github.com/j0ck4/wren-project/releases
#    (file name looks like `wren-v0.2.0-dev1.flatpak`)

# 3. Install
flatpak install --user ~/Downloads/wren-v0.2.0-dev1.flatpak

# 4. Launch
flatpak run io.github.j0ck4.Wren.Devel
# …or find "Wren" in your apps menu
```

The first install also pulls the GNOME 50 runtime (~600 MB, shared
between all Flatpak apps).

You also need **WireGuard tools** on the host (Wren shells out to
`wg-quick`, which can't run inside the Flatpak sandbox):

```bash
# Debian / Ubuntu
sudo apt install wireguard-tools

# Fedora
sudo dnf install wireguard-tools

# Arch
sudo pacman -S wireguard-tools
```

### Option B — Native install (Ubuntu 24.04+)

```bash
sudo apt install -y meson ninja-build pkg-config \
    libgtk-4-dev libadwaita-1-dev libglib2.0-dev libdbus-1-dev \
    wireguard-tools

# Rust 1.85+ is required. If apt's rustc is too old, install rustup:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

git clone https://github.com/j0ck4/wren-project.git
cd wren-project
meson setup builddir --prefix=/usr --buildtype=release
meson compile -C builddir
sudo meson install -C builddir --no-rebuild
```

After the install, find **Wren** in your applications menu.

### Option C — Build from source as Flatpak

For developers who want to iterate. See [README.md](./README.md).

### Uninstalling

Wren can be installed two ways, so remove it the way you installed
it. The two are independent — if you tried both, you'll have two
**Wren** entries in your apps menu and should run both removals.

**Flatpak** (Option A or C):

```bash
flatpak uninstall --user io.github.j0ck4.Wren.Devel
flatpak uninstall --unused          # drop the now-orphaned runtime
```

**Native** (Option B) — Meson records what it installed and can undo
it from the same build directory:

```bash
cd wren-project
sudo ninja -C builddir uninstall
```

This removes exactly the files Meson placed under `/usr`
(`/usr/bin/wren`, the `.desktop`, metainfo, gresource, icons, and
the polkit policy).

Both commands leave your tunnels and settings untouched. To wipe
those too — note they hold private keys — delete the config dir:

```bash
rm -rf ~/.config/wren                                  # native
rm -rf ~/.var/app/io.github.j0ck4.Wren.Devel           # Flatpak
```

---

## 2. First launch

When Wren starts for the first time you'll see an empty window with
a single **Import .conf** button.

A WireGuard configuration file looks like this (filename ends in
`.conf`):

```ini
[Interface]
PrivateKey = aGVsbG93b3JsZGhlbGxvd29ybGRoZWxsb3dvcmxkaGVsbA=
Address = 10.0.0.2/32
DNS = 1.1.1.1

[Peer]
PublicKey = cHViMQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
AllowedIPs = 0.0.0.0/0
Endpoint = vpn.example.com:51820
PersistentKeepalive = 25
```

Your VPN provider (or sysadmin) gives you this file.

---

## 3. Importing a tunnel

1. Click **Import .conf** (either the big button on the empty page
   or the folder icon in the sidebar header).
2. Pick your `.conf` file.
3. The tunnel appears in the sidebar.

Wren stores tunnels under
`~/.config/wren/tunnels/<name>.conf` (Flatpak: under
`~/.var/app/io.github.j0ck4.Wren.Devel/config/wren/tunnels/`),
with permissions `0600` so the private key isn't world-readable.

WireGuard interface names are limited to **15 characters**. If your
file's basename is longer (e.g. `home-laptop-vpn.conf` ⇒ 16 chars),
Wren automatically renames it to fit
(`home-laptop-vpn.conf` → `home-laptop-vpn`).

---

## 4. Connecting / disconnecting

1. Click a tunnel in the sidebar — its details appear on the right.
2. Click the blue **Connect** button in the top-right.
3. PolicyKit prompts for your password (only the first time per
   admin session — see *Polkit policy* below).
4. The button turns red and reads **Disconnect**. A toast confirms
   *<name> connected* and a desktop notification pops up.

Click **Disconnect** to bring the tunnel down.

If anything fails, you'll see the underlying `wg-quick` error in a
toast and in a desktop notification.

---

## 5. Tunnel detail page

When a tunnel is selected, the right pane shows:

- **Transfer** *(only while active)* — Received / Sent counters in
  KiB / MiB / GiB, refreshed every 2 seconds.
- **Interface** — Address, DNS, Listen Port, MTU.
- **Peers** — one collapsible row per peer. Click to expand and see
  the public key, allowed IPs, endpoint, and keepalive.

Subtitles are selectable, so you can copy any value with
right-click → Copy or Ctrl+C.

---

## 6. Editing & deleting a tunnel

Click the **pencil icon** in the detail header.

A modal dialog opens with editable fields:

- **Interface** group: Private Key, Address, DNS, Listen Port, MTU.
- **Peers** group: one expander per peer with all fields editable.
  Use the **trash icon** on a peer to remove it; use the **+** in
  the Peers header to add a new peer.

Click **Save** to write the changes back to disk and refresh the
detail page; **Cancel** discards them.

The Save button validates that PrivateKey and every peer's
PublicKey are non-empty.

To **delete** a tunnel, click the red **trash icon** in the detail
header. A confirmation dialog appears. If the tunnel is currently
active, Wren brings it down (`wg-quick down` via pkexec) before
removing the `.conf`; if that disconnect fails, the file is kept and
an error toast is shown. Deletion only removes Wren's stored copy
under the tunnels directory — it never touches the kernel interface
beyond that one disconnect.

---

## 7. Sharing a tunnel via QR code

To import the tunnel onto a phone:

1. Click the **QR icon** (next to the pencil) in the detail header.
2. Scan the QR with the WireGuard mobile app (Android or iOS):
   *Add tunnel ▸ Create from QR code*.
3. Or click **Copy Configuration** and paste it into the mobile
   app's *Create from clipboard* flow.

---

## 8. System tray

When you launch Wren, it tries to register a tray icon
(StatusNotifierItem):

- **KDE / XFCE / Cinnamon / MATE** — the icon shows up automatically.
- **GNOME** — install the [AppIndicator and KStatusNotifierItem
  Support extension](https://extensions.gnome.org/extension/615/appindicator-support/),
  log out, log back in.

Right-click the icon for a menu of:

- **Show Wren** — bring the main window forward.
- A list of all tunnels — clicking toggles connect/disconnect on
  the host.
- **Quit** — close the application.

If your desktop has no tray, the app keeps working; only the icon
is missing.

---

## 9. Start at Login

Click the **☰ menu** in the sidebar header → **Start at Login**.

This drops a `.desktop` file into `~/.config/autostart/`, so Wren
launches automatically when you log in. Toggle it off to remove
the entry.

---

## 10. Quitting with active tunnels

If you close the window while one or more tunnels are connected, a
dialog asks:

- **Cancel** — keep the window open.
- **Disconnect & Quit** — bring every active tunnel down, then
  close. (May prompt for the password once.)
- **Quit Anyway** — close the window; tunnels keep running on the
  host (you can take them down later from a terminal with
  `sudo wg-quick down <name>`).

---

## 11. Polkit policy

Wren needs root only to talk to `wg-quick`. The policy file
installed at `/usr/share/polkit-1/actions/io.github.j0ck4.Wren.policy`
sets `auth_admin_keep`, meaning:

> Once you authenticate, subsequent connect/disconnect actions
> within ~5 minutes don't ask again.

Inside Flatpak the policy file isn't installed system-wide, so the
prompt appears every time. Use the native install (Option B above)
or wait for `.deb` packaging on Ubuntu 26.04+ for a smoother flow.

---

## 12. Troubleshooting

**`wg-quick: command not found` when connecting**
You haven't installed `wireguard-tools` on the host. See *Option A*
of the install section.

**`config file must be a valid interface name, followed by .conf`**
The filename has invalid characters or is over 15 chars. Wren
should rename automatically on next start; if it didn't, rename
the file under `~/.config/wren/tunnels/` to a-z / 0-9 / `.-_`,
≤ 15 chars.

**Tray icon missing on GNOME**
Install the AppIndicator extension (see *§8*). Wren itself prints
*Tray service unavailable* in the log and continues working.

**`pkexec` keeps asking for the password**
You're running the Flatpak version. The polkit policy can't be
installed inside the sandbox; switch to a native install for
session-wide caching.

**`Could not refresh tunnel status`**
Wren reads `/sys/class/net` to see which tunnels are up. The host
must have `ip` (in `iproute2`) installed; this is true on every
mainstream distro.

**Tunnel up but no internet**
That's a WireGuard config issue, not Wren. Verify the server is
reachable, the public/private key pair matches, and your
`AllowedIPs` make sense. `sudo wg show` gives the kernel-level
view.

---

## 13. File locations

| What | Path (host install) | Path (Flatpak) |
|------|---------------------|----------------|
| Tunnel configs | `~/.config/wren/tunnels/` | `~/.var/app/io.github.j0ck4.Wren.Devel/config/wren/tunnels/` |
| Autostart entry | `~/.config/autostart/io.github.j0ck4.Wren.desktop` | same (granted by `xdg-config/autostart` permission) |
| Binary | `/usr/bin/wren` | inside the bundle |
| Polkit policy | `/usr/share/polkit-1/actions/io.github.j0ck4.Wren.policy` | not installed |

---

## 14. Reporting bugs

GitHub issues: <https://github.com/j0ck4/wren-project/issues>.

Useful info to include:

- The exact `.conf` (with the private key redacted!)
- Output of `journalctl --user -e | grep wren` or terminal logs
  from `RUST_LOG=wren=debug flatpak run io.github.j0ck4.Wren.Devel`
- `wg-quick --version`, `pkexec --version`, your distro and
  desktop environment
