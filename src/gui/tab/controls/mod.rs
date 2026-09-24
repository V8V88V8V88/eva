mod imp;

use {
    super::BookmarkEditor,
    gtk::{
        glib::{self, Object},
        prelude::{EditableExt, EntryExt},
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

    pub fn set_reload_button_sensitive(&self, sensitive: bool) {
        self.imp()
            .addr_bar
            .set_icon_sensitive(gtk::EntryIconPosition::Secondary, sensitive);
    }

    pub fn addr_bar(&self) -> gtk::Entry {
        self.imp().addr_bar.clone()
    }

    pub fn set_uri(&self, uri: &str) {
        self.imp().addr_bar.set_text(uri);
    }

    pub fn set_bookmark_icon_name(&self, name: &str) {
        self.imp().bookmark_button.set_icon_name(name);
    }

    pub fn set_bookmark_popover(&self, popover: Option<&BookmarkEditor>) {
        self.imp().bookmark_button.set_popover(popover);
    }
}
