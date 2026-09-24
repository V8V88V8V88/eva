pub mod bookmark_editor;
pub mod controls;
pub mod input;
pub use {bookmark_editor::BookmarkEditor, controls::Controls, input::Input};

use {
    super::uri,
    crate::{BOOKMARKS, CONFIG},
    gemview::GemView,
    gtk::{glib::clone, prelude::*},
    std::{
        cell::{OnceCell, RefCell},
        collections::HashMap,
        fs::File,
        io::{BufReader, Read},
        rc::Rc,
    },
    url::Url,
};

thread_local! {
    /// Every open tab, across all windows, keyed by the widget name of its
    /// content box. Tabs can be dragged between windows, so a per-window
    /// registry would lose track of them.
    static TABS: RefCell<HashMap<String, Tab>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Debug)]
pub struct Tab {
    tab: gtk::Box,
    page: Rc<OnceCell<adw::TabPage>>,
    pub bookmark_editor: BookmarkEditor,
    pub upload: gtk::FileChooserDialog,
    input: Input,
    pub controls: Controls,
    pub viewer: GemView,
}

impl Default for Tab {
    fn default() -> Self {
        let name: String = std::iter::repeat_with(fastrand::alphanumeric)
            .take(10)
            .collect();
        let tab = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .name(&name)
            .build();
        let input = Input::default();
        let bookmark_editor = BookmarkEditor::default();
        let controls = Controls::default();
        controls.set_bookmark_popover(Some(&bookmark_editor));
        let upload = gtk::FileChooserDialog::builder()
            .use_header_bar(1)
            .destroy_with_parent(true)
            .modal(true)
            .title("Choose a file to upload")
            .action(gtk::FileChooserAction::Open)
            .create_folders(true)
            .build();
        upload.add_button("Accept", gtk::ResponseType::Accept);
        upload.add_button("Cancel", gtk::ResponseType::Cancel);
        let scroller = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .propagate_natural_width(true)
            .css_classes(vec!["gemview".to_string()])
            .build();
        let viewer = GemView::new();
        viewer.set_margin_start(25);
        viewer.set_margin_end(25);
        viewer.set_margin_top(25);
        viewer.set_margin_bottom(25);
        viewer.set_css_classes(&["gemview"]);
        scroller.set_child(Some(&viewer));
        tab.append(&input);
        tab.append(&scroller);

        Self {
            tab,
            page: Rc::new(OnceCell::new()),
            input,
            upload,
            bookmark_editor,
            controls,
            viewer,
        }
    }
}

impl Tab {
    pub fn init() -> Self {
        let tab = Self::default();
        tab.set_fonts();
        tab.update_bookmark_editor();
        tab
    }

    pub fn connect_signals(&self) {
        self.controls.addr_bar().connect_activate(clone!(
            #[strong(rename_to = tab)]
            self,
            move |bar| {
                let mut uri = String::from(bar.text());
                uri = uri::uri(&mut uri);
                tab.viewer.visit(&uri);
            }
        ));
        self.viewer.connect_page_load_redirect(clone!(
            #[strong(rename_to = tab)]
            self,
            move |_, uri| {
                tab.controls.set_uri(&uri);
            }
        ));
        self.viewer.connect_request_unsupported_scheme(clone!(
            #[strong(rename_to = tab)]
            self,
            move |viewer, uri| {
                if let Some((scheme, _)) = uri.split_once(':') {
                    match scheme {
                        "eva" => tab.request_eva_page(&uri),
                        // Hand anything else to the desktop's default handler
                        // for that scheme, through the OpenURI portal
                        _ => {
                            let window = viewer.root().and_downcast::<gtk::Window>();
                            gtk::UriLauncher::new(&uri).launch(
                                window.as_ref(),
                                gtk::gio::Cancellable::NONE,
                                move |res| {
                                    if let Err(e) = res {
                                        eprintln!("Error opening {uri}: {e}");
                                    }
                                },
                            );
                        }
                    }
                }
            }
        ));
        let upload = self.upload.clone();
        self.viewer.connect_request_upload(move |viewer, _url| {
            if let Some(window) = viewer.root().and_downcast::<gtk::Window>() {
                upload.set_transient_for(Some(&window));
            }
            upload.show();
        });
        self.upload.connect_response(clone!(
            #[strong(rename_to = viewer)]
            self.viewer,
            move |dlg, response| {
                if response == gtk::ResponseType::Accept {
                    if let Some(file) = dlg.file() {
                        if let Some(path) = file.path() {
                            if let Ok(f) = File::open(path) {
                                let mut data: Vec<u8> = vec![];
                                let mut reader = BufReader::new(f);
                                if reader.read_to_end(&mut data).is_ok() {
                                    if let Ok(url) = Url::parse(&viewer.uri()) {
                                        viewer.post_spartan(url, data);
                                    }
                                }
                            }
                        }
                    }
                }
                dlg.hide();
            }
        ));
    }

    pub fn request_input(&self, meta: &str, url: String, visibility: bool) {
        let viewer = self.viewer.clone();
        let input = self.input.clone();
        self.input.request(meta, visibility, move |response| {
            let mut url = url.clone();
            url.push('?');
            url.push_str(&urlencoding::encode(response));
            viewer.visit(&url);
            input.dismiss();
        });
    }

    pub fn tab(&self) -> gtk::Box {
        self.tab.clone()
    }

    pub fn name(&self) -> String {
        self.tab.widget_name().to_string()
    }

    /// Adds this tab to the global registry
    pub fn register(&self) {
        TABS.with_borrow_mut(|tabs| tabs.insert(self.name(), self.clone()));
    }

    /// Removes the tab whose content box has the given widget name
    pub fn unregister(name: &str) {
        TABS.with_borrow_mut(|tabs| tabs.remove(name));
    }

    /// Looks up the tab displayed in the given page
    pub fn for_page(page: &adw::TabPage) -> Option<Self> {
        let name = page.child().widget_name();
        TABS.with_borrow(|tabs| tabs.get(name.as_str()).cloned())
    }

    /// Returns every open tab, across all windows
    pub fn all() -> Vec<Self> {
        TABS.with_borrow(|tabs| tabs.values().cloned().collect())
    }

    /// Associates this tab with the page it is displayed in. Must be called
    /// once, right after the tab is added to a `TabView`.
    pub fn set_page(&self, page: &adw::TabPage) {
        _ = self.page.set(page.clone());
    }

    /// Sets the title of the window containing this tab, if it is the
    /// selected tab of that window
    pub fn set_window_title(&self, suffix: &str) {
        if !self.page.get().is_some_and(adw::TabPage::is_selected) {
            return;
        }
        if let Some(window) = self.tab.root().and_downcast::<gtk::Window>() {
            window.set_title(Some(&format!(
                "{}-{} - {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                suffix,
            )));
        }
    }

    pub fn set_fonts(&self) {
        let cfg = CONFIG.lock().unwrap().clone();
        self.viewer
            .set_font_paragraph(cfg.fonts.pg.to_pango().to_string());
        self.viewer
            .set_font_quote(cfg.fonts.quote.to_pango().to_string());
        self.viewer
            .set_font_pre(cfg.fonts.pre.to_pango().to_string());
        self.viewer.set_font_h1(cfg.fonts.h1.to_pango().to_string());
        self.viewer.set_font_h2(cfg.fonts.h2.to_pango().to_string());
        self.viewer.set_font_h3(cfg.fonts.h3.to_pango().to_string());
    }

    pub fn update_bookmark_editor(&self) {
        if self.bookmark_editor.update(self.viewer.uri().as_str()) {
            self.controls
                .set_bookmark_icon_name("user-bookmarks-symbolic");
        } else {
            self.controls
                .set_bookmark_icon_name("bookmark-new-symbolic");
        }
    }

    pub fn set_label(&self, label: &str, spin: bool) {
        if let Some(page) = self.page.get() {
            page.set_title(label);
            page.set_loading(spin);
        }
    }

    pub fn request_eva_page(&self, uri: &str) {
        if let Ok(url) = Url::parse(uri) {
            match url.host_str() {
                Some("bookmarks") => match url.path() {
                    "" | "/" => self.open_bookmarks(),
                    "/tags" | "/tags/" => self.open_bookmark_tags(),
                    p => {
                        let maybe_tag = p.replace("/tags/", "");
                        let bookmarks = BOOKMARKS.lock().unwrap();
                        if let Some(page) = bookmarks.tag_to_gmi(&maybe_tag) {
                            self.viewer.render_gmi(&page);
                            self.viewer.set_uri(uri);
                            self.controls.set_uri("uri");
                            self.set_label("bookmarks", false);
                        }
                    }
                },
                //Some("history") => {}
                Some("source") => {
                    self.view_source();
                }
                _ => {}
            }
        }
    }

    pub fn open_bookmarks(&self) {
        let bookmarks = BOOKMARKS.lock().unwrap();
        let page = bookmarks.to_gmi();
        self.viewer.render_gmi(&page);
        self.viewer.set_uri("eva://bookmarks");
        self.controls.set_uri("eva://bookmarks");
        self.controls
            .set_bookmark_icon_name("bookmark-new-symbolic");
        self.set_label("bookmarks", false);
    }

    fn open_bookmark_tags(&self) {
        let bookmarks = BOOKMARKS.lock().unwrap();
        let page = bookmarks.tags_to_gmi();
        self.viewer.render_gmi(&page);
        self.viewer.set_uri("eva://bookmarks/tags");
        self.controls.set_uri("eva://bookmarks/tags");
        self.controls
            .set_bookmark_icon_name("bookmark-new-symbolic");
        self.set_label("bookmarks", false);
    }

    pub fn view_source(&self) {
        let mime = self.viewer.buffer_mime();
        let content = self.viewer.buffer_content();
        if mime.starts_with("text") {
            let content = String::from_utf8_lossy(&content);
            self.viewer.render_text(&content);
            self.controls.set_uri("eva://source");
        }
    }
}
