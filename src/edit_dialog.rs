use std::{
    cell::RefCell,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use adw::{prelude::*, subclass::prelude::*};
use anyhow::{Context, Result, anyhow};
use gtk::glib;

use crate::wg::{
    parser::{Interface, ParsedConfig, Peer},
    serializer,
};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/j0ck4/Wren/edit_dialog.ui")]
    pub struct WrenEditDialog {
        #[template_child]
        pub cancel_button:    TemplateChild<gtk::Button>,
        #[template_child]
        pub save_button:      TemplateChild<gtk::Button>,
        #[template_child]
        pub private_key_row:  TemplateChild<adw::EntryRow>,
        #[template_child]
        pub address_row:      TemplateChild<adw::EntryRow>,
        #[template_child]
        pub dns_row:          TemplateChild<adw::EntryRow>,
        #[template_child]
        pub listen_port_row:  TemplateChild<adw::EntryRow>,
        #[template_child]
        pub mtu_row:          TemplateChild<adw::EntryRow>,
        #[template_child]
        pub peers_group:      TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub add_peer_button:  TemplateChild<gtk::Button>,

        pub config_path: RefCell<Option<PathBuf>>,
        pub peer_rows:   RefCell<Vec<PeerRow>>,
        pub on_saved:    RefCell<Option<Box<dyn Fn()>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WrenEditDialog {
        const NAME: &'static str = "WrenEditDialog";
        type Type = super::WrenEditDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WrenEditDialog {}
    impl WidgetImpl for WrenEditDialog {}
    impl AdwDialogImpl for WrenEditDialog {}
}

glib::wrapper! {
    pub struct WrenEditDialog(ObjectSubclass<imp::WrenEditDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[derive(Debug, Clone)]
pub struct PeerRow {
    pub container:     adw::ExpanderRow,
    pub public_key:    adw::EntryRow,
    pub preshared_key: adw::EntryRow,
    pub allowed_ips:   adw::EntryRow,
    pub endpoint:      adw::EntryRow,
    pub keepalive:     adw::EntryRow,
}

impl WrenEditDialog {
    pub fn new(path: &Path, config: &ParsedConfig) -> Self {
        let dialog: Self = glib::Object::new();
        dialog.imp().config_path.replace(Some(path.to_path_buf()));
        dialog.populate(config);
        dialog.wire_actions();
        dialog
    }

    /// Closure invoked after a successful save (on the GTK
    /// thread). Used by [`crate::window`] to refresh the detail
    /// view.
    pub fn set_on_saved(&self, callback: impl Fn() + 'static) {
        self.imp().on_saved.replace(Some(Box::new(callback)));
    }

    fn populate(&self, config: &ParsedConfig) {
        let imp = self.imp();
        imp.private_key_row.set_text(&config.interface.private_key);
        imp.address_row.set_text(&config.interface.address.join(", "));
        imp.dns_row.set_text(&config.interface.dns.join(", "));
        imp.listen_port_row.set_text(
            &config
                .interface
                .listen_port
                .map(|p| p.to_string())
                .unwrap_or_default(),
        );
        imp.mtu_row.set_text(
            &config
                .interface
                .mtu
                .map(|m| m.to_string())
                .unwrap_or_default(),
        );

        for peer in &config.peers {
            self.append_peer_row(peer);
        }
    }

    fn wire_actions(&self) {
        let imp = self.imp();

        imp.add_peer_button.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.append_peer_row(&Peer::default());
            }
        ));

        imp.cancel_button.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.close();
            }
        ));

        imp.save_button.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                if let Err(e) = dialog.save() {
                    tracing::error!("save: {e:#}");
                    let toast = adw::Toast::new(&format!("Could not save: {e}"));
                    let _ = toast; // No overlay inside the dialog yet.
                    return;
                }
                if let Some(cb) = dialog.imp().on_saved.borrow().as_ref() {
                    cb();
                }
                dialog.close();
            }
        ));
    }

    fn append_peer_row(&self, peer: &Peer) {
        let container = adw::ExpanderRow::builder()
            .title(if peer.public_key.is_empty() {
                "New Peer".to_string()
            } else {
                short_label(&peer.public_key)
            })
            .expanded(peer.public_key.is_empty())
            .build();

        let delete_btn = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Remove Peer")
            .valign(gtk::Align::Center)
            .css_classes(["flat"])
            .build();
        container.add_suffix(&delete_btn);

        let public_key = adw::EntryRow::builder().title("Public Key").build();
        public_key.set_text(&peer.public_key);
        container.add_row(&public_key);

        let preshared_key = adw::EntryRow::builder()
            .title("Preshared Key (optional)")
            .build();
        preshared_key.set_text(peer.preshared_key.as_deref().unwrap_or(""));
        container.add_row(&preshared_key);

        let allowed_ips = adw::EntryRow::builder().title("Allowed IPs").build();
        allowed_ips.set_text(&peer.allowed_ips.join(", "));
        container.add_row(&allowed_ips);

        let endpoint = adw::EntryRow::builder()
            .title("Endpoint (optional)")
            .build();
        endpoint.set_text(peer.endpoint.as_deref().unwrap_or(""));
        container.add_row(&endpoint);

        let keepalive = adw::EntryRow::builder()
            .title("Persistent Keepalive (optional)")
            .input_purpose(gtk::InputPurpose::Number)
            .build();
        keepalive.set_text(
            &peer
                .persistent_keepalive
                .map(|v| v.to_string())
                .unwrap_or_default(),
        );
        container.add_row(&keepalive);

        // Update the row title as the user edits the public key.
        public_key.connect_changed(glib::clone!(
            #[weak]
            container,
            move |entry| {
                let text = entry.text();
                container.set_title(&if text.is_empty() {
                    "New Peer".to_string()
                } else {
                    short_label(&text)
                });
            }
        ));

        let row = PeerRow {
            container:     container.clone(),
            public_key,
            preshared_key,
            allowed_ips,
            endpoint,
            keepalive,
        };

        let imp = self.imp();
        imp.peers_group.add(&container);
        imp.peer_rows.borrow_mut().push(row);

        delete_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            #[weak]
            container,
            move |_| {
                let imp = dialog.imp();
                imp.peers_group.remove(&container);
                imp.peer_rows
                    .borrow_mut()
                    .retain(|r| r.container != container);
            }
        ));
    }

    fn save(&self) -> Result<()> {
        let imp = self.imp();
        let path = imp
            .config_path
            .borrow()
            .clone()
            .ok_or_else(|| anyhow!("no destination path set"))?;

        let private_key = imp.private_key_row.text().to_string();
        if private_key.trim().is_empty() {
            return Err(anyhow!("Interface PrivateKey is required"));
        }

        let interface = Interface {
            private_key,
            address:     split_csv(&imp.address_row.text()),
            dns:         split_csv(&imp.dns_row.text()),
            listen_port: parse_optional(&imp.listen_port_row.text(), "ListenPort")?,
            mtu:         parse_optional(&imp.mtu_row.text(), "MTU")?,
        };

        let mut peers = Vec::new();
        for (i, row) in imp.peer_rows.borrow().iter().enumerate() {
            let public_key = row.public_key.text().to_string();
            if public_key.trim().is_empty() {
                return Err(anyhow!("Peer {} has no Public Key", i + 1));
            }
            peers.push(Peer {
                public_key,
                preshared_key: nonempty(&row.preshared_key.text()),
                allowed_ips:   split_csv(&row.allowed_ips.text()),
                endpoint:      nonempty(&row.endpoint.text()),
                persistent_keepalive: parse_optional(
                    &row.keepalive.text(),
                    "PersistentKeepalive",
                )?,
            });
        }

        let config = ParsedConfig { interface, peers };
        let text = serializer::serialize(&config);

        fs::write(&path, &text).with_context(|| format!("writing {}", path.display()))?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod 0600 {}", path.display()))?;

        tracing::info!("Saved tunnel configuration to {}", path.display());
        Ok(())
    }
}

fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

fn nonempty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_optional<T: std::str::FromStr>(s: &str, key: &str) -> Result<Option<T>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<T>()
        .map(Some)
        .map_err(|_| anyhow!("Invalid value for {key}: {trimmed:?}"))
}

fn short_label(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 16 {
        return key.to_string();
    }
    let head: String = chars.iter().take(6).collect();
    let tail: String = chars
        .iter()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head}…{tail}")
}
