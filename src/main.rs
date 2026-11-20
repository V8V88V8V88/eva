#![warn(clippy::all, clippy::pedantic)]
#![doc = include_str!("../README.md")]
use std::sync::{atomic::AtomicBool, LazyLock, Mutex};
/// Everything bookmark related
pub mod bookmarks;
/// Handles getting the configuration data to and from disk
pub mod config;
/// Starts the graphical interface
mod gui;
/// Handles history creation and deletion
pub mod history;
/// Handles loading keybindings
pub mod keys;

static CONFIG: LazyLock<Mutex<config::Config>> =
    LazyLock::new(|| Mutex::new(config::Config::from_file().unwrap_or_default()));
static BOOKMARKS: LazyLock<Mutex<bookmarks::Bookmarks>> = LazyLock::new(|| {
    Mutex::new(match bookmarks::Bookmarks::from_file() {
        Ok(b) => b.unwrap_or_default(),
        Err(_) => bookmarks::Bookmarks::default(),
    })
});
static HISTORY: LazyLock<Mutex<history::History>> = LazyLock::new(|| {
    Mutex::new(match history::History::from_file() {
        Ok(h) => h.unwrap_or_default(),
        Err(e) => {
            eprintln!("Error loading history: {e}");
            history::History::default()
        }
    })
});
static PRIVATE: AtomicBool = AtomicBool::new(false);
static SEARCH: LazyLock<gui::uri::Search> = LazyLock::new(gui::uri::Search::load);

fn main() {
    gui::run();
}
