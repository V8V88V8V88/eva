mod imp;

use {
    crate::{
        config::{
            Colors, Config, DownloadScheme, Font, Fonts, General, NewPage, ShowTabs, TabPosition,
        },
        CONFIG,
    },
    gtk::{
        glib::{self, Object},
        prelude::*,
        subclass::prelude::*,
    },
    rgba_simple::RGBA,
};

glib::wrapper! {
    pub struct Prefs(ObjectSubclass<imp::Prefs>)
        @extends gtk::Dialog, gtk::Widget, gtk::Window,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
            gtk::Native, gtk::Root, gtk::ShortcutManager;
}

fn rgba_from_gdk(color: gtk::gdk::RGBA) -> RGBA<u8> {
    RGBA {
        red: (color.red() * 255.0).round() as u8,
        green: (color.green() * 255.0).round() as u8,
        blue: (color.blue() * 255.0).round() as u8,
        alpha: (color.alpha() * 255.0).round() as u8,
    }
}

fn rgba_to_gdk(color: RGBA<u8>) -> gtk::gdk::RGBA {
    gtk::gdk::RGBA::new(
        f32::from(color.red) / 255.0,
        f32::from(color.green) / 255.0,
        f32::from(color.blue) / 255.0,
        f32::from(color.alpha) / 255.0,
    )
}

impl Default for Prefs {
    fn default() -> Self {
        Self::new()
    }
}

impl Prefs {
    pub fn new() -> Self {
        let dlg: Self = Object::builder().property("use-header-bar", 1).build();
        let dialog = dlg.clone();
        dlg.imp().download_scheme.connect_changed(move |_| {
            if let Some(scheme) = dialog.download_scheme() {
                dialog.toggle_download_location(&scheme);
            }
        });
        let dl_location = dlg.init_dl_location();
        dl_location.add_button("Accept", gtk::ResponseType::Accept);
        let dloc = dl_location.clone();
        dlg.imp().download_location.connect_clicked(move |_| {
            dloc.show();
        });
        let button = dlg.imp().download_location.clone();
        dl_location.connect_response(move |dlg, res| {
            if res == gtk::ResponseType::Accept {
                if let Some(file) = dlg.file() {
                    if let Some(path) = file.path() {
                        if let Some(path) = path.to_str() {
                            button.set_label(path);
                        }
                    }
                }
            }
            dlg.hide();
        });
        dlg
    }

    pub fn homepage(&self) -> String {
        self.imp().homepage.buffer().text().to_string()
    }

    pub fn set_homepage(&self, page: &str) {
        self.imp().homepage.buffer().set_text(page);
    }

    pub fn new_page(&self) -> Option<NewPage> {
        if let Some(id) = self.imp().new_page.active_id() {
            match id.as_str() {
                "home" => Some(NewPage::Home),
                "blank" => Some(NewPage::Blank),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn set_new_page(&self, page: &NewPage) {
        self.imp().new_page.set_active_id(match page {
            NewPage::Home => Some("home"),
            NewPage::Blank => Some("blank"),
        });
    }

    pub fn show_tabs(&self) -> Option<ShowTabs> {
        if let Some(id) = self.imp().show_tabs.active_id() {
            match id.as_str() {
                "always" => Some(ShowTabs::Always),
                "multiple" => Some(ShowTabs::Multiple),
                "never" => Some(ShowTabs::Never),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn set_show_tabs(&self, show: &ShowTabs) {
        self.imp().show_tabs.set_active_id(match show {
            ShowTabs::Always => Some("always"),
            ShowTabs::Multiple => Some("multiple"),
            ShowTabs::Never => Some("never"),
        });
    }

    pub fn tab_position(&self) -> Option<TabPosition> {
        if let Some(pos) = self.imp().tab_position.active_id() {
            match pos.as_str() {
                "top" => Some(TabPosition::Top),
                "bottom" => Some(TabPosition::Bottom),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn set_tab_position(&self, pos: &TabPosition) {
        self.imp().tab_position.set_active_id(match pos {
            // The tab bar cannot be placed at the sides; these are shown as top
            TabPosition::Top | TabPosition::Left | TabPosition::Right => Some("top"),
            TabPosition::Bottom => Some("bottom"),
        });
    }

    pub fn download_scheme(&self) -> Option<DownloadScheme> {
        if let Some(scm) = self.imp().download_scheme.active_id() {
            match scm.as_str() {
                "auto" => Some(DownloadScheme::Auto),
                "ask" => Some(DownloadScheme::Ask),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn set_download_scheme(&self, scheme: &DownloadScheme) {
        match scheme {
            DownloadScheme::Ask => {
                self.imp().download_scheme.set_active_id(Some("ask"));
            }
            DownloadScheme::Auto => {
                self.imp().download_scheme.set_active_id(Some("auto"));
            }
        }
        self.toggle_download_location(scheme);
    }

    fn toggle_download_location(&self, scheme: &DownloadScheme) {
        match scheme {
            DownloadScheme::Ask => {
                self.imp().download_location_label.hide();
                self.imp().download_location.hide();
            }
            DownloadScheme::Auto => {
                self.imp().download_location_label.show();
                self.imp().download_location.show();
            }
        }
    }

    fn download_location(&self) -> Option<String> {
        self.imp()
            .download_location
            .label()
            .map(|loc| loc.to_string())
    }

    fn set_download_location(&self, location: &str) {
        self.imp().download_location.set_label(location);
    }

    pub fn general(&self) -> Option<General> {
        Some(General {
            homepage: self.homepage(),
            new_page: match self.new_page() {
                Some(np) => np,
                None => return None,
            },
            show_tabs: match self.show_tabs() {
                Some(st) => st,
                None => return None,
            },
            tab_position: match self.tab_position() {
                Some(tp) => tp,
                None => return None,
            },
            download_scheme: match self.download_scheme() {
                Some(sc) => sc,
                None => return None,
            },
            download_location: self.download_location(),
        })
    }

    pub fn set_general(&self, gen: &General) {
        self.set_homepage(&gen.homepage);
        self.set_new_page(&gen.new_page);
        self.set_show_tabs(&gen.show_tabs);
        self.set_tab_position(&gen.tab_position);
        self.set_download_scheme(&gen.download_scheme);
        self.set_download_location(match &gen.download_location {
            Some(loc) => loc,
            None => "~/Downloads",
        });
    }

    pub fn fg_color(&self) -> RGBA<u8> {
        rgba_from_gdk(self.imp().fg_color.rgba())
    }

    pub fn set_fg_color(&self, color: RGBA<u8>) {
        self.imp().fg_color.set_rgba(&rgba_to_gdk(color));
    }

    pub fn bg_color(&self) -> RGBA<u8> {
        rgba_from_gdk(self.imp().bg_color.rgba())
    }

    pub fn set_bg_color(&self, color: RGBA<u8>) {
        self.imp().bg_color.set_rgba(&rgba_to_gdk(color));
    }

    pub fn pre_fg_color(&self) -> RGBA<u8> {
        rgba_from_gdk(self.imp().pre_fg_color.rgba())
    }

    pub fn set_pre_fg_color(&self, color: RGBA<u8>) {
        self.imp().pre_fg_color.set_rgba(&rgba_to_gdk(color));
    }

    pub fn pre_bg_color(&self) -> RGBA<u8> {
        rgba_from_gdk(self.imp().pre_bg_color.rgba())
    }

    pub fn set_pre_bg_color(&self, color: RGBA<u8>) {
        self.imp().pre_bg_color.set_rgba(&rgba_to_gdk(color));
    }

    pub fn quote_fg_color(&self) -> RGBA<u8> {
        rgba_from_gdk(self.imp().quote_fg_color.rgba())
    }

    pub fn set_quote_fg_color(&self, color: RGBA<u8>) {
        self.imp().quote_fg_color.set_rgba(&rgba_to_gdk(color));
    }

    pub fn quote_bg_color(&self) -> RGBA<u8> {
        rgba_from_gdk(self.imp().quote_bg_color.rgba())
    }

    pub fn set_quote_bg_color(&self, color: RGBA<u8>) {
        self.imp().quote_bg_color.set_rgba(&rgba_to_gdk(color));
    }

    pub fn link_color(&self) -> RGBA<u8> {
        rgba_from_gdk(self.imp().link_color.rgba())
    }

    pub fn set_link_color(&self, color: RGBA<u8>) {
        self.imp().link_color.set_rgba(&rgba_to_gdk(color));
    }

    pub fn hover_color(&self) -> RGBA<u8> {
        rgba_from_gdk(self.imp().hover_color.rgba())
    }

    pub fn set_hover_color(&self, color: RGBA<u8>) {
        self.imp().hover_color.set_rgba(&rgba_to_gdk(color));
    }

    pub fn colors(&self) -> Colors {
        Colors {
            fg: self.fg_color(),
            bg: self.bg_color(),
            pre_fg: self.pre_fg_color(),
            pre_bg: self.pre_bg_color(),
            quote_fg: self.quote_fg_color(),
            quote_bg: self.quote_bg_color(),
            link: self.link_color(),
            hover: self.hover_color(),
        }
    }

    pub fn set_colors(&self, colors: &Colors) {
        self.set_fg_color(colors.fg);
        self.set_bg_color(colors.bg);
        self.set_pre_fg_color(colors.pre_fg);
        self.set_pre_bg_color(colors.pre_bg);
        self.set_quote_fg_color(colors.quote_fg);
        self.set_quote_bg_color(colors.quote_bg);
        self.set_link_color(colors.link);
        self.set_hover_color(colors.hover);
    }

    pub fn pg_font(&self) -> Option<Font> {
        self.imp().pg_font.font_desc().map(Font::from_pango)
    }

    pub fn set_pg_font(&self, font: &Font) {
        self.imp().pg_font.set_font_desc(&font.to_pango());
    }

    pub fn pre_font(&self) -> Option<Font> {
        self.imp().pre_font.font_desc().map(Font::from_pango)
    }

    pub fn set_pre_font(&self, font: &Font) {
        self.imp().pre_font.set_font_desc(&font.to_pango());
    }

    pub fn quote_font(&self) -> Option<Font> {
        self.imp().quote_font.font_desc().map(Font::from_pango)
    }

    pub fn set_quote_font(&self, font: &Font) {
        self.imp().quote_font.set_font_desc(&font.to_pango());
    }

    pub fn h1_font(&self) -> Option<Font> {
        self.imp().h1_font.font_desc().map(Font::from_pango)
    }

    pub fn set_h1_font(&self, font: &Font) {
        self.imp().h1_font.set_font_desc(&font.to_pango());
    }

    pub fn h2_font(&self) -> Option<Font> {
        self.imp().h2_font.font_desc().map(Font::from_pango)
    }

    pub fn set_h2_font(&self, font: &Font) {
        self.imp().h2_font.set_font_desc(&font.to_pango());
    }

    pub fn h3_font(&self) -> Option<Font> {
        self.imp().h3_font.font_desc().map(Font::from_pango)
    }

    pub fn set_h3_font(&self, font: &Font) {
        self.imp().h3_font.set_font_desc(&font.to_pango());
    }

    pub fn fonts(&self) -> Option<Fonts> {
        Some(Fonts {
            pg: match self.pg_font() {
                Some(f) => f,
                None => return None,
            },
            pre: match self.pre_font() {
                Some(f) => f,
                None => return None,
            },
            quote: match self.quote_font() {
                Some(f) => f,
                None => return None,
            },
            h1: match self.h1_font() {
                Some(f) => f,
                None => return None,
            },
            h2: match self.h2_font() {
                Some(f) => f,
                None => return None,
            },
            h3: match self.h3_font() {
                Some(f) => f,
                None => return None,
            },
        })
    }

    pub fn set_fonts(&self, fonts: &Fonts) {
        self.set_pg_font(&fonts.pg);
        self.set_pre_font(&fonts.pre);
        self.set_quote_font(&fonts.quote);
        self.set_h1_font(&fonts.h1);
        self.set_h2_font(&fonts.h2);
        self.set_h3_font(&fonts.h3);
    }

    pub fn config(&self) -> Option<Config> {
        Some(Config {
            general: match self.general() {
                Some(g) => g,
                None => return None,
            },
            colors: self.colors(),
            fonts: match self.fonts() {
                Some(f) => f,
                None => return None,
            },
        })
    }

    pub fn load_config(&self) {
        let cfg = CONFIG.lock().unwrap();
        self.set_general(&cfg.general);
        self.set_colors(&cfg.colors);
        self.set_fonts(&cfg.fonts);
    }

    fn init_dl_location(&self) -> gtk::FileChooserDialog {
        gtk::FileChooserDialog::builder()
            .use_header_bar(1)
            .modal(true)
            .title("Set download location")
            .transient_for(self)
            .action(gtk::FileChooserAction::SelectFolder)
            .create_folders(true)
            .build()
    }
}
