use {
    gtk::{
        glib,
        glib::{subclass::Signal, Properties},
        pango::{FontDescription, Style, Weight},
        prelude::*,
        subclass::prelude::*,
    },
    std::{cell::RefCell, sync::LazyLock},
};

mod buffer;
pub use buffer::Buffer;
mod history;
pub(crate) use history::History;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::GemView)]
pub struct GemView {
    pub(crate) history: RefCell<History>,
    pub(crate) buffer: RefCell<Buffer>,
    #[property(get, set)]
    pub(crate) font_paragraph: RefCell<String>,
    #[property(get, set)]
    pub(crate) font_pre: RefCell<String>,
    #[property(get, set)]
    pub(crate) font_quote: RefCell<String>,
    #[property(get, set)]
    pub(crate) font_h1: RefCell<String>,
    #[property(get, set)]
    pub(crate) font_h2: RefCell<String>,
    #[property(get, set)]
    pub(crate) font_h3: RefCell<String>,
    #[property(get, set)]
    pub(crate) paragraph_tag: RefCell<gtk::TextTag>,
    #[property(get, set)]
    pub(crate) h1_tag: RefCell<gtk::TextTag>,
    #[property(get, set)]
    pub(crate) h2_tag: RefCell<gtk::TextTag>,
    #[property(get, set)]
    pub(crate) h3_tag: RefCell<gtk::TextTag>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for GemView {
    const NAME: &'static str = "GemView";
    type Type = super::GemView;
    type ParentType = gtk::TextView;
}

// Trait shared by all GObjects
impl ObjectImpl for GemView {
    fn properties() -> &'static [glib::ParamSpec] {
        Self::derived_properties()
    }

    fn set_property(&self, _id: usize, _value: &glib::Value, _pspec: &glib::ParamSpec) {
        Self::derived_set_property(self, _id, _value, _pspec)
    }

    fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
        Self::derived_property(self, _id, _pspec)
    }

    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        gtk::prelude::TextViewExt::set_editable(&*obj, false);
        obj.set_cursor_visible(false);
        *self.history.borrow_mut() = History::default();
        let buffer = gtk::prelude::TextViewExt::buffer(&*obj);
        let mut font = FontDescription::new();
        font.set_family("Sans");
        font.set_style(Style::Normal);
        font.set_weight(Weight::Book);
        font.set_size(12);
        let normal = buffer
            .create_tag(
                Some("normal"),
                &[
                    ("font", &font.to_string() as &dyn glib::value::ToValue),
                    (
                        "justification",
                        &gtk::Justification::Fill as &dyn glib::value::ToValue,
                    ),
                ],
            )
            .unwrap();
        obj.set_paragraph_tag(normal);
        obj.set_font_paragraph(font.to_string());
        obj.set_font_quote(font.to_string());
        font.set_family("Monospace");
        obj.set_font_pre(font.to_string());
        font.set_family("Sans");
        font.set_weight(Weight::Medium);
        font.set_size(14);
        obj.set_font_h3(font.to_string());
        let h3tag = buffer
            .create_tag(
                Some("h3"),
                &[
                    ("font", &font.to_string() as &dyn glib::value::ToValue),
                    (
                        "justification",
                        &gtk::Justification::Fill as &dyn glib::value::ToValue,
                    ),
                ],
            )
            .unwrap();
        obj.set_h3_tag(h3tag);
        font.set_weight(Weight::Bold);
        font.set_size(16);
        obj.set_font_h2(font.to_string());
        let h2tag = buffer
            .create_tag(
                Some("h2"),
                &[
                    ("font", &font.to_string() as &dyn glib::value::ToValue),
                    (
                        "justification",
                        &gtk::Justification::Fill as &dyn glib::value::ToValue,
                    ),
                ],
            )
            .unwrap();
        obj.set_h2_tag(h2tag);
        font.set_weight(Weight::Heavy);
        font.set_size(18);
        obj.set_font_h1(font.to_string());
        obj.add_actions();
        let h1tag = buffer
            .create_tag(
                Some("h1"),
                &[
                    ("font", &font.to_string() as &dyn glib::value::ToValue),
                    (
                        "justification",
                        &gtk::Justification::Fill as &dyn glib::value::ToValue,
                    ),
                ],
            )
            .unwrap();
        obj.set_h1_tag(h1tag);
        obj.bind_properties();
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
            vec![
                Signal::builder("page-loaded")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("page-load-started")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("page-load-redirect")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("page-load-failed")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-unsupported-scheme")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-download")
                    .param_types([glib::Type::STRING, glib::Type::STRING])
                    .build(),
                Signal::builder("request-new-tab")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-new-window")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-input")
                    .param_types([glib::Type::STRING, glib::Type::STRING])
                    .build(),
                Signal::builder("request-input-sensitive")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-upload")
                    .param_types([glib::Type::STRING])
                    .build(),
            ]
        });
        SIGNALS.as_ref()
    }
}

// Trait shared by all widgets
impl WidgetImpl for GemView {}

impl TextViewImpl for GemView {}
