use std::cell::OnceCell;

use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use crate::{
    config,
    tray::{self, TrayMessage, WrenTrayHandle},
    window::WrenWindow,
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct WrenApplication {
        pub tray: OnceCell<WrenTrayHandle>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WrenApplication {
        const NAME: &'static str = "WrenApplication";
        type Type = super::WrenApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for WrenApplication {}

    impl ApplicationImpl for WrenApplication {
        fn startup(&self) {
            self.parent_startup();
            self.obj().start_tray();
        }

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

    pub fn tray(&self) -> Option<&WrenTrayHandle> {
        self.imp().tray.get()
    }

    fn start_tray(&self) {
        let (tx, rx) = async_channel::unbounded::<TrayMessage>();
        let handle = tray::spawn(tx);
        let _ = self.imp().tray.set(handle);

        let app = self.clone();
        glib::spawn_future_local(async move {
            while let Ok(msg) = rx.recv().await {
                match msg {
                    TrayMessage::Activate => {
                        if let Some(window) = app.active_window() {
                            window.present();
                        } else {
                            app.activate();
                        }
                    }
                    TrayMessage::ToggleTunnel(name) => {
                        if let Some(window) = app.active_window().and_downcast::<WrenWindow>() {
                            window.toggle_tunnel_by_name(&name);
                        }
                    }
                    TrayMessage::Quit => app.quit(),
                }
            }
        });
    }
}

impl Default for WrenApplication {
    fn default() -> Self {
        Self::new()
    }
}
