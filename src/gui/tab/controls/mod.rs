mod imp;

use {
    super::{BookmarkEditor, Input},
    gtk::{
        glib::{self, Object},
        prelude::{EditableExt, WidgetExt},
        subclass::prelude::*,
    },
};

glib::wrapper! {
    pub struct Controls(ObjectSubclass<imp::Controls>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for Controls {
    fn default() -> Self {
        Self::new()
    }
}

impl Controls {
    pub fn new() -> Self {
        Object::new()
    }

    pub fn set_back_button_sensitive(&self, sensitive: bool) {
        self.imp().back_button.set_sensitive(sensitive);
    }

    pub fn set_forward_button_sensitive(&self, sensitive: bool) {
        self.imp().forward_button.set_sensitive(sensitive);
    }

    pub fn set_reload_button_sensitive(&self, sensitive: bool) {
        self.imp().reload_button.set_sensitive(sensitive);
    }

    pub fn addr_bar(&self) -> gtk::SearchEntry {
        self.imp().addr_bar.clone()
    }

    pub fn set_uri(&self, uri: &str) {
        self.imp().addr_bar.set_text(uri);
    }

    pub fn set_input_popover(&self, popover: Option<&Input>) {
        self.imp().input_button.set_popover(popover);
    }

    pub fn set_bookmark_icon_name(&self, name: &str) {
        self.imp().bookmark_button.set_icon_name(name);
    }

    pub fn set_bookmark_popover(&self, popover: Option<&BookmarkEditor>) {
        self.imp().bookmark_button.set_popover(popover);
    }
}
