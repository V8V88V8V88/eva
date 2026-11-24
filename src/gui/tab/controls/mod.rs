mod imp;

use {
    super::BookmarkEditor,
    gtk::{
        glib::{self, Object},
        prelude::{EditableExt, EntryExt, IsA, PopoverExt},
        subclass::prelude::*,
    },
};

glib::wrapper! {
    pub struct Controls(ObjectSubclass<imp::Controls>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiteState {
    None,
    Verified,
    Identified,
    Insecure,
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

    pub fn set_site_icon(&self, state: SiteState) {
        let bar = &self.imp().addr_bar;
        let pos = gtk::EntryIconPosition::Primary;
        let (icon, tip) = match state {
            SiteState::None => {
                bar.set_icon_from_icon_name(pos, None);
                return;
            }
            SiteState::Verified => ("security-high-symbolic", "View site information"),
            SiteState::Insecure => ("security-low-symbolic", "Connection is not encrypted"),
            SiteState::Identified => (
                "avatar-default-symbolic",
                "Identified with a client certificate. View site information",
            ),
        };
        bar.set_icon_from_icon_name(pos, Some(icon));
        bar.set_icon_tooltip_text(pos, Some(tip));
    }

    pub fn show_site_popover(&self, content: &impl IsA<gtk::Widget>) {
        let bar = &self.imp().addr_bar;
        if let Some(popover) = self.imp().site_popover.get() {
            popover.set_child(Some(content));
            popover.set_pointing_to(Some(&bar.icon_area(gtk::EntryIconPosition::Primary)));
            popover.popup();
        }
    }

    pub fn hide_site_popover(&self) {
        if let Some(popover) = self.imp().site_popover.get() {
            popover.popdown();
        }
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
