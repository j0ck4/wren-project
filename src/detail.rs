use adw::{prelude::*, subclass::prelude::*};
use gtk::glib;

use crate::models::Tunnel;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/j0ck4/Wren/detail.ui")]
    pub struct TunnelDetail {
        #[template_child]
        pub content_box: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TunnelDetail {
        const NAME: &'static str = "WrenTunnelDetail";
        type Type = super::TunnelDetail;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TunnelDetail {}
    impl WidgetImpl for TunnelDetail {}
    impl BinImpl for TunnelDetail {}
}

glib::wrapper! {
    pub struct TunnelDetail(ObjectSubclass<imp::TunnelDetail>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for TunnelDetail {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl TunnelDetail {
    pub fn set_tunnel(&self, tunnel: &Tunnel) {
        let content = &self.imp().content_box;
        while let Some(child) = content.first_child() {
            content.remove(&child);
        }

        content.append(&interface_group(tunnel));
        if !tunnel.config.peers.is_empty() {
            content.append(&peers_group(tunnel));
        }
    }
}

fn interface_group(tunnel: &Tunnel) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Interface")
        .build();

    let iface = &tunnel.config.interface;

    if !iface.address.is_empty() {
        group.add(&property_row("Address", &iface.address.join(", ")));
    }
    if !iface.dns.is_empty() {
        group.add(&property_row("DNS", &iface.dns.join(", ")));
    }
    if let Some(port) = iface.listen_port {
        group.add(&property_row("Listen Port", &port.to_string()));
    }
    if let Some(mtu) = iface.mtu {
        group.add(&property_row("MTU", &mtu.to_string()));
    }

    group
}

fn peers_group(tunnel: &Tunnel) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder().title("Peers").build();

    for (i, peer) in tunnel.config.peers.iter().enumerate() {
        let row = adw::ExpanderRow::builder()
            .title(format!("Peer {}", i + 1))
            .subtitle(short_key(&peer.public_key))
            .build();

        row.add_row(&property_row("Public Key", &peer.public_key));
        if peer.preshared_key.is_some() {
            row.add_row(&property_row("Preshared Key", "(set)"));
        }
        if !peer.allowed_ips.is_empty() {
            row.add_row(&property_row("Allowed IPs", &peer.allowed_ips.join(", ")));
        }
        if let Some(endpoint) = &peer.endpoint {
            row.add_row(&property_row("Endpoint", endpoint));
        }
        if let Some(keepalive) = peer.persistent_keepalive {
            row.add_row(&property_row("Persistent Keepalive", &format!("{keepalive} s")));
        }

        group.add(&row);
    }

    group
}

fn property_row(title: &str, subtitle: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .css_classes(["property"])
        .build();
    row.set_subtitle_selectable(true);
    row
}

fn short_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 16 {
        return key.to_string();
    }
    let head: String = chars.iter().take(6).collect();
    let tail: String = chars.iter().rev().take(6).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{head}…{tail}")
}
