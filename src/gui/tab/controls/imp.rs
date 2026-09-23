use gtk::{
    glib::{self, subclass::InitializingObject},
    subclass::prelude::*,
    CompositeTemplate,
};

#[derive(CompositeTemplate, Default)]
#[template(file = "controls.ui")]
pub struct Controls {
    #[template_child]
    pub back_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub forward_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub reload_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub addr_bar: TemplateChild<gtk::SearchEntry>,
    #[template_child]
    pub input_button: TemplateChild<gtk::MenuButton>,
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
    }
}

impl WidgetImpl for Controls {}
impl BoxImpl for Controls {}
