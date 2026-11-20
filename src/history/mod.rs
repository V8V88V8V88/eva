#![warn(clippy::all, clippy::pedantic)]
use {
    chrono::prelude::*,
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, error::Error, fmt::Write, path::PathBuf},
};

#[must_use]
pub fn get_data_dir() -> PathBuf {
    let mut datadir = gtk::glib::user_data_dir();
    let progname = env!("CARGO_PKG_NAME");
    datadir.push(progname);
    datadir
}

#[must_use]
pub fn get_history_file() -> PathBuf {
    let mut histfile = get_data_dir();
    histfile.push("history.toml");
    histfile
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct History {
    items: HashMap<String, chrono::DateTime<Local>>,
}

impl History {
    pub fn append(&mut self, url: &str) {
        let _old = self.items.insert(String::from(url), Local::now());
    }

    pub fn remove(&mut self, url: &str) {
        let _old = self.items.remove(url);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[must_use]
    pub fn page(&self) -> String {
        let mut page = String::from("# History\n");
        if self.items.is_empty() {
            page.push_str("\nNo pages visited yet.\n");
            return page;
        }
        let mut entries: Vec<_> = self.items.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        let mut day = None;
        for (url, date) in entries {
            let d = date.date_naive();
            if day != Some(d) {
                let _ = write!(page, "\n## {}\n", d.format("%A, %B %-d, %Y"));
                day = Some(d);
            }
            let _ = writeln!(page, "=> {url} {} {url}", date.format("%H:%M"));
        }
        page
    }

    /// # Errors
    /// Returns an error if unable to get the data directory path, unable to
    /// create the data directory, unable to serialize toml or unable to write
    /// the toml to disk
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let datadir = get_data_dir();
        let histfile = get_history_file();
        if !datadir.exists() {
            let dd = match datadir.to_str() {
                Some(d) => d,
                None => return Err(String::from("Empty data directory path").into()),
            };
            std::fs::create_dir(&dd)?;
        }
        let toml_string = toml::to_string(self)?;
        std::fs::write(histfile, toml_string)?;
        Ok(())
    }

    /// # Errors
    /// Returns an error if unable to read the history from disk, or unable to
    /// deserialize toml
    pub fn from_file() -> Result<Option<Self>, Box<dyn Error>> {
        let histfile = get_history_file();
        let histfile = if histfile.exists() {
            std::fs::read_to_string(histfile)?
        } else {
            return Ok(None);
        };
        let history = toml::from_str(&histfile)?;
        Ok(Some(history))
    }
}
