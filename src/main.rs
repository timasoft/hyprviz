use gtk::{Application, StringList, StringObject, glib, prelude::*};
use gui::ConfigGUI;
use hyprparser::parse_config;
use std::path::Path;
use std::path::PathBuf;
use std::{cell::RefCell, fs, rc::Rc};
use utils::{
    CONFIG_PATH, HYPRVIZ_CONFIG_PATH, check_last_non_empty_line_contains, expand_source,
    find_all_profiles, get_config_path, get_current_profile, reload_hyprland, update_source_line,
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

        if !check_last_non_empty_line_contains(&config_str, "source = ./hyprviz") {
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

        let profile = get_current_profile(&config_str);

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

        let parsed_config = parse_config(&config_str_for_read);

        gui.borrow_mut().load_config(&parsed_config);

        let profiles = find_all_profiles();
        println!("Available profiles: {profiles:?}");

        println!("Loading config for profile: {profile}");
        match gui.borrow().profile_dropdown.model() {
            Some(model) => match model.downcast::<StringList>() {
                Ok(string_list) => {
                    let num_items = string_list.n_items();
                    let mut found_index = None;

                    for i in 0..num_items {
                        if let Some(gstring) = string_list.string(i)
                            && gstring.as_str() == profile
                        {
                            found_index = Some(i);
                            break;
                        }
                    }

                    match found_index {
                        Some(index) => {
                            gui.borrow().profile_dropdown.set_selected(index);
                        }
                        None => {
                            let config_dir = Path::new(HYPRVIZ_CONFIG_PATH)
                                .parent()
                                .and_then(|p| p.to_str())
                                .unwrap_or("config directory");

                            gui.borrow().custom_error_popup(
                                "Profile Not Found",
                                &format!(
                                    "Profile '{}' was not found in the config folder: {}",
                                    profile, config_dir
                                ),
                            );
                        }
                    }
                }
                Err(_) => {
                    gui.borrow().custom_error_popup_critical(
                        "Model Type Error",
                        "The dropdown model is not a StringList.",
                    );
                }
            },
            None => {
                gui.borrow().custom_error_popup_critical(
                    "Missing Model",
                    "The dropdown widget has no model assigned.",
                );
            }
        }

        let gui_clone = Rc::clone(&gui);
        gui.borrow()
            .profile_dropdown
            .connect_selected_notify(move |dropdown| {
                let selected_index = dropdown.selected();
                let model = dropdown.model().unwrap();

                if let Some(item) = model.item(selected_index)
                    && let Some(string_object) = item.downcast_ref::<StringObject>()
                {
                    let profile_name = string_object.string().as_str().to_string();
                    match update_source_line(&config_path_full, &profile_name) {
                        Ok(_) => {
                            println!("Updated source line to profile: {}", profile_name);
                            reload_hyprland();
                        }
                        Err(e) => {
                            gui_clone.borrow().custom_error_popup(
                                "Profile Switch Failed",
                                &format!(
                                    "Failed to update config for profile '{}': {}",
                                    profile_name, e
                                ),
                            );
                        }
                    }
                    let gui = Rc::clone(&gui_clone);

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

                    let parsed_config = parse_config(&config_str_for_read);
                    glib::MainContext::default().spawn_local(async move {
                        gui.borrow_mut().load_config(&parsed_config);
                    });
                }
            });
    }

    gui.borrow().window.present();
}
