use gtk::{
    glib::{self, subclass::InitializingObject},
    subclass::prelude::*,
    CompositeTemplate,
};

#[derive(CompositeTemplate, Default)]
#[template(file = "input.ui")]
pub struct Input {
    #[template_child]
    pub label: TemplateChild<gtk::Label>,
    #[template_child]
    pub entry: TemplateChild<gtk::Entry>,
}

#[glib::object_subclass]
impl ObjectSubclass for Input {
    const NAME: &'static str = "Input";
    type Type = super::Input;
    type ParentType = gtk::Popover;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Input {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for Input {}
impl PopoverImpl for Input {}
