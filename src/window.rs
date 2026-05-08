use std::cell::OnceCell;

use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use crate::{
    models::{Tunnel, TunnelObject},
    storage,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/j0ck4/Wren/window.ui")]
    pub struct WrenWindow {
        #[template_child]
        pub split_view:         TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub sidebar_stack:      TemplateChild<gtk::Stack>,
        #[template_child]
        pub tunnel_list:        TemplateChild<gtk::ListBox>,
        #[template_child]
        pub import_button:      TemplateChild<gtk::Button>,
        #[template_child]
        pub import_button_empty: TemplateChild<gtk::Button>,

        pub tunnels: OnceCell<gio::ListStore>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WrenWindow {
        const NAME: &'static str = "WrenWindow";
        type Type = super::WrenWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
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

            win.refresh_tunnels();
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
        }
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
