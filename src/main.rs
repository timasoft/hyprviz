use gtk::{Application, StringList, StringObject, glib, prelude::*};
use gui::ConfigGUI;
use hyprparser::parse_config;
use rust_i18n::{i18n, t};
use std::{
    path::{Path, PathBuf},
    {cell::RefCell, fs, rc::Rc},
};
use utils::{
    CONFIG_PATH, HYPRVIZ_CONFIG_PATH, HYPRVIZ_PROFILES_PATH, atomic_write,
    check_last_non_empty_line_contains, expand_source, find_all_profiles, get_config_path,
    get_current_profile, get_system_locale, reload_hyprland, update_source_line,
};

mod gui;
mod guides;
mod system_info;
mod utils;
mod widget;

i18n!("locales", fallback = "en");

fn main() {
    rust_i18n::set_locale(&get_system_locale());
    let app = Application::builder()
        .application_id("io.github.timasoft.hyprviz")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let gui = Rc::new(RefCell::new(ConfigGUI::new(app)));
    gui::ConfigGUI::setup_ui_events(Rc::clone(&gui));

    let config_path_full = get_config_path(false, "Default");

    if !config_path_full.exists() {
        gui.borrow().custom_error_popup_critical(
            &t!("file_not_found"),
            &t!("file_not_found_~/_", file = CONFIG_PATH),
        );
    } else {
        let hyprviz_profile_none_path = get_config_path(true, "None");
        let hyprviz_profiles_path = hyprviz_profile_none_path
            .parent()
            .unwrap_or_else(|| Path::new(HYPRVIZ_PROFILES_PATH));
        if !hyprviz_profiles_path.exists() {
            match fs::create_dir_all(hyprviz_profiles_path) {
                Ok(_) => {}
                Err(e) => {
                    gui.borrow().custom_error_popup_critical(
                        &t!("creating_failed"),
                        &t!("failed_to_create_the_profile_directory_", error = e),
                    );
                    return;
                }
            }
        } else if !hyprviz_profiles_path.is_dir() {
            gui.borrow().custom_error_popup_critical(
                &t!("creating_failed"),
                &t!("the_profile_directory_is_not_a_directory"),
            );
            return;
        }

        let config_str = match fs::read_to_string(&config_path_full) {
            Ok(s) => s,
            Err(e) => {
                gui.borrow().custom_error_popup_critical(
                    &t!("reading_failed"),
                    &t!("failed_to_read_the_configuration_file_", error = e),
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
                        &t!("creating_included_file_failed"),
                        &t!(
                            "path_for_included_file_exists_but_is_not_a_regular_file_",
                            file = hyprviz_path.display()
                        ),
                    );
                }
            } else {
                if let Some(parent) = hyprviz_path.parent()
                    && let Err(e) = fs::create_dir_all(parent)
                {
                    gui.borrow().custom_error_popup_critical(
                        &t!("creating_included_file_failed"),
                        &t!(
                            "failed_to_create_parent_directory_for__",
                            file = hyprviz_path.display(),
                            error = e
                        ),
                    );
                }

                let default = "# hyprviz configuration (created automatically)\n\n";
                if let Err(e) = atomic_write(&hyprviz_path, default) {
                    gui.borrow().custom_error_popup_critical(
                        &t!("creating_included_file_failed"),
                        &t!(
                            "failed_to_create__",
                            file = hyprviz_path.display(),
                            error = e
                        ),
                    );
                }
            }

            match atomic_write(&config_path_full, &updated_config_str) {
                Ok(_) => {
                    println!("Added 'source = ./hyprviz.conf' to ~/{}", CONFIG_PATH);
                    reload_hyprland();
                }
                Err(e) => {
                    gui.borrow().custom_error_popup_critical(
                        &t!("saving_failed"),
                        &t!(
                            "failed_to_add_source_line_to__",
                            file = CONFIG_PATH,
                            error = e
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
                    &t!("reading_failed"),
                    &t!("failed_to_read_the_configuration_file_", error = e),
                );
                String::new()
            }
        };

        let parsed_config = parse_config(&config_str_for_read);

        gui.borrow_mut().load_config(&parsed_config, &profile);

        let profiles = find_all_profiles();
        println!("Available profiles: {:?}", profiles);

        println!("Loading config for profile: {}", profile);
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
                                &t!("profile_not_found"),
                                &t!(
                                    "profile__was_not_found_in_the_config_folder_",
                                    name = profile,
                                    path = config_dir
                                ),
                            );
                        }
                    }
                }
                Err(_) => {
                    gui.borrow().custom_error_popup_critical(
                        &t!("model_type_error"),
                        &t!("the_dropdown_model_is_not_a_stringlist"),
                    );
                }
            },
            None => {
                gui.borrow().custom_error_popup_critical(
                    &t!("missing_model"),
                    &t!("the_dropdown_widget_has_no_model_assigned"),
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
                                &t!("profile_switch_failed"),
                                &t!(
                                    "failed_to_update_config_for_profile__",
                                    profile = profile_name,
                                    error = e
                                ),
                            );
                        }
                    }
                    let gui = Rc::clone(&gui_clone);

                    let config_str_for_read = match expand_source(&config_path_full) {
                        Ok(s) => s,
                        Err(e) => {
                            gui.borrow().custom_error_popup_critical(
                                &t!("reading_failed"),
                                &t!("failed_to_read_the_configuration_file_", error = e),
                            );
                            String::new()
                        }
                    };

                    let parsed_config = parse_config(&config_str_for_read);
                    glib::MainContext::default().spawn_local(async move {
                        gui.borrow_mut().load_config(&parsed_config, &profile_name);
                    });
                }
            });
    }

    gui.borrow().window.present();
}
