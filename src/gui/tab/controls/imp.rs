use gtk::{
    glib::{self, clone, subclass::InitializingObject},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

#[derive(CompositeTemplate, Default)]
#[template(file = "controls.ui")]
pub struct Controls {
    #[template_child]
    pub addr_bar: TemplateChild<gtk::Entry>,
    #[template_child]
    pub bookmark_button: TemplateChild<gtk::MenuButton>,
}

#[glib::object_subclass]
impl ObjectSubclass for Controls {
    const NAME: &'static str = "Controls";
    type Type = super::Controls;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Controls {
    fn constructed(&self) {
        self.parent_constructed();
        // The reload button sits inside the end of the address bar
        self.addr_bar.connect_icon_press(|entry, pos| {
            if pos == gtk::EntryIconPosition::Secondary {
                _ = entry.activate_action("win.reload", None);
            }
        });
        // Center the address while it is just being displayed, and align it
        // to the start while it is being edited
        let focus = gtk::EventControllerFocus::new();
        let entry = self.addr_bar.get();
        focus.connect_enter(clone!(
            #[weak]
            entry,
            move |_| EntryExt::set_alignment(&entry, 0.0)
        ));
        focus.connect_leave(clone!(
            #[weak]
            entry,
            move |_| EntryExt::set_alignment(&entry, 0.5)
        ));
        self.addr_bar.add_controller(focus);
    }
}

impl WidgetImpl for Controls {}
impl BoxImpl for Controls {}
