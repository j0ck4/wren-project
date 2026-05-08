use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashSet,
    path::PathBuf,
};

use adw::{prelude::*, subclass::prelude::*};
use gtk::{gio, glib};

use crate::{
    detail::TunnelDetail,
    models::{Tunnel, TunnelObject},
    storage,
    wg::manager,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/j0ck4/Wren/window.ui")]
    pub struct WrenWindow {
        #[template_child]
        pub split_view:          TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub sidebar_stack:       TemplateChild<gtk::Stack>,
        #[template_child]
        pub tunnel_list:         TemplateChild<gtk::ListBox>,
        #[template_child]
        pub import_button:       TemplateChild<gtk::Button>,
        #[template_child]
        pub import_button_empty: TemplateChild<gtk::Button>,
        #[template_child]
        pub content_page:        TemplateChild<adw::NavigationPage>,
        #[template_child]
        pub content_stack:       TemplateChild<gtk::Stack>,
        #[template_child]
        pub tunnel_detail:       TemplateChild<TunnelDetail>,
        #[template_child]
        pub connect_button:      TemplateChild<gtk::Button>,

        pub tunnels:        OnceCell<gio::ListStore>,
        pub active_set:     RefCell<HashSet<String>>,
        pub busy:           Cell<bool>,
        pub selected_name:  RefCell<Option<String>>,
        pub selected_path:  RefCell<Option<PathBuf>>,
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
                #[weak] win,
                move |_| win.open_import_dialog()
            ));
            self.import_button_empty.connect_clicked(glib::clone!(
                #[weak] win,
                move |_| win.open_import_dialog()
            ));

            self.tunnel_list.connect_row_activated(glib::clone!(
                #[weak] win,
                move |_, row| win.show_tunnel_at(row.index())
            ));

            self.connect_button.connect_clicked(glib::clone!(
                #[weak] win,
                move |_| win.toggle_connection()
            ));

            win.refresh_tunnels();
            win.refresh_active_set();
        }
    }

    impl WidgetImpl for WrenWindow {}
    impl WindowImpl for WrenWindow {}
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
    }

    fn show_tunnel_at(&self, index: i32) {
        let Ok(idx) = u32::try_from(index) else { return };
        let Some(item) = self.store().item(idx) else { return };
        let Some(tunnel_obj) = item.downcast_ref::<TunnelObject>() else { return };
        tunnel_obj.with(|t| self.show_tunnel_detail(t));
    }

    fn show_tunnel_detail(&self, tunnel: &Tunnel) {
        let imp = self.imp();
        imp.tunnel_detail.set_tunnel(tunnel);
        imp.content_page.set_title(&tunnel.name);
        imp.content_stack.set_visible_child_name("detail");
        imp.split_view.set_show_content(true);

        *imp.selected_name.borrow_mut() = Some(tunnel.name.clone());
        *imp.selected_path.borrow_mut() = Some(tunnel.config_path.clone());

        self.update_connect_button();
    }

    fn show_placeholder(&self) {
        let imp = self.imp();
        imp.content_page.set_title("Tunnel");
        imp.content_stack.set_visible_child_name("placeholder");
        imp.connect_button.set_visible(false);
        imp.selected_name.borrow_mut().take();
        imp.selected_path.borrow_mut().take();
    }

    fn refresh_active_set(&self) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = win)] self,
            async move {
                match manager::active_interfaces().await {
                    Ok(set) => {
                        *win.imp().active_set.borrow_mut() = set;
                        win.update_connect_button();
                    }
                    Err(e) => tracing::error!("active_interfaces: {e:#}"),
                }
            }
        ));
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

    fn toggle_connection(&self) {
        let imp = self.imp();
        let Some(name) = imp.selected_name.borrow().clone() else { return };
        let Some(path) = imp.selected_path.borrow().clone() else { return };
        let is_active = imp.active_set.borrow().contains(&name);

        imp.busy.set(true);
        self.update_connect_button();

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = win)] self,
            async move {
                let res = if is_active {
                    manager::down(&path).await
                } else {
                    manager::up(&path).await
                };
                if let Err(e) = res {
                    tracing::error!("Toggle ({name}) failed: {e:#}");
                }
                win.imp().busy.set(false);
                win.refresh_active_set();
            }
        ));
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
                #[weak(rename_to = win)] self,
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
                            }
                            Err(e) => tracing::error!("Import failed: {e:#}"),
                        }
                    }
                    Err(e) if e.matches(gtk::DialogError::Dismissed) => {}
                    Err(e) => tracing::error!("FileDialog error: {e}"),
                }
            ),
        );
    }
}
