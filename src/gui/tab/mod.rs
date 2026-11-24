pub mod bookmark_editor;
pub mod controls;
pub mod input;
pub use {bookmark_editor::BookmarkEditor, controls::Controls, input::Input};

use {
    super::uri,
    crate::{BOOKMARKS, CONFIG, HISTORY, PRIVATE},
    adw::prelude::{AdwDialogExt, AlertDialogExt, AlertDialogExtManual},
    gemview::GemView,
    gtk::{glib::clone, prelude::*},
    std::{
        cell::{Cell, OnceCell, RefCell},
        collections::HashMap,
        fs::File,
        io::{BufReader, Read},
        rc::Rc,
        sync::atomic::Ordering,
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
    sensitive: Rc<Cell<bool>>,
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
        viewer.set_known_hosts(
            crate::history::get_data_dir()
                .join("known_hosts")
                .to_string_lossy()
                .into_owned(),
        );
        viewer.set_client_certs(
            crate::history::get_data_dir()
                .join("client_certs")
                .to_string_lossy()
                .into_owned(),
        );
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
            sensitive: Rc::new(Cell::new(false)),
            upload,
            bookmark_editor,
            controls,
            viewer,
        }
    }
}

pub fn format_fingerprint(fingerprint: &str) -> String {
    let lines: Vec<String> = fingerprint
        .as_bytes()
        .chunks(32)
        .map(|line| {
            line.chunks(4)
                .map(|g| String::from_utf8_lossy(g).into_owned())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    format!(
        "<tt>{}</tt>",
        gtk::glib::markup_escape_text(&lines.join("\n"))
    )
}

fn insecure_scheme(scheme: &str) -> bool {
    matches!(scheme, "gopher" | "finger" | "spartan" | "mercury")
}

fn scheme_name(scheme: &str) -> String {
    let mut chars = scheme.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn status_row(icon: &str, heading: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.append(&gtk::Image::from_icon_name(icon));
    row.append(
        &gtk::Label::builder()
            .label(heading)
            .css_classes(["heading"])
            .build(),
    );
    row
}

fn info_label(markup: &str, selectable: bool) -> gtk::Label {
    gtk::Label::builder()
        .label(markup)
        .use_markup(true)
        .wrap(true)
        .max_width_chars(40)
        .xalign(0.0)
        .selectable(selectable)
        .css_classes(["dim-label"])
        .build()
}

fn site_info_box(host: &str, icon: &str, heading: &str, text: &str) -> gtk::Box {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    content.append(
        &gtk::Label::builder()
            .label(host)
            .css_classes(["title-4"])
            .xalign(0.0)
            .margin_bottom(4)
            .build(),
    );
    content.append(&status_row(icon, heading));
    content.append(&info_label(&gtk::glib::markup_escape_text(text), false));
    content
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
        self.controls.addr_bar().connect_icon_press(clone!(
            #[strong(rename_to = tab)]
            self,
            move |_, pos| {
                if pos == gtk::EntryIconPosition::Primary {
                    tab.site_info();
                }
            }
        ));
        self.viewer.connect_page_loaded(clone!(
            #[strong(rename_to = tab)]
            self,
            move |_, _| tab.update_site_icon()
        ));
        self.viewer.connect_page_load_started(clone!(
            #[strong(rename_to = tab)]
            self,
            move |_, _| tab.controls.set_site_icon(controls::SiteState::None)
        ));
        self.viewer.connect_request_cross_protocol_redirect(clone!(
            #[strong(rename_to = tab)]
            self,
            move |_, from, to| tab.confirm_cross_protocol_redirect(&from, to)
        ));
        self.viewer.connect_certificate_required(clone!(
            #[strong(rename_to = tab)]
            self,
            move |_, host, meta, code, has_cert, url| {
                tab.certificate_required(&host, &meta, code, has_cert, url);
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

    pub fn update_site_icon(&self) {
        let state = match Url::parse(&self.viewer.uri()) {
            Ok(url) if url.scheme() == "gemini" => {
                if self.viewer.client_certificate_scope(&url).is_some() {
                    controls::SiteState::Identified
                } else {
                    controls::SiteState::Verified
                }
            }
            Ok(url) if insecure_scheme(url.scheme()) => controls::SiteState::Insecure,
            _ => controls::SiteState::None,
        };
        self.controls.set_site_icon(state);
    }

    fn after_certificate_change(viewer: &GemView, url: &Url, result: Result<(), String>) {
        match result {
            Ok(()) => viewer.visit(url.as_str()),
            Err(e) => eprintln!("Error updating client certificate for {url}: {e}"),
        }
    }

    fn certificate_required(&self, host: &str, meta: &str, code: u8, has_cert: bool, url: String) {
        let Ok(url) = Url::parse(&url) else {
            return;
        };
        let meta = meta.trim();
        let reason = if meta.is_empty() {
            String::new()
        } else {
            format!("\n\nThe server says: “{meta}”")
        };
        let viewer = self.viewer.clone();
        let dlg = if has_cert {
            let problem = match code {
                61 => "is not authorised to view this page. The certificate itself may \
                    still be accepted elsewhere on the server",
                62 => "was not accepted as valid",
                _ => "was not accepted",
            };
            let dlg = adw::AlertDialog::new(
                Some("Certificate Rejected"),
                Some(&format!(
                    "The client certificate you use on {host} {problem} (status {code}).\
                    {reason}\n\nReplacing it gives you a new identity on this server."
                )),
            );
            dlg.add_responses(&[
                ("cancel", "Cancel"),
                ("remove", "Remove"),
                ("replace", "Replace"),
            ]);
            dlg.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
            dlg.connect_response(None, clone!(
                #[strong(rename_to = tab)]
                self,
                move |_, response| match response {
                    "remove" => {
                        if let Err(e) = viewer.remove_client_certificate(&url) {
                            eprintln!("Error removing client certificate for {url}: {e}");
                        }
                        tab.update_site_icon();
                    }
                    "replace" => Self::after_certificate_change(
                        &viewer,
                        &url,
                        viewer.replace_client_certificate(&url).map_err(|e| e.to_string()),
                    ),
                    _ => {}
                }
            ));
            dlg
        } else {
            let dlg = adw::AlertDialog::new(
                Some("Client Certificate Required"),
                Some(&format!(
                    "{host} asks for a client certificate to identify you.{reason}\n\n\
                    Eva can create one for this page and the pages below it, or for \
                    the whole site. The server will recognise you by it on later visits."
                )),
            );
            dlg.add_responses(&[
                ("cancel", "Cancel"),
                ("site", "Whole Site"),
                ("page", "This Page"),
            ]);
            dlg.set_response_appearance("page", adw::ResponseAppearance::Suggested);
            dlg.connect_response(None, move |_, response| {
                let whole_site = match response {
                    "page" => false,
                    "site" => true,
                    _ => return,
                };
                Self::after_certificate_change(
                    &viewer,
                    &url,
                    viewer
                        .create_client_certificate(&url, whole_site)
                        .map_err(|e| e.to_string()),
                );
            });
            dlg
        };
        dlg.set_default_response(Some("cancel"));
        dlg.set_close_response("cancel");
        dlg.present(Some(&self.viewer));
    }

    fn site_info(&self) {
        let Ok(url) = Url::parse(&self.viewer.uri()) else {
            return;
        };
        let content = if url.scheme() == "gemini" {
            self.gemini_site_info(&url)
        } else if insecure_scheme(url.scheme()) {
            site_info_box(
                url.host_str().unwrap_or_default(),
                "security-low-symbolic",
                "Not Encrypted",
                &format!(
                    "{} pages are sent as plain text. Anyone on the network can read \
                    what you view and send here.",
                    scheme_name(url.scheme())
                ),
            )
        } else {
            return;
        };
        self.controls.show_site_popover(&content);
    }

    fn gemini_site_info(&self, url: &Url) -> gtk::Box {
        let host = url.host_str().unwrap_or_default();
        let content = match self.viewer.server_certificate(url) {
            Some((fingerprint, expires)) => {
                let content = site_info_box(
                    host,
                    "security-high-symbolic",
                    "Verified Connection",
                    "The server certificate matches the one trusted on your first visit.",
                );
                let expiry = chrono::DateTime::from_timestamp(expires, 0).map_or_else(
                    || String::from("unknown"),
                    |d| d.format("%Y-%m-%d").to_string(),
                );
                content.append(&info_label(
                    &format!("{}\nExpires {expiry}", format_fingerprint(&fingerprint)),
                    true,
                ));
                content
            }
            None => site_info_box(
                host,
                "security-high-symbolic",
                "Encrypted Connection",
                "No server certificate has been recorded for this site yet.",
            ),
        };
        content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let scope = self.viewer.client_certificate_scope(url);
        let (heading, text) = match &scope {
            Some(scope) => (
                "Identified",
                format!(
                    "You use a client certificate on\n<tt>{}</tt>\nand the pages below it.",
                    gtk::glib::markup_escape_text(scope)
                ),
            ),
            None => (
                "Anonymous",
                String::from("You are not using a client certificate on this site."),
            ),
        };
        content.append(&status_row("avatar-default-symbolic", heading));
        content.append(&info_label(&text, true));
        let button = gtk::Button::builder()
            .label(if scope.is_some() {
                "Remove Certificate…"
            } else {
                "Create Certificate for Site"
            })
            .halign(gtk::Align::Start)
            .css_classes(["flat"])
            .build();
        let url = url.clone();
        let has_cert = scope.is_some();
        button.connect_clicked(clone!(
            #[strong(rename_to = tab)]
            self,
            move |_| {
                tab.controls.hide_site_popover();
                if has_cert {
                    tab.confirm_remove_certificate(&url);
                } else {
                    Self::after_certificate_change(
                        &tab.viewer,
                        &url,
                        tab.viewer
                            .create_client_certificate(&url, true)
                            .map_err(|e| e.to_string()),
                    );
                }
            }
        ));
        content.append(&button);
        content
    }

    fn confirm_remove_certificate(&self, url: &Url) {
        let host = url.host_str().unwrap_or_default();
        let dlg = adw::AlertDialog::new(
            Some("Remove Client Certificate?"),
            Some(&format!(
                "{host} will no longer recognise you. This cannot be undone."
            )),
        );
        dlg.add_responses(&[("cancel", "Cancel"), ("remove", "Remove")]);
        dlg.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
        dlg.set_default_response(Some("cancel"));
        dlg.set_close_response("cancel");
        let url = url.clone();
        dlg.connect_response(Some("remove"), clone!(
            #[strong(rename_to = tab)]
            self,
            move |_, _| {
                if let Err(e) = tab.viewer.remove_client_certificate(&url) {
                    eprintln!("Error removing client certificate for {url}: {e}");
                }
                tab.viewer.reload();
            }
        ));
        dlg.present(Some(&self.viewer));
    }

    fn confirm_cross_protocol_redirect(&self, from: &str, to: String) {
        let host = Url::parse(from)
            .ok()
            .and_then(|u| u.host_str().map(str::to_string))
            .unwrap_or_default();
        let scheme = to.split_once(':').map_or("", |(s, _)| s);
        let warning = if insecure_scheme(scheme) || scheme == "http" {
            "\n\nThis connection will not be encrypted."
        } else {
            ""
        };
        let dlg = adw::AlertDialog::new(
            Some("Leave Gemini?"),
            Some(&format!(
                "{host} is redirecting to\n<tt>{}</tt>{warning}",
                gtk::glib::markup_escape_text(&to)
            )),
        );
        dlg.set_body_use_markup(true);
        dlg.add_responses(&[("cancel", "Cancel"), ("open", "Open")]);
        dlg.set_response_appearance("open", adw::ResponseAppearance::Suggested);
        dlg.set_default_response(Some("cancel"));
        dlg.set_close_response("cancel");
        let viewer = self.viewer.clone();
        dlg.connect_response(Some("open"), move |_, _| viewer.visit(&to));
        dlg.present(Some(&self.viewer));
    }

    pub fn request_input(&self, meta: &str, url: String, visibility: bool) {
        let viewer = self.viewer.clone();
        let input = self.input.clone();
        let sensitive = self.sensitive.clone();
        self.input.request(meta, visibility, move |response| {
            sensitive.set(!visibility);
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
                            self.viewer.append_history(uri);
                            self.update_nav_buttons();
                            self.controls.set_uri("uri");
                            self.set_label("bookmarks", false);
                        }
                    }
                },
                Some("history") => self.open_history(),
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
        self.viewer.append_history("eva://bookmarks");
        self.update_nav_buttons();
        self.controls.set_uri("eva://bookmarks");
        self.controls
            .set_bookmark_icon_name("bookmark-new-symbolic");
        self.set_label("bookmarks", false);
    }

    fn update_nav_buttons(&self) {
        if let Some(gui) = super::Gui::for_widget(&self.viewer) {
            gui.update_nav_buttons();
        }
    }

    pub fn record_history(&self, uri: &str) {
        if self.sensitive.replace(false) || PRIVATE.load(Ordering::Relaxed) {
            return;
        }
        let Ok(url) = Url::parse(uri) else {
            return;
        };
        if matches!(url.scheme(), "about" | "data" | "eva") {
            return;
        }
        let mut history = HISTORY.lock().unwrap();
        history.append(uri);
        if let Err(e) = history.save() {
            eprintln!("Error saving history: {e}");
        }
    }

    pub fn clear_sensitive(&self) {
        self.sensitive.set(false);
    }

    pub fn is_showing_history(&self) -> bool {
        self.viewer.uri() == "eva://history"
    }

    pub fn open_history(&self) {
        let page = HISTORY.lock().unwrap().page();
        self.viewer.render_gmi(&page);
        self.viewer.append_history("eva://history");
        self.update_nav_buttons();
        self.controls.set_uri("eva://history");
        self.controls
            .set_bookmark_icon_name("bookmark-new-symbolic");
        self.set_label("history", false);
    }

    fn open_bookmark_tags(&self) {
        let bookmarks = BOOKMARKS.lock().unwrap();
        let page = bookmarks.tags_to_gmi();
        self.viewer.render_gmi(&page);
        self.viewer.append_history("eva://bookmarks/tags");
        self.update_nav_buttons();
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

#[cfg(test)]
mod tests {
    #[test]
    fn fingerprint_is_grouped() {
        let fp = "f357075437397896e31d320a3830d75637d627e824688b9660804d163e35f1f9";
        assert_eq!(
            super::format_fingerprint(fp),
            "<tt>f357 0754 3739 7896 e31d 320a 3830 d756\n37d6 27e8 2468 8b96 6080 4d16 3e35 f1f9</tt>"
        );
    }
}
