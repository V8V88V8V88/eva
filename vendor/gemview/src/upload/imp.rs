use gtk::{
    glib::{self, subclass::InitializingObject},
    subclass::prelude::*,
    CompositeTemplate,
};

#[derive(CompositeTemplate, Default)]
#[template(file = "upload_widget.ui")]
pub struct UploadWidget {}

#[glib::object_subclass]
impl ObjectSubclass for UploadWidget {
    const NAME: &'static str = "UploadWidget";
    type Type = super::UploadWidget;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for UploadWidget {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for UploadWidget {}
impl BoxImpl for UploadWidget {}
