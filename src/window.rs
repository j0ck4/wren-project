use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashSet,
    path::PathBuf,
};

use adw::{prelude::*, subclass::prelude::*};
use gtk::{gio, glib};

use crate::{
    application::WrenApplication,
    detail::TunnelDetail,
    models::{Tunnel, TunnelObject},
    qr_dialog, storage,
    tray::TunnelEntry,
    wg::manager,
};

/// Trims an `anyhow::Error` chain to the most user-relevant
/// message: keep the deepest non-trivial cause that the host
/// command gave us, dropping our internal `spawning host …`
/// wrappers.
fn friendly_error(e: &anyhow::Error) -> String {
    e.chain()
        .last()
        .map_or_else(|| e.to_string(), |c| c.to_string())
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/j0ck4/Wren/window.ui")]
    pub struct WrenWindow {
        #[template_child]
        pub split_view: TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub sidebar_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub tunnel_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub import_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub import_button_empty: TemplateChild<gtk::Button>,
        #[template_child]
        pub content_page: TemplateChild<adw::NavigationPage>,
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub tunnel_detail: TemplateChild<TunnelDetail>,
        #[template_child]
        pub connect_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub share_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,

        pub tunnels: OnceCell<gio::ListStore>,
        pub active_set: RefCell<HashSet<String>>,
        pub busy: Cell<bool>,
        pub selected_name: RefCell<Option<String>>,
        pub selected_path: RefCell<Option<PathBuf>>,
        pub force_quit: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WrenWindow {
        const NAME: &'static str = "WrenWindow";
        type Type = super::WrenWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            TunnelDetail::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WrenWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let store = gio::ListStore::new::<TunnelObject>();
            self.tunnels
                .set(store.clone())
                .expect("WrenWindow constructed twice");

            self.tunnel_list.bind_model(Some(&store), |item| {
                let tunnel = item
                    .downcast_ref::<TunnelObject>()
                    .expect("ListStore item is a TunnelObject");
                adw::ActionRow::builder()
                    .title(tunnel.name())
                    .activatable(true)
                    .build()
                    .upcast()
            });

            let win = self.obj().clone();
            self.import_button.connect_clicked(glib::clone!(
                #[weak]
                win,
                move |_| win.open_import_dialog()
            ));
            self.import_button_empty.connect_clicked(glib::clone!(
                #[weak]
                win,
                move |_| win.open_import_dialog()
            ));

            self.tunnel_list.connect_row_activated(glib::clone!(
                #[weak]
                win,
                move |_, row| win.show_tunnel_at(row.index())
            ));

            self.connect_button.connect_clicked(glib::clone!(
                #[weak]
                win,
                move |_| win.toggle_selected()
            ));

            self.share_button.connect_clicked(glib::clone!(
                #[weak]
                win,
                move |_| win.show_qr_for_selected()
            ));

            win.refresh_tunnels();
            win.refresh_active_set();
        }
    }

    impl WidgetImpl for WrenWindow {}

    impl WindowImpl for WrenWindow {
        fn close_request(&self) -> glib::Propagation {
            if self.force_quit.get() || self.active_set.borrow().is_empty() {
                return self.parent_close_request();
            }
            self.obj().confirm_quit_with_active();
            glib::Propagation::Stop
        }
    }

    impl ApplicationWindowImpl for WrenWindow {}
    impl AdwApplicationWindowImpl for WrenWindow {}
}

glib::wrapper! {
    pub struct WrenWindow(ObjectSubclass<imp::WrenWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl WrenWindow {
    pub fn new(app: &impl IsA<gtk::Application>) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    fn store(&self) -> gio::ListStore {
        self.imp()
            .tunnels
            .get()
            .expect("tunnels store not initialised")
            .clone()
    }

    fn refresh_tunnels(&self) {
        let store = self.store();
        store.remove_all();
        match storage::list() {
            Ok(tunnels) => {
                tracing::info!("Loaded {} tunnel(s)", tunnels.len());
                for t in tunnels {
                    store.append(&TunnelObject::new(t));
                }
            }
            Err(e) => tracing::error!("Failed to list tunnels: {e:#}"),
        }

        let imp = self.imp();
        if store.n_items() > 0 {
            imp.sidebar_stack.set_visible_child_name("list");
        } else {
            imp.sidebar_stack.set_visible_child_name("empty");
            self.show_placeholder();
        }
        self.push_tray_update();
    }

    fn show_tunnel_at(&self, index: i32) {
        let Ok(idx) = u32::try_from(index) else {
            return;
        };
        let Some(item) = self.store().item(idx) else {
            return;
        };
        let Some(tunnel_obj) = item.downcast_ref::<TunnelObject>() else {
            return;
        };
        tunnel_obj.with(|t| self.show_tunnel_detail(t));
    }

    fn show_tunnel_detail(&self, tunnel: &Tunnel) {
        let imp = self.imp();
        imp.tunnel_detail.set_tunnel(tunnel);
        imp.content_page.set_title(&tunnel.name);
        imp.content_stack.set_visible_child_name("detail");
        imp.split_view.set_show_content(true);
        imp.share_button.set_visible(true);

        *imp.selected_name.borrow_mut() = Some(tunnel.name.clone());
        *imp.selected_path.borrow_mut() = Some(tunnel.config_path.clone());

        let is_active = imp.active_set.borrow().contains(&tunnel.name);
        imp.tunnel_detail.set_active_state(&tunnel.name, is_active);

        self.update_connect_button();
    }

    fn show_placeholder(&self) {
        let imp = self.imp();
        imp.content_page.set_title("Tunnel");
        imp.content_stack.set_visible_child_name("placeholder");
        imp.connect_button.set_visible(false);
        imp.share_button.set_visible(false);
        imp.tunnel_detail.set_active_state("", false);
        imp.selected_name.borrow_mut().take();
        imp.selected_path.borrow_mut().take();
    }

    fn show_qr_for_selected(&self) {
        let imp = self.imp();
        let Some(path) = imp.selected_path.borrow().clone() else {
            return;
        };
        let Some(name) = imp.selected_name.borrow().clone() else {
            return;
        };

        match std::fs::read_to_string(&path) {
            Ok(conf) => qr_dialog::show(self, &name, &conf),
            Err(e) => {
                tracing::error!("read {} for share: {e}", path.display());
                self.toast(&format!("Could not read configuration: {e}"));
            }
        }
    }

    fn refresh_active_set(&self) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = win)]
            self,
            async move {
                match manager::active_interfaces().await {
                    Ok(set) => {
                        *win.imp().active_set.borrow_mut() = set;
                        win.update_connect_button();
                        win.update_detail_active_state();
                        win.push_tray_update();
                    }
                    Err(e) => {
                        tracing::error!("active_interfaces: {e:#}");
                        win.toast(&format!("Could not refresh tunnel status: {e}"));
                    }
                }
            }
        ));
    }

    /// Adds a transient notification at the bottom of the window.
    fn toast(&self, message: &str) {
        self.imp().toast_overlay.add_toast(adw::Toast::new(message));
    }

    pub fn show_toast(&self, message: &str) {
        self.toast(message);
    }

    fn confirm_quit_with_active(&self) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = win)]
            self,
            async move {
                let active: Vec<String> = win.imp().active_set.borrow().iter().cloned().collect();
                let count = active.len();
                let body = format!(
                    "{count} tunnel{plural} {verb} still active and will keep \
                     running after Wren closes.",
                    plural = if count == 1 { "" } else { "s" },
                    verb = if count == 1 { "is" } else { "are" },
                );

                let dialog = adw::AlertDialog::new(Some("Active tunnels"), Some(&body));
                dialog.add_responses(&[
                    ("cancel", "Cancel"),
                    ("disconnect", "Disconnect & Quit"),
                    ("quit", "Quit Anyway"),
                ]);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");
                dialog.set_response_appearance("disconnect", adw::ResponseAppearance::Destructive);

                let response = dialog.choose_future(&win).await;
                match response.as_str() {
                    "quit" => {
                        win.imp().force_quit.set(true);
                        win.close();
                    }
                    "disconnect" => {
                        win.disconnect_all_then_close(active).await;
                    }
                    _ => {}
                }
            }
        ));
    }

    async fn disconnect_all_then_close(&self, names: Vec<String>) {
        let store = self.store();
        let mut paths: Vec<PathBuf> = Vec::with_capacity(names.len());
        for i in 0..store.n_items() {
            let Some(item) = store.item(i) else { continue };
            let Some(t) = item.downcast_ref::<TunnelObject>() else {
                continue;
            };
            if names.contains(&t.name()) {
                paths.push(t.config_path());
            }
        }
        for path in paths {
            if let Err(e) = manager::down(&path).await {
                tracing::error!("disconnect_all_then_close: {}: {e:#}", path.display());
            }
        }
        self.imp().force_quit.set(true);
        self.close();
    }

    fn update_detail_active_state(&self) {
        let imp = self.imp();
        let Some(name) = imp.selected_name.borrow().clone() else {
            return;
        };
        let active = imp.active_set.borrow().contains(&name);
        imp.tunnel_detail.set_active_state(&name, active);
    }

    fn update_connect_button(&self) {
        let imp = self.imp();
        let Some(name) = imp.selected_name.borrow().clone() else {
            imp.connect_button.set_visible(false);
            return;
        };

        let btn = &*imp.connect_button;
        btn.set_visible(true);

        if imp.busy.get() {
            btn.set_sensitive(false);
            btn.set_label("Working…");
            btn.remove_css_class("suggested-action");
            btn.remove_css_class("destructive-action");
            return;
        }

        btn.set_sensitive(true);
        btn.remove_css_class("suggested-action");
        btn.remove_css_class("destructive-action");
        if imp.active_set.borrow().contains(&name) {
            btn.set_label("Disconnect");
            btn.add_css_class("destructive-action");
        } else {
            btn.set_label("Connect");
            btn.add_css_class("suggested-action");
        }
    }

    fn toggle_selected(&self) {
        let imp = self.imp();
        let Some(name) = imp.selected_name.borrow().clone() else {
            return;
        };
        let Some(path) = imp.selected_path.borrow().clone() else {
            return;
        };
        let is_active = imp.active_set.borrow().contains(&name);
        self.run_toggle(name, path, is_active);
    }

    pub fn toggle_tunnel_by_name(&self, target: &str) {
        let store = self.store();
        for i in 0..store.n_items() {
            let Some(item) = store.item(i) else { continue };
            let Some(t) = item.downcast_ref::<TunnelObject>() else {
                continue;
            };
            if t.name() == target {
                let name = t.name();
                let path = t.config_path();
                let is_active = self.imp().active_set.borrow().contains(&name);
                self.run_toggle(name, path, is_active);
                return;
            }
        }
        tracing::warn!("toggle_tunnel_by_name: no tunnel called {target:?}");
    }

    fn run_toggle(&self, name: String, path: PathBuf, is_active: bool) {
        if self.imp().busy.get() {
            tracing::warn!("toggle while busy; ignoring");
            return;
        }
        self.imp().busy.set(true);
        self.update_connect_button();

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = win)]
            self,
            async move {
                let action = if is_active { "Disconnect" } else { "Connect" };
                let res = if is_active {
                    manager::down(&path).await
                } else {
                    manager::up(&path).await
                };
                match &res {
                    Ok(()) => {
                        let verb = if is_active {
                            "disconnected"
                        } else {
                            "connected"
                        };
                        win.toast(&format!("{name} {verb}"));
                        if let Some(app) = win.application().and_downcast::<WrenApplication>() {
                            let title = if is_active {
                                "Tunnel disconnected"
                            } else {
                                "Tunnel connected"
                            };
                            app.notify(title, &name, true);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Toggle ({name}) failed: {e:#}");
                        let summary = friendly_error(e);
                        win.toast(&format!("{action} {name}: {summary}"));
                        if let Some(app) = win.application().and_downcast::<WrenApplication>() {
                            app.notify(
                                &format!("{action} failed"),
                                &format!("{name}: {summary}"),
                                false,
                            );
                        }
                    }
                }
                win.imp().busy.set(false);
                win.refresh_active_set();
            }
        ));
    }

    fn push_tray_update(&self) {
        let Some(app) = self.application().and_downcast::<WrenApplication>() else {
            return;
        };
        let Some(tray) = app.tray() else { return };

        let store = self.store();
        let active = self.imp().active_set.borrow();
        let mut entries = Vec::with_capacity(store.n_items() as usize);
        for i in 0..store.n_items() {
            let Some(item) = store.item(i) else { continue };
            let Some(t) = item.downcast_ref::<TunnelObject>() else {
                continue;
            };
            let name = t.name();
            let is_active = active.contains(&name);
            entries.push(TunnelEntry {
                name,
                active: is_active,
            });
        }
        drop(active);
        tray.set_tunnels(entries);
    }

    fn open_import_dialog(&self) {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("WireGuard configuration (*.conf)"));
        filter.add_pattern("*.conf");

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .title("Import WireGuard Configuration")
            .modal(true)
            .filters(&filters)
            .default_filter(&filter)
            .build();

        dialog.open(
            Some(self),
            None::<&gio::Cancellable>,
            glib::clone!(
                #[weak(rename_to = win)]
                self,
                move |result| match result {
                    Ok(file) => {
                        let Some(path) = file.path() else {
                            tracing::error!("Picked file has no path");
                            return;
                        };
                        match storage::import(&path) {
                            Ok(Tunnel { name, .. }) => {
                                tracing::info!("Imported tunnel {name}");
                                win.refresh_tunnels();
                                win.toast(&format!("Imported {name}"));
                            }
                            Err(e) => {
                                tracing::error!("Import failed: {e:#}");
                                win.toast(&format!(
                                    "Could not import {}: {}",
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
                                    friendly_error(&e)
                                ));
                            }
                        }
                    }
                    Err(e) if e.matches(gtk::DialogError::Dismissed) => {}
                    Err(e) => tracing::error!("FileDialog error: {e}"),
                }
            ),
        );
    }
}
