pub mod bookmark_editor;
pub mod controls;
pub mod input;
pub mod label;
pub use {bookmark_editor::BookmarkEditor, controls::Controls, input::Input, label::Label};

use {
    super::uri,
    crate::{BOOKMARKS, CONFIG},
    gemview::GemView,
    gtk::{glib::clone, prelude::*},
    std::{
        fs::File,
        io::{BufReader, Read},
    },
    url::Url,
};

#[derive(Clone, Debug)]
pub struct Tab {
    tab: gtk::Box,
    pub label: Label,
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
        controls.set_input_popover(Some(&input));
        controls.set_bookmark_popover(Some(&bookmark_editor));
        tab.append(&controls);
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
        tab.append(&scroller);

        Self {
            tab,
            label: Label::default(),
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
        tab.controls.set_back_button_sensitive(false);
        tab.controls.set_forward_button_sensitive(false);
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
            move |_, uri| {
                if let Some((scheme, _)) = uri.split_once(':') {
                    match scheme {
                        "eva" => tab.request_eva_page(&uri),
                        _ => {
                            if let Err(e) = mime_open::open(&uri) {
                                eprintln!("Error opening {}: {}", uri, e);
                            }
                        }
                    }
                }
            }
        ));
        let upload = self.upload.clone();
        self.viewer.connect_request_upload(move |_viewer, _url| {
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
        let popover = self.input.clone();
        self.input.set_visibility(visibility);
        self.input.entry().connect_activate(move |entry| {
            let response = entry.text();
            if response.as_str() != "" {
                let mut url = url.to_string();
                url.push('?');
                let response = urlencoding::encode(response.as_str());
                url.push_str(&response);
                viewer.visit(&url);
                popover.popdown();
            }
        });
        self.input.request(meta);
    }

    pub fn tab(&self) -> gtk::Box {
        self.tab.clone()
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
        self.label.set(label, spin);
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
