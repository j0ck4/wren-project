# TODO

Backlog of things still worth doing on Wren. Loosely ordered by
impact, not by effort.

## Distribution

- [ ] **Submit to Flathub.** Vendor Cargo deps, write
      `io.github.j0ck4.Wren.json` for the stable manifest (no
      `--share=network` allowed at build time), provide
      screenshots in metainfo, open a PR against `flathub/flathub`.
      End users install through GNOME Software / KDE Discover with
      one click; no need to ship the `.flatpak` by hand.
- [ ] **Real `.deb` build.** Currently blocked on Ubuntu 24.04
      shipping `rustc 1.75` while we need ≥ 1.85 (and gtk4-rs
      MSRV is 1.80). Wait for Ubuntu 26.04+ in apt, or amend
      `debian/rules` to use rustup / Flatpak SDK Rust, then
      publish to a PPA.
- [ ] **`v0.2.0` stable release.** Drop the `-dev1` suffix,
      polish the changelog, push the tag — CI publishes the
      bundle automatically.

## Features

- [ ] **Split tunneling UX.** A peer-aware view that lets the
      user pick which AllowedIPs ranges should go through the
      VPN, with an "all traffic" / "LAN only" / custom mode
      toggle.
- [ ] **Auto-reconnect.** Detect when a wireguard interface
      drops handshake (`wg show <iface> latest-handshakes` >
      threshold) and ask `wg-quick down && up`, with backoff.
- [ ] **GeoIP for peer endpoints.** Resolve `Endpoint = host:port`
      to a country / city tag in the detail view. Could use a
      bundled offline DB (mmdb) to avoid network round-trips.
- [ ] **DNS leak indicator.** Compare configured DNS with the
      system resolver; warn if requests are leaking outside the
      tunnel.
- [ ] **Per-tunnel notes / tags.** A free-form text field saved
      alongside the conf so the user remembers which server is
      which.

## UI polish

- [ ] **Toast inside the edit dialog.** Currently save errors
      go to tracing only.
- [ ] **Keyboard shortcuts.** Ctrl+I for import, Ctrl+E for
      edit, Ctrl+Return for connect/disconnect.
- [ ] **Searchable sidebar.** Live filter once there are more
      than ~10 tunnels.
- [ ] **Drag-and-drop import.** Drop a `.conf` onto the window
      to import.
- [ ] **Branded full-color icon by a real designer.** Replace the
      placeholder `W` mark.

## Quality

- [ ] **Pedantic clippy cleanup.** ~30 warnings remain; address
      or `allow` each one with reasoning.
- [ ] **CI clippy with `-D warnings`.** Currently informational.
- [ ] **Integration test for `wg::manager`.** Mock
      `flatpak-spawn`/`pkexec` boundary; covers the parsing of
      `ip link` output.
- [ ] **Locked Cargo.lock for reproducible Flatpak builds.**
      Verify the bundle CI build doesn't drift between runs.

## Localization

- [ ] **`po/ru.po`.** Translate the ~20 user-visible strings into
      Russian.
- [ ] **`po/de.po`, `po/fr.po`, etc.** Once Russian flow is solid.

## Known caveats / docs

- [ ] **Polkit policy inside Flatpak.** Document that the
      `auth_admin_keep` caching only works in the native install,
      not the bundle.
- [ ] **GNOME tray.** Link to the AppIndicator extension from the
      first-run experience, not just `USAGE.md`.

---

Items finished as of `v0.2.0-dev1` are tracked in the git history
and the GitHub Releases changelog.
