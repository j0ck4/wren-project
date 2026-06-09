use std::cell::OnceCell;

use adw::{prelude::*, subclass::prelude::*};
use gtk::{gio, glib};

use crate::{
    autostart, config,
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
            let app = self.obj();
            app.install_actions();
            app.start_tray();
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

    /// Sends a desktop notification.
    pub fn notify(&self, title: &str, body: &str, success: bool) {
        let notification = gio::Notification::new(title);
        notification.set_body(Some(body));
        let icon_name = if success {
            "network-vpn-symbolic"
        } else {
            "dialog-error-symbolic"
        };
        notification.set_icon(&gio::ThemedIcon::new(icon_name));
        self.send_notification(Some("wren-tunnel"), &notification);
    }

    fn install_actions(&self) {
        let about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| app.show_about())
            .build();
        let quit = gio::ActionEntry::builder("quit")
            .activate(|app: &Self, _, _| app.quit())
            .build();
        self.add_action_entries([about, quit]);
        self.set_accels_for_action("app.quit", &["<Primary>q"]);

        // Stateful boolean action shown as a toggle in the
        // primary menu. Clicking the item asks for state
        // change; we apply it on disk and confirm via set_state
        // (or leave it untouched on failure).
        let autostart_action = gio::SimpleAction::new_stateful(
            "autostart",
            None,
            &autostart::is_enabled().to_variant(),
        );
        autostart_action.connect_change_state(glib::clone!(
            #[weak(rename_to = app)]
            self,
            move |action, requested| {
                let Some(requested) = requested else { return };
                let enable: bool = requested.get().unwrap_or(false);
                let res = if enable {
                    autostart::enable()
                } else {
                    autostart::disable()
                };
                match res {
                    Ok(()) => action.set_state(requested),
                    Err(e) => {
                        tracing::error!("autostart toggle: {e:#}");
                        if let Some(window) = app.active_window().and_downcast::<WrenWindow>() {
                            window.show_toast(&format!(
                                "Could not change autostart: {}",
                                e.chain()
                                    .last()
                                    .map_or_else(|| e.to_string(), ToString::to_string)
                            ));
                        }
                    }
                }
            }
        ));
        self.add_action(&autostart_action);
    }

    fn show_about(&self) {
        let dialog = adw::AboutDialog::builder()
            .application_name("Wren")
            .application_icon(config::APP_ID)
            .version(config::VERSION)
            .developer_name("j0ck4")
            .website("https://github.com/j0ck4/wren-project")
            .issue_url("https://github.com/j0ck4/wren-project/issues")
            .copyright("© 2026 j0ck4")
            .license_type(gtk::License::Gpl30)
            .build();
        dialog.present(self.active_window().as_ref());
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
