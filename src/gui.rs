use crate::utils::{BACKUP_SUFFIX, CONFIG_PATH, get_config_path, reload_hyprland};
use crate::widget::ConfigWidget;
use gtk::{
    AlertDialog, Application, ApplicationWindow, Box, Button, ColorDialogButton, DropDown, Entry,
    FileDialog, HeaderBar, Orientation, Popover, ScrolledWindow, SearchEntry, SpinButton, Stack,
    StackSidebar, Switch, Widget, gdk, glib, prelude::*,
};
use hyprparser::{HyprlandConfig, parse_config};
use std::{cell::RefCell, collections::HashMap, fs, path::PathBuf, rc::Rc};

pub struct ConfigGUI {
    pub window: ApplicationWindow,
    config_widgets: HashMap<String, ConfigWidget>,
    save_button: Button,
    undo_button: Button,
    save_config_button: Button,
    load_config_button: Button,
    copy_button: Button,
    search_entry: SearchEntry,
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

        let header_bar = HeaderBar::builder()
            .show_title_buttons(false)
            .title_widget(&gtk::Label::new(Some("Hyprland Configuration")))
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

        let save_config_button = Button::with_label("Save HyprViz Config");
        let load_config_button = Button::with_label("Load HyprViz Config");
        let copy_button = Button::with_label("Copyright");

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

        let save_button = Button::with_label("Save");
        let undo_button = Button::with_label("Undo");

        header_bar.pack_end(&save_button);
        header_bar.pack_end(&undo_button);

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
            save_button,
            undo_button,
            save_config_button,
            load_config_button,
            copy_button,
            search_entry,
            content_box,
            changed_options: Rc::new(RefCell::new(HashMap::new())),
            stack,
            sidebar,
        }
    }

    pub fn setup_ui_events(gui: Rc<RefCell<ConfigGUI>>) {
        let gui_clone = Rc::clone(&gui);
        gui.borrow().load_config_button.connect_clicked(move |_| {
            let gui = Rc::clone(&gui_clone);

            glib::MainContext::default().spawn_local(async move {
                let dialog = FileDialog::builder()
                    .title("Load HyprViz Config")
                    .accept_label("Open")
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
                    .title("Save HyprViz Config")
                    .initial_name("hyprviz_config.json")
                    .accept_label("Save")
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
                        "Config Loaded",
                        "HyprViz configuration loaded successfully.",
                    );
                } else {
                    self.custom_error_popup(
                        "Invalid Config",
                        "Failed to parse the configuration file.",
                    );
                }
            }
            Err(e) => {
                self.custom_error_popup(
                    "Loading Failed",
                    &format!("Failed to read the configuration file: {e}"),
                );
            }
        }
    }

    fn save_hyprviz_config(&self, path: &PathBuf) {
        let config: HashMap<String, String> = self
            .changed_options
            .borrow()
            .iter()
            .map(|((category, name), value)| (format!("{category}:{name}"), value.clone()))
            .collect();

        match serde_json::to_string_pretty(&config) {
            Ok(json) => match fs::write(path, json) {
                Ok(_) => {
                    self.custom_info_popup(
                        "Config Saved",
                        "HyprViz configuration saved successfully.",
                    );
                }
                Err(e) => {
                    self.custom_error_popup(
                        "Saving Failed",
                        &format!("Failed to write the configuration file: {e}"),
                    );
                }
            },
            Err(e) => {
                self.custom_error_popup(
                    "Serialization Failed",
                    &format!("Failed to serialize the configuration: {e}"),
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
        let path = get_config_path();
        let backup_path = path.with_file_name(format!(
            "{}{}",
            path.file_name().unwrap().to_str().unwrap(),
            BACKUP_SUFFIX
        ));

        let config_str = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                self.custom_error_popup_critical(
                    "Reading failed",
                    &format!("Failed to read the configuration file: {e}"),
                );
                return;
            }
        };

        let mut parsed_config = parse_config(&config_str);
        let changes = self.changed_options.clone();

        if !changes.borrow().is_empty() {
            if !backup_path.exists()
                && let Err(e) = fs::copy(&path, &backup_path)
            {
                self.custom_error_popup("Backup failed", &format!("Failed to create backup: {e}"));
                return;
            }

            self.apply_changes(&mut parsed_config);

            let updated_config_str = parsed_config.to_string();

            match fs::write(&path, updated_config_str) {
                Ok(_) => {
                    println!("Configuration saved to: ~/{CONFIG_PATH}");
                    reload_hyprland();
                }
                Err(e) => {
                    self.custom_error_popup(
                        "Saving failed",
                        &format!("Failed to save the configuration: {e}"),
                    );
                }
            }
        } else {
            self.custom_error_popup("Saving failed", "No changes to save.");
        }
    }

    fn undo_changes(&mut self) {
        let path = get_config_path();
        let backup_path = path.with_file_name(format!(
            "{}{}",
            path.file_name().unwrap().to_str().unwrap(),
            BACKUP_SUFFIX
        ));

        if backup_path.exists() {
            match fs::copy(&backup_path, &path) {
                Ok(_) => {
                    println!("Configuration restored from backup");
                    if let Ok(config_str) = fs::read_to_string(&path) {
                        let parsed_config = parse_config(&config_str);

                        self.load_config(&parsed_config);
                        self.changed_options.clone().borrow_mut().clear();

                        if let Err(e) = fs::remove_file(&backup_path) {
                            self.custom_error_popup(
                                "Backup Deletion Failed",
                                &format!("Failed to delete the backup file: {e}"),
                            );
                        } else {
                            reload_hyprland();
                        }
                    } else {
                        self.custom_error_popup(
                            "Reload Failed",
                            "Failed to reload the configuration after undo.",
                        );
                    }
                }
                Err(e) => {
                    self.custom_error_popup(
                        "Undo Failed",
                        &format!("Failed to restore from backup: {e}"),
                    );
                }
            }
        } else {
            self.custom_error_popup(
                "Undo Failed",
                "No backup file found. Save changes at least once to create a backup.",
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

    pub fn load_config(&mut self, config: &HyprlandConfig) {
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

        let categories = [
            ("General", "general"),
            ("Decoration", "decoration"),
            ("Animations", "animations"),
            ("Input", "input"),
            ("Gestures", "gestures"),
            ("Misc", "misc"),
            ("Bind Settings", "binds"),
            ("Group", "group"),
            ("Layouts", "layouts"),
            ("XWayland", "xwayland"),
            ("OpenGL", "opengl"),
            ("Render", "render"),
            ("Cursor", "cursor"),
            ("Ecosystem", "ecosystem"),
            ("Experimental", "experimental"),
            ("Debug", "debug"),
        ];

        for (display_name, category) in &categories {
            let widget = ConfigWidget::new(category);
            self.stack
                .add_titled(&widget.scrolled_window, Some(category), display_name);
            self.config_widgets.insert(category.to_string(), widget);
        }

        for (_, category) in &categories {
            if let Some(widget) = self.config_widgets.get(*category) {
                widget.load_config(config, category, self.changed_options.clone());
            }
        }

        self.changed_options.borrow_mut().clear();
    }

    pub fn apply_changes(&self, config: &mut HyprlandConfig) {
        let changes = self.changed_options.borrow();
        for (category, widget) in &self.config_widgets {
            for (name, widget_data) in &widget.options {
                let widget = &widget_data.widget;
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
    }
}
