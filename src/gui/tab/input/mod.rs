mod imp;

use gtk::{
    glib::{self, Object},
    prelude::*,
    subclass::prelude::*,
};

glib::wrapper! {
    /// A bar shown at the top of a page when a server asks for input
    pub struct Input(ObjectSubclass<imp::Input>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
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

    /// Shows the bar with the server's prompt. `on_submit` is called with the
    /// text entered, replacing the handler of any previous request.
    pub fn request<F: Fn(&str) + 'static>(&self, meta: &str, visibility: bool, on_submit: F) {
        let imp = self.imp();
        if let Some(id) = imp.submit_handler.take() {
            imp.entry.disconnect(id);
        }
        let id = imp.entry.connect_activate(move |entry| {
            let text = entry.text();
            if !text.is_empty() {
                on_submit(&text);
            }
        });
        imp.submit_handler.replace(Some(id));
        imp.label.set_label(meta);
        imp.entry.set_visibility(visibility);
        imp.entry.set_text("");
        imp.revealer.set_reveal_child(true);
        imp.entry.grab_focus();
    }

    /// Hides the bar
    pub fn dismiss(&self) {
        self.imp().revealer.set_reveal_child(false);
    }
}
