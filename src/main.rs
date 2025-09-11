use gtk::{Application, prelude::*};
use gui::ConfigGUI;
use hyprparser::parse_config;
use std::path::PathBuf;
use std::{cell::RefCell, fs, rc::Rc};
use utils::{
    CONFIG_PATH, check_last_non_empty_line, expand_source, get_config_path, reload_hyprland,
};

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

    let config_path_full = get_config_path(false);

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
        let config_str_for_read = match expand_source(&config_path_full) {
            Ok(s) => s,
            Err(e) => {
                gui.borrow().custom_error_popup_critical(
                    "Reading failed",
                    &format!("Failed to read the configuration file: {e}"),
                );
                String::new()
            }
        };

        if !check_last_non_empty_line(&config_str, "source = ./hyprviz.conf") {
            let mut parsed_config = parse_config(&config_str);

            parsed_config.add_entry_headless("#", "Source for hyprviz");
            parsed_config.add_entry_headless("source", "./hyprviz.conf");

            let updated_config_str = parsed_config.to_string();

            let hyprviz_path: PathBuf = config_path_full
                .parent()
                .map(|d| d.join("hyprviz.conf"))
                .unwrap_or_else(|| PathBuf::from("./hyprviz.conf"));

            if hyprviz_path.exists() {
                if !hyprviz_path.is_file() {
                    gui.borrow().custom_error_popup_critical(
                        "Creating included file failed",
                        &format!(
                            "Path for included file exists but is not a regular file: {}",
                            hyprviz_path.display()
                        ),
                    );
                }
            } else {
                if let Some(parent) = hyprviz_path.parent()
                    && let Err(e) = fs::create_dir_all(parent)
                {
                    gui.borrow().custom_error_popup_critical(
                        "Creating included file failed",
                        &format!(
                            "Failed to create parent directory {} for {}: {}",
                            parent.display(),
                            hyprviz_path.display(),
                            e
                        ),
                    );
                }

                let default = "# hyprviz configuration (created automatically)\n\n";
                if let Err(e) = fs::write(&hyprviz_path, default) {
                    gui.borrow().custom_error_popup_critical(
                        "Creating included file failed",
                        &format!("Failed to create {}: {}", hyprviz_path.display(), e),
                    );
                }
            }

            match fs::write(&config_path_full, updated_config_str) {
                Ok(_) => {
                    println!("Added 'source = ./hyprviz.conf' to: ~/{CONFIG_PATH}");
                    reload_hyprland();
                }
                Err(e) => {
                    gui.borrow().custom_error_popup_critical(
                        "Saving failed",
                        &format!(
                            "Failed to add 'source = ./hyprviz.conf' to: ~/{CONFIG_PATH}: {e}"
                        ),
                    );
                }
            }
        }

        let parsed_config = parse_config(&config_str_for_read);

        gui.borrow_mut().load_config(&parsed_config);
    }

    gui.borrow().window.present();
}
