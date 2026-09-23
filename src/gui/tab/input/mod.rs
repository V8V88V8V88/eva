mod imp;

use gtk::{
    glib::{self, Object},
    prelude::*,
    subclass::prelude::*,
};

glib::wrapper! {
    pub struct Input(ObjectSubclass<imp::Input>)
        @extends gtk::Popover, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
            gtk::Native, gtk::ShortcutManager;
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Object::new()
    }

    pub fn entry(&self) -> gtk::Entry {
        self.imp().entry.clone()
    }

    pub fn set_visibility(&self, visibility: bool) {
        self.imp().entry.set_visibility(visibility);
    }

    pub fn show(&self) {
        self.popup();
    }

    pub fn request(&self, meta: &str) {
        self.imp().label.set_label(meta);
        self.show();
    }
}
