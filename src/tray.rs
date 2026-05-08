//! System tray (StatusNotifierItem) backed by [`ksni`].
//!
//! Runs on its own thread; communicates with the GTK main loop
//! through an async-channel: the tray pushes [`TrayMessage`]s,
//! [`crate::application::WrenApplication`] consumes them.

use ksni::{Handle, MenuItem, ToolTip, TrayService, menu::StandardItem};

#[derive(Debug, Clone)]
pub enum TrayMessage {
    Activate,
    ToggleTunnel(String),
    Quit,
}

#[derive(Debug, Clone)]
pub struct TunnelEntry {
    pub name: String,
    pub active: bool,
}

pub struct WrenTray {
    pub tunnels: Vec<TunnelEntry>,
    pub tx: async_channel::Sender<TrayMessage>,
}

impl ksni::Tray for WrenTray {
    fn id(&self) -> String {
        "io.github.j0ck4.Wren".into()
    }

    fn title(&self) -> String {
        "Wren".into()
    }

    fn icon_name(&self) -> String {
        if self.tunnels.iter().any(|t| t.active) {
            "network-vpn-symbolic".into()
        } else {
            "network-vpn-disabled-symbolic".into()
        }
    }

    fn tool_tip(&self) -> ToolTip {
        let active = self.tunnels.iter().filter(|t| t.active).count();
        ToolTip {
            icon_name: self.icon_name(),
            icon_pixmap: vec![],
            title: "Wren".into(),
            description: if active == 0 {
                "No active tunnels".into()
            } else {
                format!("{active} tunnel(s) active")
            },
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send_blocking(TrayMessage::Activate);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items: Vec<MenuItem<Self>> = vec![
            StandardItem {
                label: "Show Wren".into(),
                icon_name: "network-vpn-symbolic".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.send_blocking(TrayMessage::Activate);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
        ];

        if self.tunnels.is_empty() {
            items.push(
                StandardItem {
                    label: "No tunnels".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        } else {
            for tun in &self.tunnels {
                let name = tun.name.clone();
                let label = if tun.active {
                    format!("✓ {}", tun.name)
                } else {
                    tun.name.clone()
                };
                items.push(
                    StandardItem {
                        label,
                        activate: Box::new(move |tray: &mut Self| {
                            let _ = tray
                                .tx
                                .send_blocking(TrayMessage::ToggleTunnel(name.clone()));
                        }),
                        ..Default::default()
                    }
                    .into(),
                );
            }
        }

        items.push(MenuItem::Separator);
        items.push(
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit-symbolic".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.send_blocking(TrayMessage::Quit);
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

pub struct WrenTrayHandle {
    handle: Handle<WrenTray>,
}

impl std::fmt::Debug for WrenTrayHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrenTrayHandle").finish_non_exhaustive()
    }
}

impl WrenTrayHandle {
    pub fn set_tunnels(&self, entries: Vec<TunnelEntry>) {
        self.handle.update(move |tray| {
            tray.tunnels = entries;
        });
    }
}

/// Spawns the tray on a dedicated thread.
///
/// The StatusNotifierWatcher D-Bus service is optional — vanilla
/// GNOME doesn't ship one. If the service isn't available, the
/// tray thread exits cleanly with a logged warning and the app
/// keeps working without a tray icon.
pub fn spawn(tx: async_channel::Sender<TrayMessage>) -> WrenTrayHandle {
    let tray = WrenTray {
        tunnels: Vec::new(),
        tx,
    };
    let service = TrayService::new(tray);
    let handle = service.handle();
    std::thread::Builder::new()
        .name("wren-tray".into())
        .spawn(move || {
            if let Err(e) = service.run() {
                tracing::warn!("Tray service unavailable (StatusNotifierWatcher missing?): {e}");
            }
        })
        .expect("spawning tray thread");
    WrenTrayHandle { handle }
}
