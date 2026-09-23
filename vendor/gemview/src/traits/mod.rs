use bucky::gopher;
use gtk::{gdk::Cursor, glib, pango::FontDescription, Label};

pub(crate) trait ToLabel {
    fn to_label(&self, _: &FontDescription) -> Label;
}

impl ToLabel for gopher::parser::Link {
    fn to_label(&self, font: &FontDescription) -> Label {
        gtk::Label::builder()
            .use_markup(true)
            .tooltip_text(self.to_string())
            .label(self.to_markup(font))
            .cursor(&Cursor::from_name("pointer", None).unwrap())
            .build()
    }
}

impl ToLabel for gopher::parser::ExternLink {
    fn to_label(&self, font: &FontDescription) -> Label {
        gtk::Label::builder()
            .selectable(true)
            .use_markup(true)
            .tooltip_text(&self.url)
            .label(self.to_markup(font))
            .cursor(&Cursor::from_name("pointer", None).unwrap())
            .build()
    }
}

pub(crate) trait ToMarkup {
    fn to_markup(&self, _: &FontDescription) -> String;
}

impl ToMarkup for gopher::parser::ExternLink {
    fn to_markup(&self, font: &FontDescription) -> String {
        let url = self.url.replace('&', "&amp;");
        format!(
            "<span color=\"#ff0000\"> 🌐  </span><span font=\"{font}\"><a href=\"{}\">{}</a></span>",
            &url,
            glib::markup_escape_text(&self.display)
        )
    }
}

impl ToMarkup for gopher::parser::Link {
    /// Generates Pango markup from a Gopher link
    fn to_markup(&self, font: &FontDescription) -> String {
        format!(
            "<span color=\"#00ff00\"> 🕳️  </span><span font=\"{font}\"><a href=\"{}\">{}</a></span>",
            &self.to_string().replace(' ', "%20").replace('&', "&amp;"),
            glib::markup_escape_text(&self.display)
        )
    }
}
