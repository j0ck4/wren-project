use adw::subclass::prelude::*;
use gtk::{glib, prelude::*};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/j0ck4/Wren/window.ui")]
    pub struct WrenWindow;

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

    impl ObjectImpl for WrenWindow {}
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
}
