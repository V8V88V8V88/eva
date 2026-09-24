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
    std::{
        borrow::Cow,
        cell::RefCell,
        fs,
        path::PathBuf,
        rc::{Rc, Weak},
    },
    tab::Tab,
    url::Url,
};

thread_local! {
    /// Every open window. Used to find the window a tab currently belongs to,
    /// since tabs can be dragged from one window to another.
    static GUIS: RefCell<Vec<Weak<Gui>>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone)]
pub struct Gui {
    window: gtk::ApplicationWindow,
    overview: adw::TabOverview,
    tab_box: gtk::Box,
    tab_bar: adw::TabBar,
    tab_view: adw::TabView,
    dialogs: Dialogs,
}

impl Default for Gui {
    fn default() -> Self {
        let builder = gtk::Builder::from_string(include_str!("main.ui"));
        let window: gtk::ApplicationWindow = builder.object("mainWindow").unwrap();
        let overview: adw::TabOverview = builder.object("tabOverview").unwrap();
        let tab_box: gtk::Box = builder.object("tabBox").unwrap();
        let tab_bar: adw::TabBar = builder.object("tabBar").unwrap();
        let tab_view: adw::TabView = builder.object("tabView").unwrap();
        let dialogs: Dialogs = Dialogs::init(&window);

        Self {
            window,
            overview,
            tab_box,
            tab_bar,
            tab_view,
            dialogs,
        }
    }
}

impl Gui {
    /// Returns the window which currently contains `widget`
    fn for_widget(widget: &impl IsA<gtk::Widget>) -> Option<Rc<Self>> {
        let root = widget.root()?;
        GUIS.with_borrow(|guis| {
            guis.iter()
                .filter_map(Weak::upgrade)
                .find(|gui| gui.window.upcast_ref::<gtk::Root>() == &root)
        })
    }

    fn new_tab(&self, uri: Option<&str>) -> adw::TabPage {
        let newtab = tab::Tab::init();
        newtab.register();
        let page = self.tab_view.append(&newtab.tab());
        newtab.set_page(&page);
        newtab.set_label("New Tab", false);
        let cfg = CONFIG.lock().unwrap().clone();
        let uri = if cfg.general.new_page == config::NewPage::Home && uri.is_none() {
            Some(cfg.general.homepage.as_str())
        } else {
            uri
        };
        if let Some(uri) = uri {
            if let Ok(u) = Url::parse(uri) {
                let host = u.host_str().unwrap_or("Unknown host");
                newtab.set_label(host, false);
            }
            newtab.controls.set_uri(uri);
            newtab.controls.set_reload_button_sensitive(true);
            newtab.viewer.visit(uri);
        }
        newtab.connect_signals();
        newtab.viewer.connect_page_load_started(clone!(
            #[strong(rename_to = tab)]
            newtab,
            move |_, uri| {
                tab.set_window_title("[loading]");
                tab.controls.set_uri(&uri);
                tab.set_label("[loading]", true);
                tab.controls.set_reload_button_sensitive(false);
            }
        ));
        newtab.viewer.connect_page_loaded(clone!(
            #[strong(rename_to = tab)]
            newtab,
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
                    tab.set_window_title(host);
                    tab.set_label(host, false);
                }
            }
        ));
        newtab.viewer.connect_page_load_failed(clone!(
            #[strong(rename_to = tab)]
            newtab,
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
                            tab.set_window_title(host);
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
                tab.set_window_title("page load failed");
            }
        ));
        newtab.viewer.connect_request_new_tab(|viewer, uri| {
            if let Some(gui) = Self::for_widget(viewer) {
                gui.new_tab(Some(&uri));
            }
        });
        if let Some(app) = self.window.application() {
            newtab.viewer.connect_request_new_window(move |_, uri| {
                let gui = build_ui(&app);
                gui.new_tab(Some(&uri));
            });
        }
        newtab.viewer.connect_request_input(clone!(
            #[strong(rename_to = tab)]
            newtab,
            move |_viewer, meta, url| {
                if let Ok(url) = Url::parse(&url) {
                    if let Some(host) = url.host_str() {
                        tab.set_label(host, false);
                        tab.set_window_title(host);
                    }
                }
                tab.controls.set_uri(&url);
                tab.request_input(&meta, url, true);
            }
        ));
        newtab.viewer.connect_request_input_sensitive(clone!(
            #[strong(rename_to = tab)]
            newtab,
            move |_viewer, meta, url| {
                if let Ok(url) = Url::parse(&url) {
                    if let Some(host) = url.host_str() {
                        tab.set_label(host, false);
                        tab.set_window_title(host);
                    }
                }
                tab.controls.set_uri(&url);
                tab.request_input(&meta, url, false);
            }
        ));
        newtab
            .viewer
            .connect_request_download(|viewer, mime, filename| {
                if let Some(gui) = Self::for_widget(viewer) {
                    gui.download(viewer, &mime, &filename);
                }
            });
        page
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

    fn current_tab(&self) -> Option<Tab> {
        self.tab_view
            .selected_page()
            .as_ref()
            .and_then(Tab::for_page)
    }

    fn select_tab(&self, num: i32) {
        if num < self.tab_view.n_pages() {
            self.tab_view
                .set_selected_page(&self.tab_view.nth_page(num));
        }
    }

    fn next_tab(&self) {
        if let Some(page) = self.tab_view.selected_page() {
            let pos = self.tab_view.page_position(&page);
            self.select_tab((pos + 1) % self.tab_view.n_pages());
        }
    }

    fn prev_tab(&self) {
        if let Some(page) = self.tab_view.selected_page() {
            let pos = self.tab_view.page_position(&page);
            let pages = self.tab_view.n_pages();
            self.select_tab((pos + pages - 1) % pages);
        }
    }

    fn close_current_tab(&self) {
        if let Some(page) = self.tab_view.selected_page() {
            self.tab_view.close_page(&page);
        }
    }

    fn open_tab_overview(&self) {
        self.overview.set_open(true);
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

    fn switch_tab(&self) {
        if let Some(tab) = self.current_tab() {
            let uri = tab.viewer.uri();
            if let Ok(url) = Url::parse(uri.as_str()) {
                tab.set_window_title(url.host_str().unwrap_or("Unknown host"));
            }
        }
    }

    fn set_show_tabs(&self, show: &config::ShowTabs) {
        self.tab_bar.set_visible(*show != config::ShowTabs::Never);
        self.tab_bar
            .set_autohide(*show == config::ShowTabs::Multiple);
    }

    /// `AdwTabBar` can only be laid out horizontally, so the left and right
    /// positions fall back to the top.
    fn set_tab_position(&self, pos: &config::TabPosition) {
        let sibling = match pos {
            config::TabPosition::Bottom => Some(self.tab_view.upcast_ref::<gtk::Widget>()),
            _ => None,
        };
        self.tab_box.reorder_child_after(&self.tab_bar, sibling);
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

    application.connect_startup(|_| {
        if let Err(e) = adw::init() {
            eprintln!("Failed to initialize libadwaita: {e}");
        }
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
    GUIS.with_borrow_mut(|guis| {
        guis.retain(|gui| gui.strong_count() > 0);
        guis.push(Rc::downgrade(&gui));
    });
    gui.tab_view.connect_close_page(|_, page| {
        Tab::unregister(&page.child().widget_name());
        glib::Propagation::Proceed
    });
    gui.tab_view.connect_page_detached(clone!(
        #[weak]
        gui,
        move |view, _, _| {
            if view.n_pages() == 0 {
                gui.window.close();
            }
        }
    ));
    gui.tab_view.connect_selected_page_notify(clone!(
        #[weak]
        gui,
        move |_| {
            gui.switch_tab();
        }
    ));
    gui.overview.connect_create_tab(clone!(
        #[weak]
        gui,
        #[upgrade_or_panic]
        move |_| gui.new_tab(None)
    ));
    gui.window.connect_close_request(clone!(
        #[weak]
        gui,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_| {
            for i in 0..gui.tab_view.n_pages() {
                Tab::unregister(&gui.tab_view.nth_page(i).child().widget_name());
            }
            glib::Propagation::Proceed
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
                    for tab in Tab::all() {
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
