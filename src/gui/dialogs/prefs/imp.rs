use gtk::{
    glib::{self, subclass::InitializingObject},
    subclass::prelude::*,
    CompositeTemplate,
};

#[derive(CompositeTemplate, Default)]
#[template(file = "prefs.ui")]
pub struct Prefs {
    #[template_child]
    pub homepage: TemplateChild<gtk::Entry>,
    #[template_child]
    pub new_page: TemplateChild<gtk::ComboBoxText>,
    #[template_child]
    pub show_tabs: TemplateChild<gtk::ComboBoxText>,
    #[template_child]
    pub tab_position: TemplateChild<gtk::ComboBoxText>,
    #[template_child]
    pub download_scheme: TemplateChild<gtk::ComboBoxText>,
    #[template_child]
    pub download_location_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub download_location: TemplateChild<gtk::Button>,
    #[template_child]
    pub fg_color: TemplateChild<gtk::ColorButton>,
    #[template_child]
    pub bg_color: TemplateChild<gtk::ColorButton>,
    #[template_child]
    pub pre_fg_color: TemplateChild<gtk::ColorButton>,
    #[template_child]
    pub pre_bg_color: TemplateChild<gtk::ColorButton>,
    #[template_child]
    pub quote_fg_color: TemplateChild<gtk::ColorButton>,
    #[template_child]
    pub quote_bg_color: TemplateChild<gtk::ColorButton>,
    #[template_child]
    pub link_color: TemplateChild<gtk::ColorButton>,
    #[template_child]
    pub hover_color: TemplateChild<gtk::ColorButton>,
    #[template_child]
    pub pg_font: TemplateChild<gtk::FontButton>,
    #[template_child]
    pub pre_font: TemplateChild<gtk::FontButton>,
    #[template_child]
    pub quote_font: TemplateChild<gtk::FontButton>,
    #[template_child]
    pub h1_font: TemplateChild<gtk::FontButton>,
    #[template_child]
    pub h2_font: TemplateChild<gtk::FontButton>,
    #[template_child]
    pub h3_font: TemplateChild<gtk::FontButton>,
}

#[glib::object_subclass]
impl ObjectSubclass for Prefs {
    const NAME: &'static str = "Prefs";
    type Type = super::Prefs;
    type ParentType = gtk::Dialog;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Prefs {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for Prefs {}
impl DialogImpl for Prefs {}
impl WindowImpl for Prefs {}
