use {
    super::Gui,
    crate::keys::Keys,
    gtk::{
        gio::SimpleAction,
        glib::{self, clone},
        prelude::*,
    },
    std::rc::Rc,
};

const ACTIONS: [&str; 29] = [
    "new_tab",
    "tab_overview",
    "focus_address",
    "close_tab",
    "next_tab",
    "prev_tab",
    "tab1",
    "tab2",
    "tab3",
    "tab4",
    "tab5",
    "tab6",
    "tab7",
    "tab8",
    "tab9",
    "reload",
    "go_home",
    "go_previous",
    "go_next",
    "new_window",
    "open_bookmarks",
    "bookmark_page",
    "open_history",
    "clear_history",
    "view_source",
    "save_page",
    "open_prefs",
    "open_about",
    "quit",
];

pub fn add(gui: &Rc<Gui>, app: &gtk::Application) {
    let keys = Keys::from_file().unwrap_or_default();
    for name in &ACTIONS {
        let action = SimpleAction::new(name, None);
        app.set_accels_for_action(&format!("win.{}", name), &[keys.get(name)]);
        gui.window.add_action(&action);
        match *name {
            "new_tab" => {
                action.connect_activate(clone!(
                    #[strong]
                    gui,
                    move |_, _| {
                        gui.open_new_tab();
                    }
                ));
            }
            "focus_address" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.focus_address_bar();
                    }
                ));
            }
            "tab_overview" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.open_tab_overview();
                    }
                ));
            }
            "close_tab" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.close_current_tab();
                    }
                ));
            }
            "next_tab" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.next_tab();
                    }
                ));
            }
            "prev_tab" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.prev_tab();
                    }
                ));
            }
            "tab1" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.select_tab(0);
                    }
                ));
            }
            "tab2" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.select_tab(1);
                    }
                ));
            }
            "tab3" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.select_tab(2);
                    }
                ));
            }
            "tab4" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.select_tab(3);
                    }
                ));
            }
            "tab5" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.select_tab(4);
                    }
                ));
            }
            "tab6" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.select_tab(5);
                    }
                ));
            }
            "tab7" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.select_tab(6);
                    }
                ));
            }
            "tab8" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.select_tab(7);
                    }
                ));
            }
            "tab9" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.select_tab(8);
                    }
                ));
            }
            "reload" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        if let Err(e) = gui.reload_current_tab() {
                            eprintln!("{}", e);
                        }
                    }
                ));
            }
            "go_home" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        if let Err(e) = gui.go_home() {
                            eprintln!("{}", e);
                        }
                    }
                ));
            }
            "go_previous" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        if let Err(e) = gui.go_previous() {
                            eprintln!("{}", e);
                        }
                    }
                ));
            }
            "go_next" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        if let Err(e) = gui.go_next() {
                            eprintln!("{}", e);
                        }
                    }
                ));
            }
            "new_window" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    #[strong]
                    app,
                    move |_, _| {
                        let new_gui = crate::gui::build_ui(&app);
                        new_gui.new_tab(None);
                    }
                ));
            }
            "open_bookmarks" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.open_bookmarks();
                    }
                ));
            }
            "bookmark_page" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        if let Some(tab) = gui.current_tab() {
                            tab.bookmark_editor.popup();
                        }
                    }
                ));
            }
            "open_history" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        println!("Not implemented yet");
                    }
                ));
            }
            "clear_history" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        println!("Not implemented yet");
                    }
                ));
            }
            "view_source" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        if let Some(tab) = gui.current_tab() {
                            tab.view_source();
                        }
                    }
                ));
            }
            "save_page" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.save_page();
                    }
                ));
            }
            "open_prefs" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.dialogs.preferences.show();
                    }
                ));
            }
            "open_about" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.dialogs.about.show();
                    }
                ));
            }
            "quit" => {
                action.connect_activate(clone!(
                    #[weak]
                    gui,
                    move |_, _| {
                        gui.window.close();
                    }
                ));
            }
            _ => {}
        }
    }
}
