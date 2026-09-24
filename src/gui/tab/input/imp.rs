use {
    adw::subclass::prelude::*,
    gtk::{
        glib::{self, clone, subclass::InitializingObject, SignalHandlerId},
        prelude::*,
        CompositeTemplate,
    },
    std::cell::RefCell,
};

#[derive(CompositeTemplate, Default)]
#[template(file = "input.ui")]
pub struct Input {
    #[template_child]
    pub revealer: TemplateChild<gtk::Revealer>,
    #[template_child]
    pub label: TemplateChild<gtk::Label>,
    #[template_child]
    pub entry: TemplateChild<gtk::Entry>,
    #[template_child]
    pub send: TemplateChild<gtk::Button>,
    #[template_child]
    pub cancel: TemplateChild<gtk::Button>,
    /// Handler for the current request, replaced by each new request
    pub submit_handler: RefCell<Option<SignalHandlerId>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Input {
    const NAME: &'static str = "Input";
    type Type = super::Input;
    type ParentType = adw::Bin;

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
        let obj = self.obj();
        self.send.connect_clicked(clone!(
            #[weak(rename_to = entry)]
            self.entry,
            move |_| entry.emit_activate()
        ));
        self.cancel.connect_clicked(clone!(
            #[weak]
            obj,
            move |_| obj.dismiss()
        ));
        let keys = gtk::EventControllerKey::new();
        keys.connect_key_pressed(clone!(
            #[weak]
            obj,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, _| {
                if key == gtk::gdk::Key::Escape {
                    obj.dismiss();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        self.entry.add_controller(keys);
    }
}

impl WidgetImpl for Input {}
impl BinImpl for Input {}
