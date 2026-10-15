#![warn(clippy::all, clippy::pedantic)]
mod prefs;
pub use prefs::Prefs;
use {adw::prelude::AdwDialogExt, gtk::prelude::*, std::env};

#[derive(Clone)]
pub struct Dialogs {
    pub save: gtk::FileChooserDialog,
    pub preferences: Prefs,
}

impl Dialogs {
    pub fn init(window: &gtk::ApplicationWindow) -> Self {
        Self {
            save: Self::init_save(window),
            preferences: Self::init_preferences(window),
        }
    }

    /// Shows the About dialog over `window`
    pub fn show_about(window: &gtk::ApplicationWindow) {
        let about = adw::AboutDialog::builder()
            .application_name("Eva")
            .application_icon("eva")
            .developer_name("Vaibhav Pratap Singh")
            .developers(["Vaibhav Pratap Singh", "Nathan Fisher (original author)"])
            .version(env!("CARGO_PKG_VERSION"))
            .comments("A browser for the SmolWeb, built using Rust and GTK")
            .website(env!("CARGO_PKG_HOMEPAGE"))
            .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues"))
            .copyright("© 2020 Nathan Fisher\n© 2026 Vaibhav Pratap Singh")
            .license_type(gtk::License::Gpl30)
            .debug_info(format!(
                "Eva {}\nGTK {}.{}.{}\nlibadwaita {}.{}.{}\n",
                env!("CARGO_PKG_VERSION"),
                gtk::major_version(),
                gtk::minor_version(),
                gtk::micro_version(),
                adw::major_version(),
                adw::minor_version(),
                adw::micro_version(),
            ))
            .build();
        about.add_acknowledgement_section(
            Some("Built with"),
            &[
                "gtk-rs https://gtk-rs.org",
                "libadwaita-rs https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/",
            ],
        );
        about.present(Some(window));
    }

    fn init_save(window: &gtk::ApplicationWindow) -> gtk::FileChooserDialog {
        let dlg = gtk::FileChooserDialog::builder()
            .use_header_bar(1)
            .destroy_with_parent(true)
            .modal(true)
            .transient_for(window)
            .title("Choose location to save file")
            .action(gtk::FileChooserAction::Save)
            .create_folders(true)
            .build();
        dlg.add_button("Accept", gtk::ResponseType::Accept);
        dlg.add_button("Cancel", gtk::ResponseType::Cancel);
        dlg
    }

    fn init_preferences(window: &gtk::ApplicationWindow) -> Prefs {
        let dlg = Prefs::new();
        dlg.load_config();
        dlg.set_transient_for(Some(window));
        dlg
    }
}
