use gtk::{Application, prelude::*};
use gui::ConfigGUI;
use hyprparser::parse_config;
use std::{cell::RefCell, fs, rc::Rc};
use utils::{CONFIG_PATH, get_config_path};

mod gui;
mod utils;
mod widget;

fn main() {
    let app = Application::builder()
        .application_id("io.github.timasoft.hyprviz")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let gui = Rc::new(RefCell::new(ConfigGUI::new(app)));
    gui::ConfigGUI::setup_ui_events(Rc::clone(&gui));

    let config_path_full = get_config_path();

    if !config_path_full.exists() {
        gui.borrow().custom_error_popup_critical(
            "File not found",
            &format!("File not found: ~/{CONFIG_PATH}"),
        );
    } else {
        let config_str = match fs::read_to_string(&config_path_full) {
            Ok(s) => s,
            Err(e) => {
                gui.borrow().custom_error_popup_critical(
                    "Reading failed",
                    &format!("Failed to read the configuration file: {e}"),
                );
                String::new()
            }
        };
        let parsed_config = parse_config(&config_str);

        gui.borrow_mut().load_config(&parsed_config);
    }

    gui.borrow().window.present();
}
