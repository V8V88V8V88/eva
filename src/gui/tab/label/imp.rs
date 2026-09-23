use gtk::{
    glib::{self, subclass::InitializingObject},
    subclass::prelude::*,
    CompositeTemplate,
};

#[derive(CompositeTemplate, Default)]
#[template(file = "label.ui")]
pub struct Label {
    #[template_child]
    pub label: TemplateChild<gtk::Label>,
    #[template_child]
    pub spinner: TemplateChild<gtk::Spinner>,
    #[template_child]
    pub button: TemplateChild<gtk::Button>,
}

#[glib::object_subclass]
impl ObjectSubclass for Label {
    const NAME: &'static str = "Label";
    type Type = super::Label;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Label {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for Label {}
impl BoxImpl for Label {}
