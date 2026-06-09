# Wren — User Guide

Wren is a native GTK4 / libadwaita client for managing
[WireGuard](https://www.wireguard.com/) VPN tunnels on Linux.

🇷🇺 Русская версия — [USAGE.ru.md](./USAGE.ru.md).

---

## Install

### Step 1 — make sure WireGuard tools are on your system

Wren is a UI on top of `wg-quick`. The `wg-quick` binary must
exist on the host:

| Distro | Command |
|--------|---------|
| Ubuntu / Debian / Mint | `sudo apt install wireguard-tools` |
| Fedora | `sudo dnf install wireguard-tools` |
| Arch / Manjaro | `sudo pacman -S wireguard-tools` |
| openSUSE | `sudo zypper install wireguard-tools` |

### Step 2 — install Wren itself

1. Open <https://github.com/j0ck4/wren-project/releases> and download
   the latest `wren-vX.Y.Z.flatpak` file.
2. **Double-click** the file. GNOME Software / KDE Discover /
   Discover-equivalent opens, click **Install**.
3. Find **Wren** in your applications menu and launch.

That's it. Wren will pull a one-time GNOME runtime download
(~600 MB) on first install — subsequent Flatpak apps reuse it.

> If double-click does nothing, your distro doesn't ship a
> sideload helper. Open a terminal and run:
> ```
> flatpak install --user ~/Downloads/wren-vX.Y.Z.flatpak
> ```

---

## Using Wren

### First launch

Empty window with a single **Import .conf** button. A
WireGuard configuration file (`.conf`) comes from your VPN
provider or system administrator.

### Import a tunnel

Click **Import .conf**, pick a file. The tunnel appears in the
sidebar. Wren copies the file into its own storage (`0600` perms
so the private key stays private) and auto-renames if the name
exceeds WireGuard's 15-character interface name limit.

### Connect / disconnect

1. Click a tunnel in the sidebar.
2. Click the blue **Connect** button.
3. Enter your account password when polkit asks.
4. Button turns red **Disconnect**, a toast confirms it, and a
   desktop notification fires.

### Detail page

Selecting a tunnel shows:

- **Transfer** — Received / Sent counters in KiB / MiB / GiB,
  refreshed every 2 s. Only visible when the tunnel is active.
- **Interface** — Address, DNS, Listen Port, MTU.
- **Peers** — collapsible row per peer with public key, allowed
  IPs, endpoint, and keepalive. Right-click ▸ Copy works on any
  value.

### Edit a tunnel

Click the **pencil** icon in the header. A dialog opens with
editable Interface fields and a list of peers; trash removes a
peer, **+** in the Peers header adds a new one. **Save** writes
back to disk; **Cancel** discards.

### Delete a tunnel

Click the red **trash** icon in the header. A confirmation dialog
appears; if the tunnel is currently connected it is brought down
first, then its `.conf` is permanently removed from Wren's storage.

### Share via QR

Click the **QR** icon. A dialog shows the tunnel as a QR code —
scan it with the WireGuard mobile app
(*Add tunnel ▸ Create from QR code*). Or click
**Copy Configuration** to put the conf on your clipboard.

### System tray

Wren registers a tray icon when the desktop supports it
(KDE / XFCE / Cinnamon / MATE — out of the box; GNOME — install
[the AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/)).

Right-click the icon for **Show Wren**, a quick toggle list of
tunnels, and **Quit**.

### Autostart at login

**☰ menu** in the sidebar header → **Start at Login**. Toggle
off to remove.

### Quitting with active tunnels

If you close the window with tunnels still up, a dialog asks:

- **Cancel** — keep the window open.
- **Disconnect & Quit** — bring everything down, then close.
- **Quit Anyway** — close the window; tunnels stay running.

### Authentication frequency

The first connect of an admin session asks for a password; the
polkit policy (`auth_admin_keep`) caches that authentication for
~5 minutes, so subsequent connect/disconnect actions inside that
window don't ask again.

> Caveat: inside the Flatpak sandbox the policy file isn't
> installed system-wide, so the prompt may appear every time.
> The native (`apt`) install closes that gap, and is on the
> roadmap for Ubuntu 26.04+.

---

## Troubleshooting

**`wg-quick: command not found` when connecting**
You skipped Step 1 of Install — `sudo apt install wireguard-tools`.

**`config file must be a valid interface name, followed by .conf`**
The filename has weird characters or is over 15 characters. Wren
should rename it automatically; if not, rename it manually under
`~/.config/wren/tunnels/`.

**No tray icon on GNOME**
Install the AppIndicator extension (link above), log out, log
back in.

**`Could not refresh tunnel status`**
Your distro doesn't ship `iproute2` (extremely rare). Install it.

**Tunnel up but no internet**
That's a WireGuard *config* issue, not Wren. Check that your
server is reachable, your keys match the server, and `AllowedIPs`
is sane. `sudo wg show` gives the kernel-level view.

**The password prompt comes up every single click**
You're on the Flatpak version; that's expected (see *Authentication
frequency* above).

---

## File locations

Tunnel configurations live in:

- Flatpak: `~/.var/app/io.github.j0ck4.Wren.Devel/config/wren/tunnels/`

You can copy a `.conf` directly there if you prefer the file
manager over the import dialog.

---

## Reporting bugs

Open an issue at <https://github.com/j0ck4/wren-project/issues>.

Helpful info to include:

- Your distro + desktop environment (`uname -a`,
  `echo $XDG_CURRENT_DESKTOP`)
- The `.conf` you tried to import — **redact the PrivateKey**
- Logs: run from a terminal with
  `RUST_LOG=wren=debug flatpak run io.github.j0ck4.Wren.Devel`
  and paste anything that looks relevant
