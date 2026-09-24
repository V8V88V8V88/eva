use {
    crate::config::get_config_dir,
    serde::Deserialize,
    std::{collections::HashMap, fs, path::PathBuf},
};

/// Returns the path to keys.toml
#[allow(clippy::must_use_candidate)]
pub fn get_key_file() -> PathBuf {
    let mut file = get_config_dir();
    file.push("keys.toml");
    file
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Keys {
    keys: HashMap<String, String>,
}

impl Keys {
    #[must_use]
    pub fn get(&self, action: &str) -> &str {
        if let Some(key) = self.keys.get(action) {
            if gtk::accelerator_parse(key).is_some() {
                return key;
            }
        }
        match action {
            "new_tab" => "<primary>T",
            "tab_overview" => "<primary><Shift>E",
            "close_tab" => "<primary>W",
            "next_tab" => "<primary>Page_Down",
            "prev_tab" => "<primary>Page_Up",
            "tab1" => "<Alt>1",
            "tab2" => "<Alt>2",
            "tab3" => "<Alt>3",
            "tab4" => "<Alt>4",
            "tab5" => "<Alt>5",
            "tab6" => "<Alt>6",
            "tab7" => "<Alt>7",
            "tab8" => "<Alt>8",
            "tab9" => "<Alt>9",
            "reload" => "<primary>R",
            "go_home" => "<Alt>Home",
            "go_previous" => "<Alt>Left",
            "go_next" => "<Alt>Right",
            "new_window" => "<primary>N",
            "open_bookmarks" => "<primary><Shift>O",
            "bookmark_page" => "<primary>D",
            "open_history" => "<primary>H",
            "view_source" => "<primary>U",
            "save_page" => "<primary>S",
            "open_prefs" => "<primary><Shift>P",
            "open_about" => "<primary><Shift>A",
            "quit" => "<primary>Q",
            _ => "",
        }
    }

    #[must_use]
    pub fn from_file() -> Option<Self> {
        let keyfile = get_key_file();
        let keyfile = if keyfile.exists() {
            match fs::read_to_string(keyfile) {
                Ok(k) => k,
                Err(_) => return None,
            }
        } else {
            return None;
        };
        let keys: Self = match toml::from_str(&keyfile) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        };
        Some(keys)
    }
}
