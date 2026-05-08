use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use crate::{config, window::WrenWindow};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct WrenApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for WrenApplication {
        const NAME: &'static str = "WrenApplication";
        type Type = super::WrenApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for WrenApplication {}

    impl ApplicationImpl for WrenApplication {
        fn activate(&self) {
            self.parent_activate();
            let app = self.obj();
            let window = app
                .active_window()
                .unwrap_or_else(|| WrenWindow::new(&*app).upcast());
            window.present();
        }
    }

    impl GtkApplicationImpl for WrenApplication {}
    impl AdwApplicationImpl for WrenApplication {}
}

glib::wrapper! {
    pub struct WrenApplication(ObjectSubclass<imp::WrenApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl WrenApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", "/io/github/j0ck4/Wren/")
            .build()
    }
}

impl Default for WrenApplication {
    fn default() -> Self {
        Self::new()
    }
}
