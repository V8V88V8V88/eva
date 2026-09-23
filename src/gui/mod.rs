#![allow(clippy::too_many_lines)]
mod actions;
mod dialogs;
pub mod tab;
pub mod uri;
use {
    crate::{config, CONFIG},
    dialogs::Dialogs,
    gemview::GemView,
    gtk::{
        gdk::Display,
        gio::{Cancellable, Notification},
        glib,
        glib::{char::Char, clone, OptionArg, OptionFlags},
        prelude::*,
        Application, CssProvider, ResponseType,
    },
    mime2ext::mime2ext,
    std::{borrow::Cow, cell::RefCell, collections::HashMap, fs, path::PathBuf, rc::Rc},
    tab::Tab,
    url::Url,
};

#[derive(Clone)]
pub struct Gui {
    window: gtk::ApplicationWindow,
    notebook: gtk::Notebook,
    tabs: RefCell<HashMap<String, Tab>>,
    dialogs: Dialogs,
}

impl Default for Gui {
    fn default() -> Self {
        let builder = gtk::Builder::from_string(include_str!("main.ui"));
        let window: gtk::ApplicationWindow = builder.object("mainWindow").unwrap();
        let notebook: gtk::Notebook = builder.object("mainNotebook").unwrap();
        let tabs: RefCell<HashMap<String, Tab>> = RefCell::new(HashMap::new());
        let dialogs: Dialogs = Dialogs::init(&window);

        Self {
            window,
            notebook,
            tabs,
            dialogs,
        }
    }
}

impl Gui {
    fn new_tab(&self, uri: Option<&str>) {
        let newtab = tab::Tab::init();
        self.tabs
            .borrow_mut()
            .insert(newtab.tab().widget_name().to_string(), newtab.clone());
        let cfg = CONFIG.lock().unwrap().clone();
        let uri = if cfg.general.new_page == config::NewPage::Home && uri.is_none() {
            Some(cfg.general.homepage.as_str())
        } else {
            uri
        };
        if let Some(uri) = uri {
            if let Ok(u) = Url::parse(uri) {
                let host = u.host_str().unwrap_or("Unknown host");
                newtab.label.set(host, false);
            }
            newtab.controls.set_uri(uri);
            newtab.controls.set_reload_button_sensitive(true);
            newtab.viewer.visit(uri);
        }
        self.notebook
            .append_page(&newtab.tab(), Some(&newtab.label));
        self.notebook.set_tab_reorderable(&newtab.tab(), true);
        newtab.connect_signals();
        newtab.upload.set_transient_for(Some(&self.window));
        newtab.label.close_button().connect_clicked(clone!(
            #[strong(rename_to = tab)]
            newtab,
            #[weak(rename_to = nb)]
            self.notebook,
            move |_| {
                let _name = tab.tab().widget_name().to_string();
                nb.detach_tab(&tab.tab());
            }
        ));
        newtab.viewer.connect_page_load_started(clone!(
            #[weak(rename_to = window)]
            self.window,
            #[strong(rename_to = tab)]
            newtab,
            move |_, uri| {
                window.set_title(Some(&format!(
                    "{}-{} - [loading]",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                )));
                tab.controls.set_uri(&uri);
                tab.set_label("[loading]", true);
                tab.controls.set_reload_button_sensitive(false);
            }
        ));
        newtab.viewer.connect_page_loaded(clone!(
            #[strong(rename_to = tab)]
            newtab,
            #[weak(rename_to = window)]
            self.window,
            move |_, uri| {
                tab.controls.set_uri(&uri);
                tab.controls.set_reload_button_sensitive(true);
                tab.controls
                    .set_back_button_sensitive(tab.viewer.has_previous());
                tab.controls
                    .set_forward_button_sensitive(tab.viewer.has_next());
                tab.update_bookmark_editor();
                if let Ok(url) = Url::parse(uri.as_str()) {
                    let scheme = url.scheme();
                    let host = url.host_str().unwrap_or_else(|| {
                        if scheme == "file" {
                            "filesystem"
                        } else {
                            "Unknown host"
                        }
                    });
                    window.set_title(Some(&format!(
                        "{}-{} - {}",
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION"),
                        host,
                    )));
                    tab.set_label(host, false);
                }
            }
        ));
        newtab.viewer.connect_page_load_failed(clone!(
            #[strong(rename_to = tab)]
            newtab,
            #[weak(rename_to = window)]
            self.window,
            move |_, err| {
                tab.controls.set_reload_button_sensitive(true);
                tab.controls
                    .set_back_button_sensitive(tab.viewer.has_previous());
                tab.controls
                    .set_forward_button_sensitive(tab.viewer.has_next());
                if err.contains("unsupported-scheme") {
                    if let Ok(url) = Url::parse(tab.viewer.uri().as_str()) {
                        if let Some(host) = url.host_str() {
                            tab.set_label(host, false);
                            window.set_title(Some(&format!(
                                "{}-{} -{}",
                                env!("CARGO_PKG_NAME"),
                                env!("CARGO_PKG_VERSION"),
                                host,
                            )));
                        }
                    }
                    tab.controls.set_uri(tab.viewer.uri().as_str());
                    return;
                }
                tab.set_label("Load failure", false);
                tab.viewer.render_gmi(&format!(
                    "# Page load failure\n\n{}",
                    match err.as_str() {
                        "RelativeUrlWithCannotBeABaseBase" => "Invalid url",
                        s if s.contains(
                            "failed to lookup address information: Name or service not known"
                        ) =>
                        {
                            "Cannot resolve dns for host"
                        }
                        s => s,
                    },
                ));
                window.set_title(Some(&format!(
                    "{}-{} - page load failed",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                )));
            }
        ));
        newtab.viewer.connect_request_new_tab(clone!(
            #[strong(rename_to = gui)]
            self,
            move |_, uri| {
                gui.new_tab(Some(&uri));
            }
        ));
        if let Some(app) = self.window.application() {
            newtab.viewer.connect_request_new_window(move |_, uri| {
                let gui = build_ui(&app);
                gui.new_tab(Some(&uri));
            });
        }
        newtab.viewer.connect_request_input(clone!(
            #[strong(rename_to = tab)]
            newtab,
            #[weak(rename_to = window)]
            self.window,
            move |_viewer, meta, url| {
                if let Ok(url) = Url::parse(&url) {
                    if let Some(host) = url.host_str() {
                        tab.set_label(host, false);
                        window.set_title(Some(&format!(
                            "{}-{} - {}",
                            env!("CARGO_PKG_NAME"),
                            env!("CARGO_PKG_VERSION"),
                            host,
                        )));
                    }
                }
                tab.controls.set_uri(&url);
                tab.request_input(&meta, url, true);
            }
        ));
        newtab.viewer.connect_request_input_sensitive(clone!(
            #[strong(rename_to = tab)]
            newtab,
            #[weak(rename_to = window)]
            self.window,
            move |_viewer, meta, url| {
                if let Ok(url) = Url::parse(&url) {
                    if let Some(host) = url.host_str() {
                        tab.set_label(host, false);
                        window.set_title(Some(&format!(
                            "{}-{} - {}",
                            env!("CARGO_PKG_NAME"),
                            env!("CARGO_PKG_VERSION"),
                            host,
                        )));
                    }
                }
                tab.controls.set_uri(&url);
                tab.request_input(&meta, url, false);
            }
        ));
        newtab.viewer.connect_request_download(clone!(
            #[strong(rename_to = gui)]
            self,
            move |viewer, mime, filename| {
                gui.download(viewer, &mime, &filename);
            }
        ));
    }

    fn download(&self, viewer: &GemView, mime: &str, filename: &str) {
        let cfg = CONFIG.lock().unwrap();
        let filename = if filename == "download" {
            if let Some(extension) = mime2ext(mime) {
                Cow::from(format!("{}.{}", filename, extension))
            } else {
                Cow::from(filename)
            }
        } else {
            Cow::from(filename)
        };
        let scheme = &cfg.general.download_scheme;
        match scheme {
            config::DownloadScheme::Ask => {
                self.dialogs.save.set_current_name(&filename);
                self.dialogs.save.connect_response(clone!(
                    #[weak]
                    viewer,
                    #[strong(rename_to = gui)]
                    self,
                    move |dlg, response| {
                        match response {
                            gtk::ResponseType::Accept => {
                                if let Some(file) = dlg.file() {
                                    if let Some(path) = file.path() {
                                        match fs::write(&path, &viewer.buffer_content()) {
                                            Ok(_) => gui.send_notification(&format!(
                                                "File saved: {}",
                                                path.display(),
                                            )),
                                            Err(e) => {
                                                gui.send_notification(&format!("Error: {}", e,))
                                            }
                                        }
                                    }
                                }
                                dlg.hide();
                            }
                            _ => dlg.hide(),
                        }
                    }
                ));
                self.dialogs.save.show();
                viewer.reload();
            }
            config::DownloadScheme::Auto => {
                if let Some(location) = &cfg.general.download_location {
                    let mut location = PathBuf::from(location);
                    if !location.exists() {
                        if let Err(e) = fs::create_dir_all(&location) {
                            self.send_notification(&format!("Error: {}", e,));
                            viewer.reload();
                            return;
                        }
                    }
                    location.push(&*filename);
                    match fs::write(&location, &viewer.buffer_content()) {
                        Ok(_) => {
                            self.send_notification(&format!("File saved: {}", location.display()));
                        }
                        Err(e) => self.send_notification(&format!("Error: {}", e,)),
                    }
                    viewer.reload();
                }
            }
        }
    }

    fn send_notification(&self, message: &str) {
        if let Some(application) = self.window.application() {
            let notification = Notification::new(env!("CARGO_PKG_NAME"));
            notification.set_body(Some(message));
            application.send_notification(None, &notification);
        }
    }

    fn current_page(&self) -> Option<u32> {
        self.notebook.current_page()
    }

    fn current_tab(&self) -> Option<Tab> {
        if let Some(t) = self.notebook.nth_page(self.current_page()) {
            self.tabs
                .borrow()
                .get(&t.widget_name().to_string())
                .cloned()
        } else {
            None
        }
    }

    fn nth_tab(&self, num: u32) -> Option<Tab> {
        if let Some(t) = self.notebook.nth_page(Some(num)) {
            self.tabs
                .borrow()
                .get(&t.widget_name().to_string())
                .cloned()
        } else {
            None
        }
    }

    fn next_tab(&self) {
        if let Some(current) = self.notebook.current_page() {
            let pages = self.notebook.n_pages();
            if current == pages - 1 {
                self.notebook.set_current_page(Some(0));
            } else {
                self.notebook.set_current_page(Some(current + 1));
            }
        }
    }

    fn prev_tab(&self) {
        if let Some(current) = self.current_page() {
            let pages = self.notebook.n_pages();
            if current == 0 {
                self.notebook.set_current_page(Some(pages - 1));
            } else {
                self.notebook.set_current_page(Some(current - 1));
            }
        }
    }

    fn close_current_tab(&self) {
        if let Some(page) = self.current_page() {
            if let Some(tab) = self.current_tab() {
                let name = tab.tab().widget_name().to_string();
                self.tabs.borrow_mut().remove(&name);
            }
            self.notebook.remove_page(Some(page));
        }
    }

    /* fn close_tab_named(&self, name: &str) {
        match self.tabs.borrow().get(name) {
            Some(tab) => self.notebook.detach_tab(&tab.tab()),
            None => {},
        }
        self.tabs.borrow_mut().remove(name);
    } */

    fn cleanup_tabs(&self) {
        let tabs = self.tabs.borrow_mut().clone();
        for (name, tab) in tabs {
            match self.notebook.page_num(&tab.tab()) {
                Some(_) => {}
                None => {
                    let _rem = self.tabs.borrow_mut().remove(&name);
                }
            }
        }
    }

    fn reload_current_tab(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tab) = self.current_tab() {
            tab.viewer.reload();
            Ok(())
        } else {
            Err(String::from("Error getting tab").into())
        }
    }

    fn go_home(&self) -> Result<(), Box<dyn std::error::Error>> {
        let home = CONFIG.lock().unwrap().clone().general.homepage;
        if let Some(tab) = self.current_tab() {
            tab.viewer.visit(&home);
            Ok(())
        } else {
            Err(String::from("Error getting tab").into())
        }
    }

    fn go_previous(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tab) = self.current_tab() {
            tab.viewer.go_previous();
            Ok(())
        } else {
            Err(String::from("Error getting tab").into())
        }
    }

    fn go_next(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tab) = self.current_tab() {
            tab.viewer.go_next();
            Ok(())
        } else {
            Err(String::from("Error getting tab").into())
        }
    }

    fn switch_tab(&self, page: u32) {
        if let Some(tab) = self.nth_tab(page) {
            let uri = tab.viewer.uri();
            if let Ok(url) = Url::parse(uri.as_str()) {
                self.window.set_title(Some(&format!(
                    "{}-{} - {}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                    url.host_str().unwrap_or("Unknown host"),
                )));
            }
        }
    }

    fn set_show_tabs(&self, show: &config::ShowTabs) {
        self.notebook.set_show_tabs(match show {
            config::ShowTabs::Always => true,
            config::ShowTabs::Never => false,
            config::ShowTabs::Multiple => self.notebook.n_pages() > 1,
        });
    }

    fn set_tab_position(&self, pos: &config::TabPosition) {
        self.notebook.set_tab_pos(pos.to_gtk());
    }

    fn set_general(&self, gen: &config::General) {
        self.set_show_tabs(&gen.show_tabs);
        self.set_tab_position(&gen.tab_position);
    }

    fn set_css(&self, colors: &config::Colors) {
        let provider = CssProvider::new();
        let context = self.window.style_context();
        let css = include_str!("gemview.css")
            .replace("NORMAL_FG_COLOR", &colors.fg.to_string())
            .replace("NORMAL_BG_COLOR", &colors.bg.to_string())
            .replace("QUOTE_FG_COLOR", &colors.quote_fg.to_string())
            .replace("QUOTE_BG_COLOR", &colors.quote_bg.to_string())
            .replace("PRE_FG_COLOR", &colors.pre_fg.to_string())
            .replace("PRE_BG_COLOR", &colors.pre_bg.to_string())
            .replace("LINK_COLOR", &colors.link.to_string())
            .replace("HOVER_COLOR", &colors.hover.to_string())
            .replace("DEFAULT_FG_COLOR", &context.color().to_string())
            .replace("ReducedRGBA", "rgba")
            .replace("RGBA", "rgba");
        provider.load_from_data(&css);
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Cannot connect to display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn open_bookmarks(&self) {
        if let Some(tab) = self.current_tab() {
            tab.open_bookmarks();
            self.window.set_title(Some(&format!(
                "{}-{} - bookmarks",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            )));
        }
    }

    fn save_page(&self) {
        if let Some(tab) = self.current_tab() {
            let viewer = tab.viewer;
            let mut filename = if let Some(s) = viewer.uri().split('/').last() {
                match s {
                    "" => "unknown",
                    _ => s,
                }
            } else {
                "unknown"
            }
            .to_string();
            if !filename.contains('.') {
                let mime = viewer.buffer_mime();
                let ext = if let Some(e) = mime2ext(&mime) {
                    Some(e)
                } else if mime == "text/gemini" {
                    Some("gmi")
                } else {
                    None
                };
                if let Some(ext) = ext {
                    filename.push('.');
                    filename.push_str(ext);
                }
            }
            self.dialogs.save.set_current_name(&filename);
            self.dialogs.save.connect_response(clone!(
                #[weak]
                viewer,
                #[strong(rename_to = gui)]
                self,
                move |dlg, response| {
                    match response {
                        gtk::ResponseType::Accept => {
                            if let Some(file) = dlg.file() {
                                if let Some(path) = file.path() {
                                    match fs::write(&path, &viewer.buffer_content()) {
                                        Ok(_) => gui.send_notification(&format!(
                                            "File saved: {}",
                                            path.display(),
                                        )),
                                        Err(e) => gui.send_notification(&format!("Error: {}", e,)),
                                    }
                                }
                            }
                            dlg.hide();
                        }
                        _ => dlg.hide(),
                    }
                }
            ));
            self.dialogs.save.show();
            viewer.reload();
        }
    }
}

pub fn run() {
    let application = Rc::new(gtk::Application::new(
        Some("org.hitchhiker-linux.eva"),
        gtk::gio::ApplicationFlags::HANDLES_OPEN,
    ));

    application.add_main_option(
        "private",
        Char::from(b'p'),
        OptionFlags::NONE,
        OptionArg::None,
        "Do not save history",
        None,
    );

    application.add_main_option(
        "version",
        Char::from(b'v'),
        OptionFlags::NONE,
        OptionArg::None,
        "Display program version",
        None,
    );

    application.connect_handle_local_options(move |_, dict| {
        if dict.contains("version") {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return std::ops::ControlFlow::Break(gtk::glib::ExitCode::SUCCESS);
        }
        std::ops::ControlFlow::Continue(())
    });

    match application.register(Some(&Cancellable::new())) {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    };

    application.connect_open(move |app, addr, _| {
        let gui = build_ui(app);
        for uri in addr {
            gui.new_tab(Some(&uri.uri()));
        }
    });
    application.connect_activate(|app| {
        let gui = build_ui(app);
        gui.new_tab(None);
    });
    application.run();
}

pub fn build_ui(app: &Application) -> Rc<Gui> {
    let gui = Rc::new(Gui::default());
    actions::add(&gui, app);
    let config = CONFIG.lock().unwrap().clone();
    gui.set_css(&config.colors);
    gui.window.set_application(Some(app));
    gui.notebook.connect_page_removed(clone!(
        #[weak]
        gui,
        #[strong]
        config,
        move |nb, _page, _| {
            gui.cleanup_tabs();
            let multi = config.general.show_tabs == config::ShowTabs::Multiple;
            match nb.n_pages() {
                0 => gui.window.close(),
                1 => {
                    if multi {
                        nb.set_show_tabs(false);
                    }
                }
                _ => {
                    if multi {
                        nb.set_show_tabs(true);
                    }
                }
            }
        }
    ));
    gui.notebook.connect_page_added(clone!(
        #[weak]
        gui,
        #[strong]
        config,
        move |nb, _page, _| {
            if nb.n_pages() > 1 && config.general.show_tabs == config::ShowTabs::Multiple {
                nb.set_show_tabs(true);
            }
        }
    ));
    gui.notebook.connect_switch_page(clone!(
        #[weak]
        gui,
        move |_, _, page| {
            gui.switch_tab(page);
        }
    ));
    gui.dialogs.preferences.connect_response(clone!(
        #[weak]
        gui,
        move |dlg, res| {
            if res == ResponseType::Accept {
                if let Some(cfg) = gui.dialogs.preferences.config() {
                    *CONFIG.lock().unwrap() = cfg.clone();
                    if let Err(e) = cfg.save_to_file(&config::get_config_file()) {
                        eprintln!("{}", e);
                    }
                    gui.set_general(&cfg.general);
                    gui.set_css(&cfg.colors);
                    for (_, tab) in gui.tabs.borrow().clone() {
                        tab.set_fonts();
                    }
                } else {
                    gui.dialogs.preferences.load_config();
                }
            }
            dlg.hide();
        }
    ));
    gui.set_general(&config.general);

    gui.window.show();
    gui
}
