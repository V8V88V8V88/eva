#![warn(clippy::all, clippy::pedantic)]
use {
    rgba_simple::{PrimaryColor, RGBA},
    serde::{Deserialize, Serialize},
    std::{
        fs, io,
        path::{Path, PathBuf},
    },
};

mod fonts;

pub use fonts::{Font, Fonts};

/// Returns an OS appropriate configuration directory path
///
/// # Panics
/// Can panic if the string returned from [`gtk::glib::user_config_dir`] is not valid
/// unicode (unlikely)
#[must_use]
pub fn get_config_dir() -> PathBuf {
    let mut configdir: PathBuf = gtk::glib::user_config_dir();
    let progname = env!("CARGO_PKG_NAME");
    configdir.push(progname);
    if !configdir.exists() {
        fs::create_dir(&configdir.to_str().unwrap()).unwrap_or_else(|e| eprintln!("{}", e));
    }
    configdir
}

/// Returns the path to config.toml
#[allow(clippy::must_use_candidate)]
pub fn get_config_file() -> PathBuf {
    let mut file = get_config_dir();
    file.push("config.toml");
    file
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Colors {
    pub fg: RGBA<u8>,
    pub bg: RGBA<u8>,
    pub pre_fg: RGBA<u8>,
    pub pre_bg: RGBA<u8>,
    pub quote_fg: RGBA<u8>,
    pub quote_bg: RGBA<u8>,
    pub link: RGBA<u8>,
    pub hover: RGBA<u8>,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            fg: RGBA {
                red: 153,
                green: 193,
                blue: 241,
                alpha: 255,
            },
            bg: RGBA {
                red: 36,
                green: 31,
                blue: 49,
                alpha: 255,
            },
            pre_fg: RGBA {
                red: 98,
                green: 160,
                blue: 234,
                alpha: 255,
            },
            pre_bg: RGBA {
                red: 5,
                green: 34,
                blue: 79,
                alpha: 255,
            },
            quote_fg: RGBA {
                red: 87,
                green: 227,
                blue: 137,
                alpha: 255,
            },
            quote_bg: RGBA {
                red: 22,
                green: 110,
                blue: 0,
                alpha: 255,
            },
            link: RGBA {
                red: 192,
                green: 97,
                blue: 203,
                alpha: 255,
            },
            hover: PrimaryColor::Red.into(),
        }
    }
}

impl Colors {
    #[must_use]
    pub fn fg(&self) -> RGBA<u8> {
        self.fg
    }

    pub fn set_fg(&mut self, color: RGBA<u8>) {
        self.fg = color;
    }

    #[must_use]
    pub fn bg(&self) -> RGBA<u8> {
        self.bg
    }

    pub fn set_bg(&mut self, color: RGBA<u8>) {
        self.bg = color;
    }

    #[must_use]
    pub fn pre_fg(&self) -> RGBA<u8> {
        self.pre_fg
    }

    pub fn set_pre_fg(&mut self, color: RGBA<u8>) {
        self.pre_fg = color;
    }

    #[must_use]
    pub fn pre_bg(&self) -> RGBA<u8> {
        self.pre_bg
    }

    pub fn set_pre_bg(&mut self, color: RGBA<u8>) {
        self.pre_bg = color;
    }

    #[must_use]
    pub fn quote_fg(&self) -> RGBA<u8> {
        self.quote_fg
    }

    pub fn set_quote_fg(&mut self, color: RGBA<u8>) {
        self.quote_fg = color;
    }

    #[must_use]
    pub fn quote_bg(&self) -> RGBA<u8> {
        self.quote_bg
    }

    pub fn set_quote_bg(&mut self, color: RGBA<u8>) {
        self.quote_bg = color;
    }

    #[must_use]
    pub fn link(&self) -> RGBA<u8> {
        self.link
    }

    pub fn set_link(&mut self, color: RGBA<u8>) {
        self.link = color;
    }

    #[must_use]
    pub fn hover(&self) -> RGBA<u8> {
        self.hover
    }

    pub fn set_hover(&mut self, color: RGBA<u8>) {
        self.hover = color;
    }
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Serialize)]
pub enum NewPage {
    Home,
    Blank,
}

impl Default for NewPage {
    fn default() -> Self {
        Self::Home
    }
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Serialize)]
pub enum ShowTabs {
    Always,
    Multiple,
    Never,
}

impl Default for ShowTabs {
    fn default() -> Self {
        Self::Always
    }
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Serialize)]
pub enum TabPosition {
    Top,
    Bottom,
    Left,
    Right,
}

impl Default for TabPosition {
    fn default() -> Self {
        Self::Top
    }
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Serialize)]
pub enum DownloadScheme {
    Ask,
    Auto,
}

impl Default for DownloadScheme {
    fn default() -> Self {
        Self::Ask
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct General {
    pub homepage: String,
    pub new_page: NewPage,
    pub show_tabs: ShowTabs,
    pub tab_position: TabPosition,
    pub download_scheme: DownloadScheme,
    pub download_location: Option<String>,
}

impl Default for General {
    fn default() -> Self {
        Self {
            homepage: String::from("gemini://gemini.circumlunar.space/"),
            new_page: NewPage::default(),
            show_tabs: ShowTabs::default(),
            tab_position: TabPosition::default(),
            download_scheme: DownloadScheme::default(),
            download_location: None,
        }
    }
}

#[derive(Clone, Default, Deserialize, Debug, Serialize)]
pub struct Config {
    pub general: General,
    pub colors: Colors,
    pub fonts: Fonts,
}

impl Config {
    /// Saves Config struct as a .toml file
    /// # Errors
    /// Returns an io error if unable to save the config file
    pub fn save_to_file(&self, file: &Path) -> Result<(), io::Error> {
        let toml_string = match toml::to_string(&self) {
            Ok(t) => t,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };
        fs::write(file, toml_string)?;
        Ok(())
    }

    /// Deserializes config.toml into a `GfretConfig` struct
    #[must_use]
    pub fn from_file() -> Option<Self> {
        let config_file = get_config_file();
        let config_file = if config_file.exists() {
            match fs::read_to_string(config_file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    return None;
                }
            }
        } else {
            return None;
        };
        let config: Self = match toml::from_str(&config_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        };
        Some(config)
    }
}
