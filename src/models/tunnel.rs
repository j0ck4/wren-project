use std::path::PathBuf;

use gtk::{glib, subclass::prelude::*};

use crate::wg::ParsedConfig;

/// A WireGuard tunnel known to Wren — i.e. a `.conf` file we have
/// imported into our config dir, parsed and ready to use.
#[derive(Debug, Clone)]
pub struct Tunnel {
    pub name: String,
    pub config_path: PathBuf,
    pub config: ParsedConfig,
}

mod imp {
    use std::cell::RefCell;

    use gtk::{glib, subclass::prelude::*};

    use super::Tunnel;

    #[derive(Debug, Default)]
    pub struct TunnelObject {
        pub data: RefCell<Option<Tunnel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TunnelObject {
        const NAME: &'static str = "WrenTunnelObject";
        type Type = super::TunnelObject;
    }

    impl ObjectImpl for TunnelObject {}
}

glib::wrapper! {
    /// `glib::Object` wrapper around [`Tunnel`] so it can live in a
    /// [`gio::ListStore`](gtk::gio::ListStore).
    pub struct TunnelObject(ObjectSubclass<imp::TunnelObject>);
}

impl TunnelObject {
    pub fn new(tunnel: Tunnel) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().data.replace(Some(tunnel));
        obj
    }

    pub fn name(&self) -> String {
        self.with(|t| t.name.clone())
    }

    pub fn config_path(&self) -> PathBuf {
        self.with(|t| t.config_path.clone())
    }

    pub fn with<R>(&self, f: impl FnOnce(&Tunnel) -> R) -> R {
        let borrowed = self.imp().data.borrow();
        let tunnel = borrowed.as_ref().expect("TunnelObject not initialised");
        f(tunnel)
    }
}
