use crate::{
    utils::{
        BACKUP_SUFFIX, HYPRVIZ_PROFILES_PATH, atomic_write, expand_source, find_all_profiles,
        get_config_path, is_development_mode, reload_hyprland,
    },
    widget::ConfigWidget,
};
use gtk::{
    AlertDialog, Application, ApplicationWindow, Box, Button, ColorDialogButton, DropDown, Entry,
    FileDialog, HeaderBar, Label, Orientation, Popover, ScrolledWindow, SearchEntry, SpinButton,
    Stack, StackSidebar, StringList, StringObject, Switch, Widget, Window, gdk, glib, prelude::*,
};
use hyprparser::{HyprlandConfig, parse_config};
use rust_i18n::{available_locales, locale, set_locale, t};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

pub struct ConfigGUI {
    pub window: ApplicationWindow,
    config_widgets: HashMap<String, ConfigWidget>,
    title_label: Label,
    save_button: Button,
    undo_button: Button,
    pub profile_dropdown: DropDown,
    current_profile_label: Label,
    create_profile_button: Button,
    delete_profile_button: Button,
    clear_backups_button: Button,
    save_config_button: Button,
    load_config_button: Button,
    copy_button: Button,
    search_entry: SearchEntry,
    locale_dropdown: DropDown,
    changed_options: Rc<RefCell<HashMap<(String, String), String>>>,
    content_box: Box,
    stack: Stack,
    sidebar: StackSidebar,
}

impl ConfigGUI {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(1000)
            .default_height(600)
            .build();

        let title_label = Label::new(Some(&t!("gui.hyprland_configuration")));

        let header_bar = HeaderBar::builder()
            .show_title_buttons(false)
            .title_widget(&title_label)
            .build();

        let gear_button = Button::from_icon_name("emblem-system-symbolic");
        header_bar.pack_start(&gear_button);

        let gear_menu = Rc::new(RefCell::new(Popover::new()));
        gear_menu.borrow().set_parent(&gear_button);

        let gear_menu_box = Box::new(Orientation::Vertical, 5);
        gear_menu_box.set_margin_top(5);
        gear_menu_box.set_margin_bottom(5);
        gear_menu_box.set_margin_start(5);
        gear_menu_box.set_margin_end(5);

        let search_button = Button::from_icon_name("system-search-symbolic");
        let search_entry = SearchEntry::new();
        search_entry.set_width_chars(25);

        let popover = gtk::Popover::new();
        popover.set_child(Some(&search_entry));
        popover.set_position(gtk::PositionType::Bottom);
        popover.set_parent(&search_button);

        let create_profile_button = Button::with_label(&t!("gui.create_profile"));
        let delete_profile_button = Button::with_label(&t!("gui.delete_profile"));
        let clear_backups_button = Button::with_label(&t!("gui.clear_backups_files"));
        let load_config_button = Button::with_label(&t!("gui.load_hyprviz_config"));
        let save_config_button = Button::with_label(&t!("gui.save_hyprviz_config"));
        let copy_button = Button::with_label(&t!("gui.copyright"));

        gear_menu_box.append(&create_profile_button);
        gear_menu_box.append(&delete_profile_button);
        gear_menu_box.append(&clear_backups_button);
        gear_menu_box.append(&load_config_button);
        gear_menu_box.append(&save_config_button);
        gear_menu_box.append(&copy_button);

        gear_menu.borrow().set_child(Some(&gear_menu_box));

        let gear_menu_clone = gear_menu.clone();
        gear_button.connect_clicked(move |_| {
            gear_menu_clone.borrow().popup();
        });

        let popover_clone = popover.clone();
        let search_entry_clone = search_entry.clone();
        search_button.connect_clicked(move |_| {
            if !popover_clone.is_visible() {
                popover_clone.popup();
                search_entry_clone.grab_focus();
            }
        });

        let popover_clone = popover.clone();
        search_entry.connect_activate(move |_| {
            popover_clone.popdown();
        });

        let popover_clone = popover.clone();
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                popover_clone.popdown();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        search_entry.add_controller(key_controller);

        header_bar.pack_start(&search_button);

        let locales = available_locales!();
        let locales_string_list = StringList::new(locales.as_slice());
        let locale_dropdown =
            DropDown::new(Some(locales_string_list.clone()), None::<gtk::Expression>);

        let selected_locale_id = locales
            .iter()
            .position(|s| &locale().to_string() == s)
            .unwrap() as u32;
        locale_dropdown.set_selected(selected_locale_id);

        header_bar.pack_start(&locale_dropdown);

        let save_button = Button::with_label(&t!("gui.save"));
        let undo_button = Button::with_label(&t!("gui.undo"));

        let profiles = if let Some(mut profiles) = find_all_profiles() {
            if profiles.contains(&"Default".to_string()) {
                profiles
            } else {
                profiles.insert(0, "Default".to_string());
                profiles
            }
        } else {
            vec!["Default".to_string()]
        };
        let profiles_str_vec: Vec<&str> = profiles.iter().map(|s| s.as_str()).collect();
        let profiles_str: &[&str] = profiles_str_vec.as_slice();
        let string_list = StringList::new(profiles_str);
        let profile_dropdown = DropDown::new(Some(string_list.clone()), None::<gtk::Expression>);
        profile_dropdown.set_halign(gtk::Align::End);
        profile_dropdown.set_width_request(100);
        let current_profile_label = Label::new(Some(&t!("gui.profile")));

        header_bar.pack_end(&save_button);
        header_bar.pack_end(&undo_button);
        header_bar.pack_end(&profile_dropdown);
        header_bar.pack_end(&current_profile_label);

        window.set_titlebar(Some(&header_bar));

        let main_box = Box::new(Orientation::Vertical, 0);

        let content_box = Box::new(Orientation::Horizontal, 0);
        main_box.append(&content_box);

        window.set_child(Some(&main_box));

        let config_widgets = HashMap::new();

        let stack = Stack::new();

        let sidebar = StackSidebar::new();
        sidebar.set_stack(&stack);
        sidebar.set_width_request(200);

        ConfigGUI {
            window,
            config_widgets,
            title_label,
            save_button,
            undo_button,
            profile_dropdown,
            current_profile_label,
            create_profile_button,
            delete_profile_button,
            clear_backups_button,
            save_config_button,
            load_config_button,
            copy_button,
            search_entry,
            locale_dropdown,
            content_box,
            changed_options: Rc::new(RefCell::new(HashMap::new())),
            stack,
            sidebar,
        }
    }

    pub fn setup_ui_events(gui: Rc<RefCell<ConfigGUI>>) {
        let gui_clone = Rc::clone(&gui);
        gui.borrow()
            .create_profile_button
            .connect_clicked(move |_| {
                let gui = Rc::clone(&gui_clone);

                let dialog_window = Window::builder()
                    .title(t!("gui.create_new_profile").to_string())
                    .modal(true)
                    .transient_for(&gui.borrow().window)
                    .destroy_with_parent(true)
                    .default_width(300)
                    .build();

                let dialog_box = Box::new(Orientation::Vertical, 10);
                dialog_box.set_margin_top(10);
                dialog_box.set_margin_bottom(10);
                dialog_box.set_margin_start(10);
                dialog_box.set_margin_end(10);

                let label = Label::new(Some(&t!("gui.enter_profile_name")));
                let entry = Entry::new();
                entry.set_placeholder_text(Some(&t!("gui.new_profile")));

                dialog_box.append(&label);
                dialog_box.append(&entry);

                let buttons_box = Box::new(Orientation::Horizontal, 5);
                buttons_box.set_halign(gtk::Align::End);

                let cancel_button = Button::with_label(&t!("gui.cancel"));
                let create_button = Button::with_label(&t!("gui.create"));

                buttons_box.append(&cancel_button);
                buttons_box.append(&create_button);

                dialog_box.append(&buttons_box);
                dialog_window.set_child(Some(&dialog_box));

                let dialog_window_clone = dialog_window.clone();
                cancel_button.connect_clicked(move |_| {
                    dialog_window_clone.close();
                });

                let dialog_window_clone = dialog_window.clone();
                let gui_clone = Rc::clone(&gui);
                let entry_clone = entry.clone();
                create_button.connect_clicked(move |_| {
                    let profile_name = entry_clone.text().to_string();
                    if profile_name.is_empty() || profile_name == "Default" {
                        gui_clone.borrow().custom_error_popup(
                            &t!("gui.invalid_profile_name"),
                            &t!("gui.profile_name_cannot_be_empty_or_default"),
                        );
                        return;
                    }

                    let hyprviz_path = get_config_path(true, &profile_name);

                    if hyprviz_path.exists() {
                        gui_clone.borrow().custom_error_popup(
                            &t!("gui.profile_exists"),
                            &t!("gui.profile__already_exists", name = profile_name),
                        );
                        return;
                    }

                    let config_path_full = get_config_path(true, "Default");
                    let config_str = match fs::read_to_string(&config_path_full) {
                        Ok(s) => s,
                        Err(e) => {
                            gui_clone.borrow().custom_error_popup_critical(
                                &t!("gui.reading_failed"),
                                &t!(
                                    "gui.failed_to_read_the_default_configuration_file_",
                                    error = e
                                ),
                            );
                            return;
                        }
                    };

                    if let Err(e) = atomic_write(&hyprviz_path, &config_str) {
                        gui_clone.borrow().custom_error_popup(
                            &t!("gui.failed_to_create_profile"),
                            &t!("gui.failed_to_create_profile_file_", error = e),
                        );
                        return;
                    }

                    dialog_window_clone.close();

                    if let Some(mut profiles) = find_all_profiles() {
                        if !profiles.contains(&"Default".to_string()) {
                            profiles.insert(0, "Default".to_string());
                        }
                        let profiles_str_vec: Vec<&str> =
                            profiles.iter().map(|s| s.as_str()).collect();
                        let string_list = StringList::new(&profiles_str_vec);
                        gui_clone
                            .borrow()
                            .profile_dropdown
                            .set_model(Some(&string_list));

                        if let Some(pos) = profiles.iter().position(|p| *p == profile_name) {
                            gui_clone.borrow().profile_dropdown.set_selected(pos as u32);
                        }
                    }
                });

                dialog_window.present();
            });

        let gui_clone = Rc::clone(&gui);
        gui.borrow()
            .delete_profile_button
            .connect_clicked(move |_| {
                let gui = Rc::clone(&gui_clone);

                let selected_index = gui.borrow().profile_dropdown.selected();
                let model = gui.borrow().profile_dropdown.model().unwrap();
                let profile_name = if let Some(item) = model.item(selected_index) {
                    if let Some(string_object) = item.downcast_ref::<StringObject>() {
                        string_object.string().as_str().to_string()
                    } else {
                        "Default".to_string()
                    }
                } else {
                    "Default".to_string()
                };

                if profile_name == "Default" {
                    gui.borrow().custom_error_popup(
                        &t!("gui.cannot_delete_profile"),
                        &t!("gui.the_default_profile_cannot_be_deleted"),
                    );
                    return;
                }

                let dialog_window = Window::builder()
                    .title(t!("gui.delete_profile"))
                    .modal(true)
                    .transient_for(&gui.borrow().window)
                    .destroy_with_parent(true)
                    .default_width(300)
                    .build();

                let dialog_box = Box::new(Orientation::Vertical, 10);
                dialog_box.set_margin_top(10);
                dialog_box.set_margin_bottom(10);
                dialog_box.set_margin_start(10);
                dialog_box.set_margin_end(10);

                let label = Label::new(Some(&t!(
                    "gui.are_you_sure_you_want_to_delete_the_profile_",
                    name = profile_name
                )));
                label.set_wrap(true);
                label.set_width_chars(45);
                label.set_max_width_chars(60);
                label.set_halign(gtk::Align::Center);

                dialog_box.append(&label);

                let buttons_box = Box::new(Orientation::Horizontal, 5);
                buttons_box.set_halign(gtk::Align::End);

                let cancel_button = Button::with_label(&t!("gui.cancel"));
                let delete_button = Button::with_label(&t!("gui.delete"));

                buttons_box.append(&cancel_button);
                buttons_box.append(&delete_button);

                dialog_box.append(&buttons_box);
                dialog_window.set_child(Some(&dialog_box));

                let dialog_window_clone = dialog_window.clone();
                cancel_button.connect_clicked(move |_| {
                    dialog_window_clone.close();
                });

                let dialog_window_clone = dialog_window.clone();
                let gui_clone = Rc::clone(&gui);
                let profile_name_clone = profile_name.clone();
                delete_button.connect_clicked(move |_| {
                    let hyprviz_path = get_config_path(true, &profile_name_clone);

                    match fs::remove_file(&hyprviz_path) {
                        Ok(_) => {
                            if let Some(mut profiles) = find_all_profiles() {
                                if !profiles.contains(&"Default".to_string()) {
                                    profiles.insert(0, "Default".to_string());
                                }

                                let profiles_str_vec: Vec<&str> =
                                    profiles.iter().map(|s| s.as_str()).collect();
                                let string_list = StringList::new(&profiles_str_vec);
                                gui_clone
                                    .borrow()
                                    .profile_dropdown
                                    .set_model(Some(&string_list));

                                if let Some(pos) = profiles.iter().position(|p| *p == "Default") {
                                    gui_clone.borrow().profile_dropdown.set_selected(pos as u32);
                                }
                            }

                            dialog_window_clone.close();
                        }
                        Err(e) => {
                            gui_clone.borrow().custom_error_popup(
                                &t!("gui.failed_to_delete_profile"),
                                &t!("gui.failed_to_delete_profile_file_", error = e),
                            );
                        }
                    }
                });

                dialog_window.present();
            });

        let gui_clone = Rc::clone(&gui);
        gui.borrow().clear_backups_button.connect_clicked(move |_| {
            let gui = Rc::clone(&gui_clone);

            let none_config = get_config_path(true, "None");

            let config_dir = none_config
                .parent()
                .unwrap_or_else(|| Path::new(HYPRVIZ_PROFILES_PATH));

            let mut backup_files = Vec::new();
            if let Ok(entries) = fs::read_dir(config_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    if let Some(name) = file_name.to_str()
                        && name.ends_with(BACKUP_SUFFIX)
                    {
                        backup_files.push(entry.path());
                    }
                }
            }

            if backup_files.is_empty() {
                gui.borrow().custom_info_popup(
                    &t!("gui.no_backup_files"),
                    &t!("gui.no_backup_files_found_to_delete"),
                );
                return;
            }

            let dialog_window = Window::builder()
                .title(t!("gui.clear_backups_files"))
                .modal(true)
                .transient_for(&gui.borrow().window)
                .destroy_with_parent(true)
                .default_width(300)
                .build();

            let dialog_box = Box::new(Orientation::Vertical, 10);
            dialog_box.set_margin_top(10);
            dialog_box.set_margin_bottom(10);
            dialog_box.set_margin_start(10);
            dialog_box.set_margin_end(10);

            let label = Label::new(Some(&format!(
                "{}\n{}.",
                &t!(
                    "gui.are_you_sure_you_want_to_delete__backup_files",
                    n = backup_files.len()
                ),
                &t!("gui.this_operation_cannot_be_undone"),
            )));
            label.set_wrap(true);
            label.set_width_chars(50);
            label.set_max_width_chars(60);
            label.set_halign(gtk::Align::Center);

            dialog_box.append(&label);

            let buttons_box = Box::new(Orientation::Horizontal, 5);
            buttons_box.set_halign(gtk::Align::End);

            let cancel_button = Button::with_label(&t!("gui.cancel"));
            let clear_button = Button::with_label(&t!("gui.clear"));

            buttons_box.append(&cancel_button);
            buttons_box.append(&clear_button);

            dialog_box.append(&buttons_box);
            dialog_window.set_child(Some(&dialog_box));

            let dialog_window_clone = dialog_window.clone();
            cancel_button.connect_clicked(move |_| {
                dialog_window_clone.close();
            });

            let dialog_window_clone = dialog_window.clone();
            let gui_clone = Rc::clone(&gui);
            let backup_files_clone = backup_files.clone();
            clear_button.connect_clicked(move |_| {
                let mut deleted_count = 0;
                let mut error_message = String::new();

                for file_path in &backup_files_clone {
                    match fs::remove_file(file_path) {
                        Ok(_) => deleted_count += 1,
                        Err(e) => {
                            error_message.push_str(&t!(
                                "gui.failed_to_delete__",
                                file = file_path.display(),
                                error = e
                            ));
                            error_message.push('\n');
                        }
                    }
                }

                dialog_window_clone.close();

                if !error_message.is_empty() {
                    gui_clone.borrow().custom_error_popup(
                        &t!("gui.partial_success"),
                        &format!(
                            "{}.\n{}",
                            &t!(
                                "gui.deleted__of__backup_files",
                                n = deleted_count,
                                all = backup_files_clone.len()
                            ),
                            &t!("gui.errors_", errors = error_message),
                        ),
                    );
                } else {
                    gui_clone.borrow().custom_info_popup(
                        &t!("gui.success"),
                        &t!("gui.successfully_deleted__backup_files", n = deleted_count),
                    );
                }
            });

            dialog_window.present();
        });

        let gui_clone = Rc::clone(&gui);
        gui.borrow().load_config_button.connect_clicked(move |_| {
            let gui = Rc::clone(&gui_clone);

            glib::MainContext::default().spawn_local(async move {
                let dialog = FileDialog::builder()
                    .title(t!("gui.load_hyprviz_config"))
                    .accept_label(t!("gui.open"))
                    .build();
                let window = gui.borrow().window.clone();

                if let Ok(file) = dialog.open_future(Some(&window)).await
                    && let Some(path) = file.path()
                {
                    gui.borrow().load_hyprviz_config(&path);
                }
            });
        });

        let gui_clone = Rc::clone(&gui);
        gui.borrow().save_config_button.connect_clicked(move |_| {
            let gui = Rc::clone(&gui_clone);

            glib::MainContext::default().spawn_local(async move {
                let dialog = FileDialog::builder()
                    .title(t!("gui.save_hyprviz_config"))
                    .initial_name("hyprviz_config.json")
                    .accept_label(t!("gui.save"))
                    .build();
                let window = gui.borrow().window.clone();

                if let Ok(file) = dialog.save_future(Some(&window)).await
                    && let Some(path) = file.path()
                {
                    gui.borrow().save_hyprviz_config(&path);
                }
            });
        });

        let gui_clone = Rc::clone(&gui);
        gui.borrow().copy_button.connect_clicked(move |_| {
            gui_clone.borrow().custom_info_popup(
                "GPL-2.0",
                "This program is free software; you can redistribute it and/or
modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, version 2 of
the License.
This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.
You should have received a copy of the GNU General Public License
along with this program; if not, see
<https://www.gnu.org/licenses/old-licenses/gpl-2.0>.",
            );
        });

        let gui_clone = Rc::clone(&gui);
        gui.borrow()
            .locale_dropdown
            .connect_selected_notify(move |dd| {
                if let Some(selected) = dd.selected_item()
                    && let Some(string_object) = selected.downcast_ref::<StringObject>()
                {
                    let new_locale = string_object.string().to_string();
                    set_locale(&new_locale);

                    gui_clone.borrow_mut().reload_ui();
                    gui_clone.borrow().custom_info_popup(
                        &t!("gui.language_changed"),
                        &t!("gui.language_changed_to_", language = new_locale),
                    )
                } else {
                    gui_clone.borrow().custom_error_popup(
                        &t!("gui.failed_to_get_selected_language"),
                        &t!("gui.failed_to_get_selected_language"),
                    )
                }
            });

        let gui_clone = Rc::clone(&gui);
        gui.borrow()
            .search_entry
            .connect_changed(move |entry| gui_clone.borrow().filter_options(entry.text()));

        let gui_clone = Rc::clone(&gui);
        gui.borrow()
            .save_button
            .connect_clicked(move |_| gui_clone.borrow().save_config_file());

        let gui_clone = Rc::clone(&gui);
        gui.borrow()
            .undo_button
            .connect_clicked(move |_| gui_clone.borrow_mut().undo_changes());
    }

    fn load_hyprviz_config(&self, path: &PathBuf) {
        match fs::read_to_string(path) {
            Ok(content) => {
                if let Ok(config) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    for (key, value) in config {
                        let parts: Vec<&str> = key.split(':').collect();
                        if parts.len() >= 2 {
                            let category = parts[0].to_string();
                            let name = parts[1..].join(":");
                            if let Some(widget) = self.config_widgets.get(&category)
                                && let Some(option_widget) = widget.options.get(&name)
                            {
                                let actual_widget = &option_widget.widget;

                                self.set_widget_value(actual_widget, &value);
                                self.changed_options
                                    .borrow_mut()
                                    .insert((category, name), value.to_string());
                            }
                        }
                    }
                    self.custom_info_popup(
                        &t!("gui.config_loaded"),
                        &t!("gui.hyprviz_configuration_loaded_successfully"),
                    );
                } else {
                    self.custom_error_popup(
                        &t!("gui.invalid_config"),
                        &t!("gui.failed_to_parse_the_configuration_file"),
                    );
                }
            }
            Err(e) => {
                self.custom_error_popup(
                    &t!("gui.loading_failed"),
                    &t!("gui.failed_to_read_the_configuration_file_", error = e),
                );
            }
        }
    }

    fn save_hyprviz_config(&self, path: &Path) {
        let config: HashMap<String, String> = self
            .changed_options
            .borrow()
            .iter()
            .map(|((category, name), value)| (format!("{category}:{name}"), value.clone()))
            .collect();

        match serde_json::to_string_pretty(&config) {
            Ok(json) => match atomic_write(path, &json) {
                Ok(_) => {
                    self.custom_info_popup(
                        &t!("gui.config_saved"),
                        &t!("gui.hyprviz_configuration_saved_successfully"),
                    );
                }
                Err(e) => {
                    self.custom_error_popup(
                        &t!("gui.saving_failed"),
                        &t!("gui.failed_to_write_the_configuration_file_", error = e),
                    );
                }
            },
            Err(e) => {
                self.custom_error_popup(
                    &t!("gui.serialization_failed"),
                    &t!("gui.failed_to_serialize_the_configuration_", error = e),
                );
            }
        }
    }

    fn filter_options(&self, search_text: impl AsRef<str>) {
        let search_text = search_text.as_ref().to_lowercase();

        self.sidebar.set_visible(search_text.is_empty());

        for config_widget in self.config_widgets.values() {
            if search_text.is_empty() {
                config_widget.scrolled_window.set_visible(true);
                if let Some(scrolled) = config_widget.scrolled_window.child()
                    && let Some(container) = scrolled.first_child()
                {
                    let mut child = container.first_child();
                    while let Some(widget) = child {
                        widget.set_visible(true);
                        child = widget.next_sibling();
                    }
                }
            } else {
                let mut has_matches = false;

                if let Some(scrolled) = config_widget.scrolled_window.child()
                    && let Some(container) = scrolled.first_child()
                {
                    let mut child = container.first_child();
                    while let Some(widget) = child {
                        widget.set_visible(false);
                        if let Some(box_widget) = widget.downcast_ref::<gtk::Box>()
                            && let Some(label_box) = box_widget.first_child()
                            && let Some(label) = label_box.first_child()
                            && let Some(label) = label.downcast_ref::<gtk::Label>()
                            && label.text().to_lowercase().contains(&search_text)
                        {
                            has_matches = true;
                            widget.set_visible(true);
                        }

                        child = widget.next_sibling();
                    }
                }

                config_widget.scrolled_window.set_visible(has_matches);
            }
        }
    }

    fn save_config_file(&self) {
        let profile_name = {
            let selected_index = self.profile_dropdown.selected();
            let model = self.profile_dropdown.model().unwrap();

            if let Some(item) = model.item(selected_index)
                && let Some(string_object) = item.downcast_ref::<StringObject>()
            {
                string_object.string().as_str().to_string()
            } else {
                "Default".to_string()
            }
        };
        let path = get_config_path(true, &profile_name);
        let backup_path = path.with_file_name(format!(
            "{}{}",
            path.file_name().unwrap().to_str().unwrap(),
            BACKUP_SUFFIX
        ));

        if !path.exists()
            && let Err(e) = fs::File::create(&path)
        {
            self.custom_error_popup_critical(
                &t!("gui.failed_to_create_hyprviz_config_file"),
                &t!("gui.failed_to_create_hyprviz_config_file_", error = e),
            );
        }

        let config_str = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                self.custom_error_popup_critical(
                    &t!("gui.reading_failed"),
                    &t!("gui.failed_to_read_the_configuration_file_", error = e),
                );
                return;
            }
        };

        let mut parsed_config = parse_config(&config_str);
        let changes = self.changed_options.clone();

        if !changes.borrow().is_empty() {
            if let Err(e) = fs::copy(&path, &backup_path) {
                self.custom_error_popup(
                    &t!("gui.backup_failed"),
                    &t!("gui.failed_to_create_backup_", error = e),
                );
            }

            self.apply_changes(&mut parsed_config);

            let updated_config_str = parsed_config.to_string();
            let temp_path = path.with_extension("tmp");

            let result = atomic_write(&path, &updated_config_str);

            for (category, category_widget) in &self.config_widgets {
                for widget_data in category_widget.options.values() {
                    let widget = &widget_data.widget;

                    if let Some(gtkbox) = widget.downcast_ref::<Box>() {
                        while let Some(child) = gtkbox.first_child() {
                            gtkbox.remove(&child);
                        }

                        category_widget.load_config(
                            &self.window,
                            &parsed_config,
                            &profile_name,
                            category,
                            self.changed_options.clone(),
                        );
                    }
                }
            }
            self.changed_options.clone().borrow_mut().clear();

            match result {
                Ok(()) => {
                    println!("Configuration saved automatically to: {:?}", path);
                    reload_hyprland();
                }
                Err(e) => {
                    let _ = fs::remove_file(&temp_path);
                    self.custom_error_popup(
                        &t!("gui.saving_failed"),
                        &t!(
                            "gui.failed_to_save_the_configuration_automatically_",
                            error = e
                        ),
                    );
                }
            }
        } else {
            self.custom_error_popup(&t!("gui.saving_failed"), &t!("gui.no_changes_to_save"));
        }
    }

    fn undo_changes(&mut self) {
        let profile_name = {
            let selected_index = self.profile_dropdown.selected();
            let model = self.profile_dropdown.model().unwrap();

            if let Some(item) = model.item(selected_index)
                && let Some(string_object) = item.downcast_ref::<StringObject>()
            {
                string_object.string().as_str().to_string()
            } else {
                "Default".to_string()
            }
        };
        let path = get_config_path(true, &profile_name);
        let path_for_read = get_config_path(false, "Default");
        let backup_path = path.with_file_name(format!(
            "{}{}",
            path.file_name().unwrap().to_str().unwrap(),
            BACKUP_SUFFIX
        ));

        if backup_path.exists() {
            match fs::rename(&backup_path, &path) {
                Ok(_) => {
                    println!("{}", &t!("gui.configuration_restored_from_backup"));
                    reload_hyprland();
                    if let Ok(config_str) = expand_source(&path_for_read) {
                        let parsed_config = parse_config(&config_str);

                        self.load_config(&parsed_config, &profile_name);
                        self.changed_options.clone().borrow_mut().clear();
                    } else {
                        self.custom_error_popup(
                            &t!("gui.reload_failed"),
                            &t!("gui.failed_to_reload_the_configuration_after_undo"),
                        );
                    }
                }
                Err(e) => {
                    self.custom_error_popup(
                        &t!("gui.undo_failed"),
                        &t!("gui.failed_to_restore_from_backup_", error = e),
                    );
                }
            }
        } else {
            self.custom_error_popup(
                &t!("gui.undo_failed"),
                &t!("gui.no_backup_file_found_save_changes_to_create_a_backup"),
            );
        }
    }

    fn set_widget_value(&self, widget: &Widget, value: &str) {
        if let Some(spin_button) = widget.downcast_ref::<SpinButton>() {
            if let Ok(float_value) = value.parse::<f64>() {
                spin_button.set_value(float_value);
            }
        } else if let Some(entry) = widget.downcast_ref::<Entry>() {
            entry.set_text(value);
        } else if let Some(switch) = widget.downcast_ref::<Switch>() {
            switch.set_active(value == "true" || value == "1");
        } else if let Some(color_button) = widget.downcast_ref::<ColorDialogButton>() {
            let dummy_config = HyprlandConfig::new();
            if let Some((red, green, blue, alpha)) = dummy_config.parse_color(value) {
                color_button.set_rgba(&gdk::RGBA::new(red, green, blue, alpha));
            }
        } else if let Some(dropdown) = widget.downcast_ref::<DropDown>() {
            let model = dropdown.model().unwrap();
            for i in 0..model.n_items() {
                if let Some(item) = model.item(i)
                    && let Some(string_object) = item.downcast_ref::<gtk::StringObject>()
                    && string_object.string() == value
                {
                    dropdown.set_selected(i);
                    break;
                }
            }
        }
    }

    pub fn custom_info_popup(&self, title: &str, text: &str) {
        let dialog = AlertDialog::builder()
            .message(title)
            .detail(text)
            .buttons(&["OK"][..])
            .modal(true)
            .build();
        dialog.show(Some(&self.window));
    }

    pub fn custom_error_popup(&self, title: &str, text: &str) {
        let dialog = AlertDialog::builder()
            .message(title)
            .detail(text)
            .buttons(&["OK"][..])
            .modal(true)
            .build();
        dialog.show(Some(&self.window));
    }

    pub fn custom_error_popup_critical(&self, title: &str, text: &str) {
        let dialog = AlertDialog::builder()
            .message(title)
            .detail(text)
            .buttons(&["OK"][..])
            .modal(true)
            .build();
        dialog.choose(
            Some(&self.window),
            None::<&gio::Cancellable>,
            move |_res: Result<i32, _>| {
                std::process::exit(1);
            },
        );
    }

    pub fn load_config(&mut self, config: &HyprlandConfig, profile_name: &str) {
        self.config_widgets.clear();
        self.content_box.set_visible(true);

        while let Some(child) = self.stack.first_child() {
            self.stack.remove(&child);
        }

        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }

        self.sidebar = StackSidebar::new();
        self.sidebar.set_stack(&self.stack);
        self.sidebar.set_width_request(200);

        self.content_box.append(&self.sidebar);
        self.content_box.append(&self.stack);

        self.stack.connect_visible_child_notify(move |stack| {
            if let Some(child) = stack.visible_child()
                && let Some(scrolled_window) = child.downcast_ref::<ScrolledWindow>()
            {
                let adj = scrolled_window.vadjustment();
                adj.set_value(adj.lower());
            }
        });

        let mut categories = vec![
            (t!("gui.general").to_string(), "general"),
            (t!("gui.decoration").to_string(), "decoration"),
            (t!("gui.animations_settings").to_string(), "animations"),
            (t!("gui.input").to_string(), "input"),
            (t!("gui.gestures_settings").to_string(), "gestures"),
            (t!("gui.misc").to_string(), "misc"),
            (t!("gui.bind_settings").to_string(), "binds"),
            (t!("gui.group").to_string(), "group"),
            (t!("gui.layouts").to_string(), "layouts"),
            ("XWayland".to_string(), "xwayland"),
            ("OpenGL".to_string(), "opengl"),
            (t!("gui.render").to_string(), "render"),
            (t!("gui.cursor").to_string(), "cursor"),
            (t!("gui.ecosystem").to_string(), "ecosystem"),
            (t!("gui.experimental").to_string(), "experimental"),
            (t!("gui.debug").to_string(), "debug"),
            (t!("gui.monitors").to_string(), "monitor"),
            (t!("gui.workspaces").to_string(), "workspace"),
            (t!("gui.animations").to_string(), "animation"),
            (t!("gui.binds").to_string(), "bind"),
            (t!("gui.gestures").to_string(), "gesture"),
            (t!("gui.window_rules").to_string(), "windowrule"),
            (t!("gui.layer_rules").to_string(), "layerrule"),
            (t!("gui.execs").to_string(), "exec"),
            (t!("gui.envs").to_string(), "env"),
            (t!("gui.all_top_level").to_string(), "top_level"),
            (t!("gui.system_info").to_string(), "systeminfo"),
        ];

        if is_development_mode() {
            categories.push((t!("gui.togtkbox_test").to_string(), "togtkbox_test"));
        }

        for (display_name, category) in &categories {
            let widget = ConfigWidget::new(category, display_name);
            self.stack
                .add_titled(&widget.scrolled_window, Some(category), display_name);
            self.config_widgets.insert(category.to_string(), widget);
        }

        for (_, category) in &categories {
            if let Some(widget) = self.config_widgets.get(*category) {
                widget.load_config(
                    &self.window,
                    config,
                    profile_name,
                    category,
                    self.changed_options.clone(),
                );
            }
        }

        self.changed_options.borrow_mut().clear();
    }

    pub fn apply_changes(&self, config: &mut HyprlandConfig) {
        let changes = self.changed_options.borrow();
        for (category, widget) in &self.config_widgets {
            for (name, widget_data) in &widget.options {
                let widget = &widget_data.widget;

                if widget.downcast_ref::<Box>().is_some() {
                    continue;
                }

                if let Some(value) = changes.get(&(category.to_string(), name.to_string())) {
                    let formatted_value =
                        if let Some(color_button) = widget.downcast_ref::<ColorDialogButton>() {
                            let rgba = color_button.rgba();
                            format!(
                                "rgba({:02X}{:02X}{:02X}{:02X})",
                                (rgba.red() * 255.0) as u8,
                                (rgba.green() * 255.0) as u8,
                                (rgba.blue() * 255.0) as u8,
                                (rgba.alpha() * 255.0) as u8
                            )
                        } else {
                            value.clone()
                        };

                    if !formatted_value.is_empty() {
                        if category == "layouts" {
                            let parts: Vec<&str> = name.split(':').collect();
                            if parts.len() == 2 {
                                config.add_entry(
                                    parts[0],
                                    &format!("{} = {}", parts[1], formatted_value),
                                );
                            }
                        } else if name.contains(':') {
                            let parts: Vec<&str> = name.split(':').collect();
                            if parts.len() == 2 {
                                config.add_entry(
                                    &format!("{}.{}", category, parts[0]),
                                    &format!("{} = {}", parts[1], formatted_value),
                                );
                            }
                        } else {
                            config.add_entry(category, &format!("{name} = {formatted_value}"));
                        }
                    }
                }
            }
        }

        let mut names: HashMap<String, String> = HashMap::new();
        let mut values: HashMap<String, String> = HashMap::new();
        let mut delete: Vec<String> = Vec::new();
        for ((_, raw), new) in changes.iter() {
            if raw.ends_with("_name") {
                let formatted_raw = raw.strip_suffix("_name").unwrap().to_string();
                names.insert(formatted_raw.clone(), new.clone());
            }

            if raw.ends_with("_value") {
                let formatted_raw = raw.strip_suffix("_value").unwrap().to_string();
                values.insert(formatted_raw.clone(), new.clone());
            }

            if raw.ends_with("_delete") {
                let formatted_raw = raw.strip_suffix("_delete").unwrap().to_string();
                delete.push(formatted_raw.clone());
            }
        }

        let mut lines: Vec<String> = config.to_string().lines().map(String::from).collect();

        let mut delete_tracker = HashMap::new();

        for raw in &delete {
            *delete_tracker.entry(raw.clone()).or_insert(0) += 1;
        }

        lines.retain(|line| {
            let normalized_line = line.trim_start().to_string();

            if let Some(count) = delete_tracker.get_mut(&normalized_line)
                && *count > 0
            {
                *count -= 1;
                return false;
            }

            true
        });

        let mut changed: Vec<String> = Vec::new();

        for ((_, name), _) in changes.iter() {
            if name.ends_with("_name") || name.ends_with("_value") {
                let key = name
                    .strip_suffix("_name")
                    .or_else(|| name.strip_suffix("_value"))
                    .unwrap_or(name);
                if changed.contains(&key.to_string()) {
                    continue;
                }

                let has_name = names.contains_key(key);
                let has_value = values.contains_key(key);

                let mut new_name = names.get(key).cloned().unwrap_or_default();
                let mut new_value = values.get(key).cloned().unwrap_or_default();

                let mut found = false;
                for line in &mut lines {
                    if line.trim_start().starts_with(key)
                        && let Some((original_name, original_value)) = line.split_once('=')
                    {
                        let indent = line
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .collect::<String>();

                        if !has_name {
                            new_name = original_name.trim().to_string();
                        }
                        if !has_value {
                            new_value = original_value.trim_start().to_string();
                        }

                        *line = format!("{}{} = {}", indent, new_name, new_value);
                        found = true;
                        break;
                    }
                }

                if !found {
                    lines.push(format!("{} = {}", new_name, new_value));
                }
                changed.push(key.to_string());
            }
        }

        let new_config = parse_config(&lines.join("\n"));
        *config = new_config;
    }

    fn reload_ui(&mut self) {
        let current_profile = {
            let selected_index = self.profile_dropdown.selected();
            let model = self.profile_dropdown.model().unwrap();

            if let Some(item) = model.item(selected_index)
                && let Some(string_object) = item.downcast_ref::<StringObject>()
            {
                string_object.string().as_str().to_string()
            } else {
                "Default".to_string()
            }
        };

        let current_page = self.stack.visible_child_name().map(|n| n.to_string());

        let path = get_config_path(false, "Default");
        let config_str = match expand_source(&path) {
            Ok(config) => config,
            Err(e) => {
                self.custom_error_popup(
                    &t!("gui.reading_failed"),
                    &t!("gui.failed_to_read_the_configuration_file_", error = e),
                );
                return;
            }
        };

        let parsed_config = parse_config(&config_str);
        self.load_config(&parsed_config, &current_profile);

        if let Some(page) = current_page
            && let Some(child) = self.stack.child_by_name(&page)
        {
            self.stack.set_visible_child(&child);
        }

        self.title_label
            .set_label(&t!("gui.hyprland_configuration"));
        self.current_profile_label.set_label(&t!("gui.profile"));
        self.undo_button.set_label(&t!("gui.undo"));
        self.save_button.set_label(&t!("gui.save"));

        self.create_profile_button
            .set_label(&t!("gui.create_profile"));
        self.delete_profile_button
            .set_label(&t!("gui.delete_profile"));
        self.clear_backups_button
            .set_label(&t!("gui.clear_backups_files"));
        self.load_config_button
            .set_label(&t!("gui.load_hyprviz_config"));
        self.save_config_button
            .set_label(&t!("gui.save_hyprviz_config"));
        self.copy_button.set_label(&t!("gui.copyright"));
    }
}
