use gtk::{
    ApplicationWindow, Box, Button, ColorDialog, ColorDialogButton, DropDown, Entry, Expander,
    Frame, Grid, Justification, Label, Orientation, Popover, ScrolledWindow, SpinButton,
    StringList, StringObject, Switch, Widget, gdk, glib, prelude::*,
};
use hyprparser::HyprlandConfig;
use rust_i18n::t;
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    fs,
    rc::Rc,
};

use crate::{
    advanced_editors::{create_bind_editor, create_curve_editor},
    guides::create_guide,
    utils::{
        MAX_SAFE_INTEGER_F64, compare_versions, expand_source, expand_source_str, get_config_path,
        get_latest_version, parse_top_level_options,
    },
};

use crate::system_info::*;

pub struct WidgetData {
    pub widget: Widget,
    pub default: String,
}
pub struct ConfigWidget {
    pub options: HashMap<String, WidgetData>,
    pub scrolled_window: ScrolledWindow,
}

fn add_section(container: &Box, title: &str, description: &str, first_section: Rc<RefCell<bool>>) {
    let section_box = Box::new(Orientation::Vertical, 5);
    section_box.set_margin_top(15);
    section_box.set_margin_bottom(10);

    let title_label = Label::new(Some(title));
    let desc_label = Label::new(Some(description));
    desc_label.set_wrap(true);

    if *first_section.borrow() {
        title_label.set_halign(gtk::Align::Center);
        desc_label.set_halign(gtk::Align::Center);
        title_label.set_hexpand(true);
        desc_label.set_hexpand(true);
        *first_section.borrow_mut() = false;
    } else {
        title_label.set_halign(gtk::Align::Start);
        desc_label.set_halign(gtk::Align::Start);
    }

    title_label.set_markup(&format!("<b>{title}</b>"));
    section_box.append(&title_label);

    desc_label.set_opacity(0.7);
    section_box.append(&desc_label);

    let frame = Frame::new(None);
    frame.set_margin_top(10);
    section_box.append(&frame);

    container.append(&section_box);
}

pub fn add_info_row(container: &Box, label: &str, value: &str) -> (Label, Button) {
    let row = Box::new(Orientation::Horizontal, 10);
    row.set_margin_start(10);
    row.set_margin_end(10);
    row.set_margin_top(5);
    row.set_margin_bottom(5);

    let label_widget = Label::new(Some(label));
    label_widget.set_xalign(0.0);
    label_widget.add_css_class("heading");
    label_widget.set_hexpand(false);

    let value_widget = Label::new(Some(value));
    value_widget.set_xalign(0.0);
    value_widget.set_selectable(true);
    value_widget.set_hexpand(true);
    value_widget.set_wrap(true);

    let refresh_button = Button::from_icon_name("view-refresh-symbolic");
    if label.to_lowercase().contains("version") {
        refresh_button.set_tooltip_text(Some(&t!("check_if_there_is_a_new_version_available")));
    } else {
        refresh_button.set_tooltip_text(Some(&t!("refresh")));
    }
    refresh_button.set_valign(gtk::Align::Center);
    refresh_button.set_has_frame(false);

    row.append(&label_widget);
    row.append(&value_widget);
    row.append(&refresh_button);
    container.append(&row);

    (value_widget, refresh_button)
}

fn add_dropdown_option(
    container: &Box,
    options: &mut HashMap<String, WidgetData>,
    name: &str,
    label: &str,
    description: &str,
    items: &[&str],
    default: &str,
) {
    let hbox = Box::new(Orientation::Horizontal, 10);
    hbox.set_margin_start(10);
    hbox.set_margin_end(10);
    hbox.set_margin_top(5);
    hbox.set_margin_bottom(5);

    let label_box = Box::new(Orientation::Horizontal, 5);
    label_box.set_hexpand(true);

    let label_widget = Label::new(None);
    label_widget.set_halign(gtk::Align::Start);
    let formatted_text = format!(
        "{}\n<span foreground=\"gray\">({})</span>",
        glib::markup_escape_text(label),
        glib::markup_escape_text(name)
    );
    label_widget.set_markup(&formatted_text);
    label_widget.set_use_markup(true);

    let popover = Popover::new();
    let description_label = Label::new(Some(description));
    description_label.set_wrap(true);
    description_label.set_width_chars(40);
    description_label.set_max_width_chars(60);
    description_label.set_justify(Justification::Fill);
    description_label.set_xalign(0.0);
    description_label.set_margin_top(5);
    description_label.set_margin_bottom(5);
    description_label.set_margin_start(5);
    description_label.set_margin_end(5);
    popover.set_child(Some(&description_label));
    popover.set_position(gtk::PositionType::Right);

    let tooltip_button = Button::from_icon_name("dialog-question-symbolic");
    tooltip_button.set_has_frame(false);
    tooltip_button.connect_clicked(move |button| {
        popover.set_parent(button);
        popover.popup();
    });

    label_box.append(&label_widget);
    label_box.append(&tooltip_button);

    let string_list = StringList::new(items);
    let dropdown = DropDown::new(Some(string_list.clone()), None::<gtk::Expression>);
    dropdown.set_halign(gtk::Align::End);
    dropdown.set_width_request(100);

    let reset_button = Button::from_icon_name("view-refresh-symbolic");
    reset_button.set_tooltip_text(Some(&t!("reset_to_default")));
    reset_button.set_valign(gtk::Align::Center);
    reset_button.set_has_frame(false);

    let dropdown_clone = dropdown.clone();
    let parsed_default: String = default
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse the default value for '{}'", name));

    reset_button.connect_clicked(move |_| {
        for idx in 0..string_list.n_items() {
            if let Some(item) = string_list.item(idx) {
                let item_str = item.property::<String>("string");

                if item_str == parsed_default {
                    dropdown_clone.set_selected(idx);
                    break;
                }
            }
        }
    });

    hbox.append(&label_box);
    hbox.append(&dropdown);
    hbox.append(&reset_button);

    container.append(&hbox);

    options.insert(
        name.to_string(),
        WidgetData {
            widget: dropdown.upcast(),
            default: default.to_string(),
        },
    );
}

fn add_bool_option(
    container: &Box,
    options: &mut HashMap<String, WidgetData>,
    name: &str,
    label: &str,
    description: &str,
    default: &str,
) {
    let hbox = Box::new(Orientation::Horizontal, 10);
    hbox.set_margin_start(10);
    hbox.set_margin_end(10);
    hbox.set_margin_top(5);
    hbox.set_margin_bottom(5);

    let label_box = Box::new(Orientation::Horizontal, 5);
    label_box.set_hexpand(true);

    let label_widget = Label::new(None);
    label_widget.set_halign(gtk::Align::Start);
    let formatted_text = format!(
        "{}\n<span foreground=\"gray\">({})</span>",
        glib::markup_escape_text(label),
        glib::markup_escape_text(name)
    );
    label_widget.set_markup(&formatted_text);
    label_widget.set_use_markup(true);

    let popover = Popover::new();
    let description_label = Label::new(Some(description));
    description_label.set_wrap(true);
    description_label.set_width_chars(40);
    description_label.set_max_width_chars(60);
    description_label.set_justify(Justification::Fill);
    description_label.set_xalign(0.0);
    description_label.set_margin_top(5);
    description_label.set_margin_bottom(5);
    description_label.set_margin_start(5);
    description_label.set_margin_end(5);
    popover.set_child(Some(&description_label));
    popover.set_position(gtk::PositionType::Right);

    let tooltip_button = Button::from_icon_name("dialog-question-symbolic");
    tooltip_button.set_has_frame(false);
    tooltip_button.connect_clicked(move |button| {
        popover.set_parent(button);
        popover.popup();
    });

    label_box.append(&label_widget);
    label_box.append(&tooltip_button);

    let switch = Switch::new();
    switch.set_halign(gtk::Align::End);
    switch.set_valign(gtk::Align::Center);

    let reset_button = Button::from_icon_name("view-refresh-symbolic");
    reset_button.set_tooltip_text(Some(&t!("reset_to_default")));
    reset_button.set_valign(gtk::Align::Center);
    reset_button.set_has_frame(false);

    let switch_clone = switch.clone();
    let parsed_default: bool = default
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse the default value for '{}'", name));

    reset_button.connect_clicked(move |_| {
        switch_clone.set_active(parsed_default);
    });

    hbox.append(&label_box);
    hbox.append(&switch);
    hbox.append(&reset_button);

    container.append(&hbox);

    options.insert(
        name.to_string(),
        WidgetData {
            widget: switch.upcast(),
            default: default.to_string(),
        },
    );
}

fn add_bool_int_option(
    container: &Box,
    options: &mut HashMap<String, WidgetData>,
    name: &str,
    label: &str,
    description: &str,
    default: &str,
) {
    let hbox = Box::new(Orientation::Horizontal, 10);
    hbox.set_margin_start(10);
    hbox.set_margin_end(10);
    hbox.set_margin_top(5);
    hbox.set_margin_bottom(5);

    let label_box = Box::new(Orientation::Horizontal, 5);
    label_box.set_hexpand(true);

    let label_widget = Label::new(None);
    label_widget.set_halign(gtk::Align::Start);
    let formatted_text = format!(
        "{}\n<span foreground=\"gray\">({})</span>",
        glib::markup_escape_text(label),
        glib::markup_escape_text(name)
    );
    label_widget.set_markup(&formatted_text);
    label_widget.set_use_markup(true);

    let popover = Popover::new();
    let description_label = Label::new(Some(description));
    description_label.set_wrap(true);
    description_label.set_width_chars(40);
    description_label.set_max_width_chars(60);
    description_label.set_justify(Justification::Fill);
    description_label.set_xalign(0.0);
    description_label.set_margin_top(5);
    description_label.set_margin_bottom(5);
    description_label.set_margin_start(5);
    description_label.set_margin_end(5);
    popover.set_child(Some(&description_label));
    popover.set_position(gtk::PositionType::Right);

    let tooltip_button = Button::from_icon_name("dialog-question-symbolic");
    tooltip_button.set_has_frame(false);
    tooltip_button.connect_clicked(move |button| {
        popover.set_parent(button);
        popover.popup();
    });

    label_box.append(&label_widget);
    label_box.append(&tooltip_button);

    let switch = Switch::new();
    switch.set_halign(gtk::Align::End);
    switch.set_valign(gtk::Align::Center);

    let reset_button = Button::from_icon_name("view-refresh-symbolic");
    reset_button.set_tooltip_text(Some(&t!("reset_to_default")));
    reset_button.set_valign(gtk::Align::Center);
    reset_button.set_has_frame(false);

    let parsed_default: i32 = default
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse the default value for '{}'", name));

    switch.set_active(parsed_default == 1);

    let switch_clone = switch.clone();
    reset_button.connect_clicked(move |_| {
        switch_clone.set_active(parsed_default == 1);
    });

    hbox.append(&label_box);
    hbox.append(&switch);
    hbox.append(&reset_button);

    container.append(&hbox);

    options.insert(
        name.to_string(),
        WidgetData {
            widget: switch.upcast(),
            default: default.to_string(),
        },
    );
}

fn add_int_option(
    container: &Box,
    options: &mut HashMap<String, WidgetData>,
    name: &str,
    label: &str,
    description: &str,
    default: &str,
    range: (f64, f64, f64),
) {
    let hbox = Box::new(Orientation::Horizontal, 10);
    hbox.set_margin_start(10);
    hbox.set_margin_end(10);
    hbox.set_margin_top(5);
    hbox.set_margin_bottom(5);

    let label_box = Box::new(Orientation::Horizontal, 5);
    label_box.set_hexpand(true);

    let label_widget = Label::new(None);
    label_widget.set_halign(gtk::Align::Start);
    let formatted_text = format!(
        "{}\n<span foreground=\"gray\">({})</span>",
        glib::markup_escape_text(label),
        glib::markup_escape_text(name)
    );
    label_widget.set_markup(&formatted_text);
    label_widget.set_use_markup(true);

    let popover = Popover::new();
    let description_label = Label::new(Some(description));
    description_label.set_wrap(true);
    description_label.set_width_chars(40);
    description_label.set_max_width_chars(60);
    description_label.set_justify(Justification::Fill);
    description_label.set_xalign(0.0);
    description_label.set_margin_top(5);
    description_label.set_margin_bottom(5);
    description_label.set_margin_start(5);
    description_label.set_margin_end(5);
    popover.set_child(Some(&description_label));
    popover.set_position(gtk::PositionType::Right);

    let tooltip_button = Button::from_icon_name("dialog-question-symbolic");
    tooltip_button.set_has_frame(false);
    tooltip_button.connect_clicked(move |button| {
        popover.set_parent(button);
        popover.popup();
    });

    label_box.append(&label_widget);
    label_box.append(&tooltip_button);

    let (min, max, step) = range;
    let spin_button = SpinButton::with_range(min, max, step);
    spin_button.set_digits(0);
    spin_button.set_halign(gtk::Align::End);
    spin_button.set_width_request(100);

    let reset_button = Button::from_icon_name("view-refresh-symbolic");
    reset_button.set_tooltip_text(Some(&t!("reset_to_default")));
    reset_button.set_valign(gtk::Align::Center);
    reset_button.set_has_frame(false);

    let spin_clone = spin_button.clone();
    let parsed_default: f64 = default
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse the default value for '{}'", name));

    reset_button.connect_clicked(move |_| {
        spin_clone.set_value(parsed_default);
    });

    hbox.append(&label_box);
    hbox.append(&spin_button);
    hbox.append(&reset_button);

    container.append(&hbox);

    options.insert(
        name.to_string(),
        WidgetData {
            widget: spin_button.upcast(),
            default: default.to_string(),
        },
    );
}

fn add_float_option(
    container: &Box,
    options: &mut HashMap<String, WidgetData>,
    name: &str,
    label: &str,
    description: &str,
    default: &str,
    range: (f64, f64, f64),
) {
    let hbox = Box::new(Orientation::Horizontal, 10);
    hbox.set_margin_start(10);
    hbox.set_margin_end(10);
    hbox.set_margin_top(5);
    hbox.set_margin_bottom(5);

    let label_box = Box::new(Orientation::Horizontal, 5);
    label_box.set_hexpand(true);

    let label_widget = Label::new(None);
    label_widget.set_halign(gtk::Align::Start);
    let formatted_text = format!(
        "{}\n<span foreground=\"gray\">({})</span>",
        glib::markup_escape_text(label),
        glib::markup_escape_text(name)
    );
    label_widget.set_markup(&formatted_text);
    label_widget.set_use_markup(true);

    let popover = Popover::new();
    let description_label = Label::new(Some(description));
    description_label.set_wrap(true);
    description_label.set_width_chars(40);
    description_label.set_max_width_chars(60);
    description_label.set_justify(Justification::Fill);
    description_label.set_xalign(0.0);
    description_label.set_margin_top(5);
    description_label.set_margin_bottom(5);
    description_label.set_margin_start(5);
    description_label.set_margin_end(5);
    popover.set_child(Some(&description_label));
    popover.set_position(gtk::PositionType::Right);

    let tooltip_button = Button::from_icon_name("dialog-question-symbolic");
    tooltip_button.set_has_frame(false);
    tooltip_button.connect_clicked(move |button| {
        popover.set_parent(button);
        popover.popup();
    });

    label_box.append(&label_widget);
    label_box.append(&tooltip_button);

    let (min, max, step) = range;
    let spin_button = SpinButton::with_range(min, max, step);
    spin_button.set_digits(2);
    spin_button.set_halign(gtk::Align::End);
    spin_button.set_width_request(100);

    let reset_button = Button::from_icon_name("view-refresh-symbolic");
    reset_button.set_tooltip_text(Some(&t!("reset_to_default")));
    reset_button.set_valign(gtk::Align::Center);
    reset_button.set_has_frame(false);

    let spin_clone = spin_button.clone();
    let parsed_default: f64 = default
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse the default value for '{}'", name));

    reset_button.connect_clicked(move |_| {
        spin_clone.set_value(parsed_default);
    });

    hbox.append(&label_box);
    hbox.append(&spin_button);
    hbox.append(&reset_button);

    container.append(&hbox);

    options.insert(
        name.to_string(),
        WidgetData {
            widget: spin_button.upcast(),
            default: default.to_string(),
        },
    );
}

fn add_string_option(
    container: &Box,
    options: &mut HashMap<String, WidgetData>,
    name: &str,
    label: &str,
    description: &str,
    default: &str,
) {
    let hbox = Box::new(Orientation::Horizontal, 10);
    hbox.set_margin_start(10);
    hbox.set_margin_end(10);
    hbox.set_margin_top(5);
    hbox.set_margin_bottom(5);

    let label_box = Box::new(Orientation::Horizontal, 5);
    label_box.set_hexpand(true);

    let label_widget = Label::new(None);
    label_widget.set_halign(gtk::Align::Start);
    let formatted_text = format!(
        "{}\n<span foreground=\"gray\">({})</span>",
        glib::markup_escape_text(label),
        glib::markup_escape_text(name)
    );
    label_widget.set_markup(&formatted_text);
    label_widget.set_use_markup(true);

    let popover = Popover::new();
    let description_label = Label::new(Some(description));
    description_label.set_wrap(true);
    description_label.set_width_chars(40);
    description_label.set_max_width_chars(60);
    description_label.set_justify(Justification::Fill);
    description_label.set_xalign(0.0);
    description_label.set_margin_top(5);
    description_label.set_margin_bottom(5);
    description_label.set_margin_start(5);
    description_label.set_margin_end(5);
    popover.set_child(Some(&description_label));
    popover.set_position(gtk::PositionType::Right);

    let tooltip_button = Button::from_icon_name("dialog-question-symbolic");
    tooltip_button.set_has_frame(false);
    tooltip_button.connect_clicked(move |button| {
        popover.set_parent(button);
        popover.popup();
    });

    label_box.append(&label_widget);
    label_box.append(&tooltip_button);

    let entry = Entry::new();
    entry.set_halign(gtk::Align::End);
    entry.set_width_request(100);

    let reset_button = Button::from_icon_name("view-refresh-symbolic");
    reset_button.set_tooltip_text(Some(&t!("reset_to_default")));
    reset_button.set_valign(gtk::Align::Center);
    reset_button.set_has_frame(false);

    let entry_clone = entry.clone();
    let parsed_default: String = default
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse the default value for '{}'", name));

    reset_button.connect_clicked(move |_| {
        entry_clone.set_text(&parsed_default);
    });

    hbox.append(&label_box);
    hbox.append(&entry);
    hbox.append(&reset_button);

    container.append(&hbox);

    options.insert(
        name.to_string(),
        WidgetData {
            widget: entry.upcast(),
            default: default.to_string(),
        },
    );
}

fn add_color_option(
    container: &Box,
    options: &mut HashMap<String, WidgetData>,
    name: &str,
    label: &str,
    description: &str,
    default: &str,
) {
    let hbox = Box::new(Orientation::Horizontal, 10);
    hbox.set_margin_start(10);
    hbox.set_margin_end(10);
    hbox.set_margin_top(5);
    hbox.set_margin_bottom(5);

    let label_box = Box::new(Orientation::Horizontal, 5);
    label_box.set_hexpand(true);

    let label_widget = Label::new(None);
    label_widget.set_halign(gtk::Align::Start);
    let formatted_text = format!(
        "{}\n<span foreground=\"gray\">({})</span>",
        glib::markup_escape_text(label),
        glib::markup_escape_text(name)
    );
    label_widget.set_markup(&formatted_text);
    label_widget.set_use_markup(true);

    let popover = Popover::new();
    let description_label = Label::new(Some(description));
    description_label.set_wrap(true);
    description_label.set_width_chars(40);
    description_label.set_max_width_chars(60);
    description_label.set_justify(Justification::Fill);
    description_label.set_xalign(0.0);
    description_label.set_margin_top(5);
    description_label.set_margin_bottom(5);
    description_label.set_margin_start(5);
    description_label.set_margin_end(5);
    popover.set_child(Some(&description_label));
    popover.set_position(gtk::PositionType::Right);

    let tooltip_button = Button::from_icon_name("dialog-question-symbolic");
    tooltip_button.set_has_frame(false);
    tooltip_button.connect_clicked(move |button| {
        popover.set_parent(button);
        popover.popup();
    });

    label_box.append(&label_widget);
    label_box.append(&tooltip_button);

    let color_dialog = ColorDialog::new();
    color_dialog.set_with_alpha(true);
    let color_button = ColorDialogButton::new(Some(color_dialog));
    color_button.set_halign(gtk::Align::End);

    let entry = Entry::new();
    entry.set_max_length(9);
    entry.set_width_chars(9);
    entry.set_placeholder_text(Some("#RRGGBBAA"));
    entry.set_halign(gtk::Align::End);

    {
        let rgba = color_button.rgba();
        let r = (rgba.red() * 255.0).round() as u8;
        let g = (rgba.green() * 255.0).round() as u8;
        let b = (rgba.blue() * 255.0).round() as u8;
        let a = (rgba.alpha() * 255.0).round() as u8;
        let hex = format!("#{r:02X}{g:02X}{b:02X}{a:02X}");
        entry.set_text(&hex);
    }

    let reset_button = Button::from_icon_name("view-refresh-symbolic");
    reset_button.set_tooltip_text(Some(&t!("reset_to_default")));
    reset_button.set_valign(gtk::Align::Center);
    reset_button.set_has_frame(false);

    let entry_clone = entry.clone();
    let parsed_default: String = default
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse the default value for '{}'", name));

    reset_button.connect_clicked(move |_| {
        entry_clone.set_text(&parsed_default);
    });

    hbox.append(&label_box);
    hbox.append(&color_button);
    hbox.append(&entry);
    hbox.append(&reset_button);

    container.append(&hbox);

    color_button.connect_rgba_notify(glib::clone!(
        #[weak]
        entry,
        move |btn| {
            let rgba = btn.rgba();
            let r = (rgba.red() * 255.0).round() as u8;
            let g = (rgba.green() * 255.0).round() as u8;
            let b = (rgba.blue() * 255.0).round() as u8;
            let a = (rgba.alpha() * 255.0).round() as u8;
            let hex = format!("#{r:02X}{g:02X}{b:02X}{a:02X}");
            entry.set_text(&hex);
        }
    ));

    entry.connect_changed(glib::clone!(
        #[weak]
        color_button,
        #[weak]
        entry,
        move |e| {
            let text = e.text().trim().to_string();

            if text.len() == 9
                && text.starts_with("#")
                && let Ok(color) = gtk::gdk::RGBA::parse(&text)
            {
                color_button.set_rgba(&color);
                entry.set_css_classes(&[]);
            } else {
                entry.set_css_classes(&["error"]);
            }
        }
    ));

    options.insert(
        name.to_string(),
        WidgetData {
            widget: color_button.upcast(),
            default: default.to_string(),
        },
    );
}

fn append_option_row(
    window: &ApplicationWindow,
    gtkbox: &Box,
    raw: String,
    name: String,
    value: String,
    changed_options: &Rc<RefCell<HashMap<(String, String), String>>>,
    category: &str,
) {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_top(5);
    vbox.set_margin_bottom(5);
    vbox.set_margin_start(5);
    vbox.set_margin_end(5);

    let boxline = Box::new(Orientation::Horizontal, 5);
    boxline.set_hexpand(true);
    boxline.set_margin_top(5);
    boxline.set_margin_bottom(5);
    boxline.set_margin_start(5);
    boxline.set_margin_end(5);

    let value_entry = Entry::new();
    let (editor_box, show_button) = match category {
        "animation" => create_curve_editor(&value_entry),
        "bind" => create_bind_editor(window, &value_entry),
        _ => (Box::new(Orientation::Vertical, 5), Button::new()),
    };

    if category == "animation" {
        show_button.set_visible(false)
    }

    let name_entry = Entry::new();
    name_entry.set_text(&name);
    name_entry.set_margin_top(5);
    name_entry.set_margin_bottom(5);
    name_entry.set_margin_start(5);
    name_entry.set_margin_end(5);

    let changed_options_clone = changed_options.clone();
    let raw_clone = raw.clone();
    let editor_box_clone = editor_box.clone();
    let show_button_clone = show_button.clone();
    let category_str = category.to_string();
    name_entry.connect_changed(move |entry| {
        let mut changes = changed_options_clone.borrow_mut();
        let new_name = entry.text().to_string();
        changes.insert(
            (category_str.clone(), format!("{}_name", raw_clone)),
            new_name.clone(),
        );

        if (category_str == "animation" && new_name == "bezier") || (category_str == "bind") {
            show_button_clone.set_visible(true);
        } else {
            show_button_clone.set_visible(false);
            editor_box_clone.set_visible(false);
        }
    });

    boxline.append(&name_entry);

    let equals_label = Label::new(Some("="));
    equals_label.set_xalign(0.5);
    boxline.append(&equals_label);

    value_entry.set_text(&value);
    value_entry.set_margin_top(5);
    value_entry.set_margin_bottom(5);
    value_entry.set_margin_start(5);
    value_entry.set_margin_end(5);
    value_entry.set_hexpand(true);

    let changed_options_clone = changed_options.clone();
    let raw_clone = raw.clone();
    let category_str = category.to_string();

    value_entry.connect_changed(move |entry| {
        let mut changes = changed_options_clone.borrow_mut();
        let new_value = entry.text().to_string();
        changes.insert(
            (category_str.clone(), format!("{}_value", raw_clone)),
            new_value,
        );
    });

    boxline.append(&value_entry);

    boxline.append(&show_button);

    let delete_button = Button::from_icon_name("edit-delete-symbolic");
    delete_button.set_tooltip_text(Some(&t!("delete_this_option")));
    delete_button.set_valign(gtk::Align::Center);
    delete_button.set_has_frame(false);

    let gtkbox_clone = gtkbox.clone();
    let category_str = category.to_string();
    let changed_options_clone = changed_options.clone();
    let vbox_clone = vbox.clone();

    delete_button.connect_clicked(move |_| {
        gtkbox_clone.remove(&vbox_clone);

        let mut changes = changed_options_clone.borrow_mut();

        changes.remove(&(category_str.clone(), format!("{}_name", raw)));
        changes.remove(&(category_str.clone(), format!("{}_value", raw)));

        changes.insert(
            (category_str.clone(), format!("{}_delete", raw)),
            "DELETE".to_string(),
        );
    });

    boxline.append(&delete_button);

    vbox.append(&boxline);
    vbox.append(&editor_box);

    gtkbox.append(&vbox);
}

fn add_guide(container: &Box, name: &str, default_collapsed: bool) {
    let expander = Expander::new(None);
    expander.set_margin_top(5);
    expander.set_margin_bottom(10);

    let title_label = Label::new(None);
    title_label.set_halign(gtk::Align::Start);
    title_label.add_css_class("heading");
    title_label.set_markup(&t!("guide"));

    expander.set_label_widget(Some(&title_label));

    expander.set_expanded(!default_collapsed);

    let guide_box = create_guide(name);
    expander.set_child(Some(&guide_box));

    container.append(&expander);

    let frame = Frame::new(None);
    frame.set_margin_bottom(10);
    container.append(&frame);
}

fn update_version_label(label: &Label, repo: &str, version: &str) {
    let latest_version = get_latest_version(repo);
    let version_str = if !latest_version.starts_with("v") {
        format!("{} ( {} )", version, &t!("unable_to_get_latest_version"))
    } else {
        match compare_versions(version, &latest_version) {
            Ordering::Greater => {
                format!(
                    "{} ( {} )",
                    version,
                    &t!("your_version_is_greater_than_latest_()", v = latest_version),
                )
            }
            Ordering::Less => {
                format!(
                    "{} ( {} )",
                    version,
                    &t!("new_version_available_()", v = latest_version),
                )
            }
            Ordering::Equal => {
                format!("{} ( {} )", version, &t!("your_version_is_up_to_date"))
            }
        }
    };
    label.set_label(&version_str);
}

// transform from general{snap{enabled = true}} to general:snap:enabled = true
fn transform_config(input: String) -> String {
    let mut result = Vec::new();
    let mut path = VecDeque::new();

    for line in input.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.ends_with('{') {
            // start of the block
            let key = line.trim_end_matches('{').trim();
            path.push_back(key.to_string());
        } else if line == "}" {
            // end of the block
            path.pop_back();
        } else if line.contains('=') {
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap().trim();
            let value = parts.next().unwrap().trim();
            let prefix = path.iter().cloned().collect::<Vec<_>>().join(":");
            let full_key = if !prefix.is_empty() {
                format!("{prefix}:{key}")
            } else {
                key.to_string()
            };
            result.push(format!("{full_key} = {value}"));
        }
    }

    result.join("\n")
}

fn extract_value(config: &HyprlandConfig, category: &str, name: &str, default: &str) -> String {
    let config_str = transform_config(config.to_string());
    if category == "layouts" {
        for line in config_str.lines().rev() {
            if line.trim().starts_with(&format!("{name} = ")) {
                return line
                    .split('=')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            }
        }
    } else {
        for line in config_str.lines().rev() {
            if line.trim().starts_with(&format!("{category}:{name} = ")) {
                return line
                    .split('=')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            }
        }
    }
    default.to_string()
}

impl ConfigWidget {
    pub fn new(category: &str, display_name: &str) -> Self {
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_vexpand(false);
        scrolled_window.set_propagate_natural_height(true);

        let container = Box::new(Orientation::Vertical, 0);
        container.set_margin_start(20);
        container.set_margin_end(20);
        container.set_margin_top(20);
        container.set_margin_bottom(20);

        scrolled_window.set_child(Some(&container));

        let mut options: HashMap<String, WidgetData> = HashMap::new();

        let first_section = Rc::new(RefCell::new(true));

        match category {
            "general" => {
                add_section(
                    &container,
                    &t!("general_category.first_section_title"),
                    &t!("general_category.first_section_description"),
                    first_section.clone(),
                );

                add_section(
                    &container,
                    &t!("general_category.layout_section_title"),
                    &t!("general_category.layout_section_description"),
                    first_section.clone(),
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "layout",
                    &t!("general_category.layout_label"),
                    &t!("general_category.layout_description"),
                    &["dwindle", "master"],
                    "dwindle",
                );

                add_section(
                    &container,
                    &t!("general_category.gaps_section_title"),
                    &t!("general_category.gaps_section_description"),
                    first_section.clone(),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "gaps_in",
                    &t!("general_category.gaps_in_label"),
                    &t!("general_category.gaps_in_description"),
                    "5",
                    (0.0, 500.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "gaps_out",
                    &t!("general_category.gaps_out_label"),
                    &t!("general_category.gaps_out_description"),
                    "20",
                    (0.0, 500.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "float_gaps",
                    &t!("general_category.float_gaps_label"),
                    &t!("general_category.float_gaps_description"),
                    "0",
                    (0.0, 500.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "gaps_workspaces",
                    &t!("general_category.gaps_workspaces_label"),
                    &t!("general_category.gaps_workspaces_description"),
                    "0",
                    (0.0, 100.0, 1.0),
                );

                add_section(
                    &container,
                    &t!("general_category.borders_section_title"),
                    &t!("general_category.borders_section_description"),
                    first_section.clone(),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "border_size",
                    &t!("general_category.border_size_label"),
                    &t!("general_category.border_size_description"),
                    "1",
                    (0.0, 20.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "no_border_on_floating",
                    &t!("general_category.no_border_on_floating_label"),
                    &t!("general_category.no_border_on_floating_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "resize_on_border",
                    &t!("general_category.resize_on_border_label"),
                    &t!("general_category.resize_on_border_description"),
                    "false",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "resize_corner",
                    &t!("general_category.resize_corner_label"),
                    &t!("general_category.resize_corner_description"),
                    &[
                        &t!("general_category.resize_corner_disabled"),
                        &t!("general_category.resize_corner_top_left"),
                        &t!("general_category.resize_corner_top_right"),
                        &t!("general_category.resize_corner_bottom_right"),
                        &t!("general_category.resize_corner_bottom_left"),
                    ],
                    "0",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "extend_border_grab_area",
                    &t!("general_category.extend_border_grab_area_label"),
                    &t!("general_category.extend_border_grab_area_description"),
                    "15",
                    (0.0, 100.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "hover_icon_on_border",
                    &t!("general_category.hover_icon_on_border_label"),
                    &t!("general_category.hover_icon_on_border_description"),
                    "true",
                );

                add_section(
                    &container,
                    &t!("general_category.snap.section_title"),
                    &t!("general_category.snap.section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "snap:enabled",
                    &t!("general_category.snap.enabled_label"),
                    &t!("general_category.snap.enabled_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "snap:window_gap",
                    &t!("general_category.snap.window_gap_label"),
                    &t!("general_category.snap.window_gap_description"),
                    "10",
                    (0.0, 100.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "snap:monitor_gap",
                    &t!("general_category.snap.monitor_gap_label"),
                    &t!("general_category.snap.monitor_gap_description"),
                    "10",
                    (0.0, 100.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "snap:border_overlap",
                    &t!("general_category.snap.border_overlap_label"),
                    &t!("general_category.snap.border_overlap_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "snap:respect_gaps",
                    &t!("general_category.snap.respect_gaps_label"),
                    &t!("general_category.snap.respect_gaps_description"),
                    "false",
                );

                add_section(
                    &container,
                    &t!("general_category.other_stuff_section_title"),
                    &t!("general_category.other_stuff_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "no_focus_fallback",
                    &t!("general_category.no_focus_fallback_label"),
                    &t!("general_category.no_focus_fallback_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "allow_tearing",
                    &t!("general_category.allow_tearing_label"),
                    &t!("general_category.allow_tearing_description"),
                    "false",
                );

                add_section(
                    &container,
                    &t!("general_category.colors_section_title"),
                    &t!("general_category.colors_section_description"),
                    first_section.clone(),
                );
                add_color_option(
                    &container,
                    &mut options,
                    "col.inactive_border",
                    &t!("general_category.col_inactive_border_label"),
                    &t!("general_category.col_inactive_border_description"),
                    "#444444FF",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "col.active_border",
                    &t!("general_category.col_active_border_label"),
                    &t!("general_category.col_active_border_description"),
                    "#FFFFFFFF",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "col.nogroup_border",
                    &t!("general_category.col_nogroup_border_label"),
                    &t!("general_category.col_nogroup_border_description"),
                    "#FFAAFFFF",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "col.nogroup_border_active",
                    &t!("general_category.col_nogroup_border_active_label"),
                    &t!("general_category.col_nogroup_border_active_description"),
                    "#FF00FFFF",
                );
            }
            "decoration" => {
                add_section(
                    &container,
                    &t!("decoration_category.window_decoration_section_title"),
                    &t!("decoration_category.window_decoration_section_description"),
                    first_section.clone(),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "rounding",
                    &t!("decoration_category.rounding_label"),
                    &t!("decoration_category.rounding_description"),
                    "0",
                    (0.0, 20.0, 1.0),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "rounding_power",
                    &t!("decoration_category.rounding_power_label"),
                    &t!("decoration_category.rounding_power_description"),
                    "2.0",
                    (2.0, 10.0, 0.1),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "active_opacity",
                    &t!("decoration_category.active_opacity_label"),
                    &t!("decoration_category.active_opacity_description"),
                    "1.0",
                    (0.0, 1.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "inactive_opacity",
                    &t!("decoration_category.inactive_opacity_label"),
                    &t!("decoration_category.inactive_opacity_description"),
                    "1.0",
                    (0.0, 1.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "fullscreen_opacity",
                    &t!("decoration_category.fullscreen_opacity_label"),
                    &t!("decoration_category.fullscreen_opacity_description"),
                    "1.0",
                    (0.0, 1.0, 0.01),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "dim_inactive",
                    &t!("decoration_category.dim_inactive_label"),
                    &t!("decoration_category.dim_inactive_description"),
                    "false",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "dim_strength",
                    &t!("decoration_category.dim_strength_label"),
                    &t!("decoration_category.dim_strength_description"),
                    "0.5",
                    (0.0, 1.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "dim_special",
                    &t!("decoration_category.dim_special_label"),
                    &t!("decoration_category.dim_special_description"),
                    "0.2",
                    (0.0, 1.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "dim_around",
                    &t!("decoration_category.dim_around_label"),
                    &t!("decoration_category.dim_around_description"),
                    "0.4",
                    (0.0, 1.0, 0.01),
                );
                add_string_option(
                    &container,
                    &mut options,
                    "screen_shader",
                    &t!("decoration_category.screen_shader_label"),
                    &t!("decoration_category.screen_shader_description"),
                    "",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "border_part_of_window",
                    &t!("decoration_category.border_part_of_window_label"),
                    &t!("decoration_category.border_part_of_window_description"),
                    "true",
                );

                add_section(
                    &container,
                    &t!("decoration_category.blur.section_title"),
                    &t!("decoration_category.blur.section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "blur:enabled",
                    &t!("decoration_category.blur.enabled_label"),
                    &t!("decoration_category.blur.enabled_description"),
                    "true",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "blur:size",
                    &t!("decoration_category.blur.size_label"),
                    &t!("decoration_category.blur.size_description"),
                    "8",
                    (1.0, 100.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "blur:passes",
                    &t!("decoration_category.blur.passes_label"),
                    &t!("decoration_category.blur.passes_description"),
                    "1",
                    (1.0, 10.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "blur:ignore_opacity",
                    &t!("decoration_category.blur.ignore_opacity_label"),
                    &t!("decoration_category.blur.ignore_opacity_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "blur:new_optimizations",
                    &t!("decoration_category.blur.new_optimizations_label"),
                    &t!("decoration_category.blur.new_optimizations_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "blur:xray",
                    &t!("decoration_category.blur.xray_label"),
                    &t!("decoration_category.blur.xray_description"),
                    "false",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "blur:noise",
                    &t!("decoration_category.blur.noise_label"),
                    &t!("decoration_category.blur.noise_description"),
                    "0.0117",
                    (0.0, 1.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "blur:contrast",
                    &t!("decoration_category.blur.contrast_label"),
                    &t!("decoration_category.blur.contrast_description"),
                    "0.8916",
                    (0.0, 2.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "blur:brightness",
                    &t!("decoration_category.blur.brightness_label"),
                    &t!("decoration_category.blur.brightness_description"),
                    "0.8172",
                    (0.0, 2.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "blur:vibrancy",
                    &t!("decoration_category.blur.vibrancy_label"),
                    &t!("decoration_category.blur.vibrancy_description"),
                    "0.1696",
                    (0.0, 1.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "blur:vibrancy_darkness",
                    &t!("decoration_category.blur.vibrancy_darkness_label"),
                    &t!("decoration_category.blur.vibrancy_darkness_description"),
                    "0.0",
                    (0.0, 1.0, 0.01),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "blur:special",
                    &t!("decoration_category.blur.special_label"),
                    &t!("decoration_category.blur.special_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "blur:popups",
                    &t!("decoration_category.blur.popups_label"),
                    &t!("decoration_category.blur.popups_description"),
                    "false",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "blur:popups_ignorealpha",
                    &t!("decoration_category.blur.popups_ignorealpha_label"),
                    &t!("decoration_category.blur.popups_ignorealpha_description"),
                    "0.2",
                    (0.0, 1.0, 0.01),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "blur:input_methods",
                    &t!("decoration_category.blur.input_methods_label"),
                    &t!("decoration_category.blur.input_methods_description"),
                    "false",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "input_methods_ignorealpha",
                    &t!("decoration_category.input_methods_ignorealpha_label"),
                    &t!("decoration_category.input_methods_ignorealpha_description"),
                    "0.2",
                    (0.0, 1.0, 0.01),
                );

                add_section(
                    &container,
                    &t!("decoration_category.shadow.section_title"),
                    &t!("decoration_category.shadow.section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "shadow:enabled",
                    &t!("decoration_category.shadow.enabled_label"),
                    &t!("decoration_category.shadow.enabled_description"),
                    "true",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "shadow:range",
                    &t!("decoration_category.shadow.range_label"),
                    &t!("decoration_category.shadow.range_description"),
                    "4",
                    (0.0, 100.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "shadow:render_power",
                    &t!("decoration_category.shadow.render_power_label"),
                    &t!("decoration_category.shadow.render_power_description"),
                    "3",
                    (1.0, 4.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "shadow:sharp",
                    &t!("decoration_category.shadow.sharp_label"),
                    &t!("decoration_category.shadow.sharp_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "shadow:ignore_window",
                    &t!("decoration_category.shadow.ignore_window_label"),
                    &t!("decoration_category.shadow.ignore_window_description"),
                    "true",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "shadow:color",
                    &t!("decoration_category.shadow.color_label"),
                    &t!("decoration_category.shadow.color_description"),
                    "#1A1A1AEE",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "shadow:color_inactive",
                    &t!("decoration_category.shadow.color_inactive_label"),
                    &t!("decoration_category.shadow.color_inactive_description"),
                    "",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "shadow:offset",
                    &t!("decoration_category.shadow.offset_label"),
                    &t!("decoration_category.shadow.offset_description"),
                    "[0, 0]",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "shadow:scale",
                    &t!("decoration_category.shadow.scale_label"),
                    &t!("decoration_category.shadow.scale_description"),
                    "1.0",
                    (0.0, 1.0, 0.01),
                );
            }
            "animations" => {
                add_section(
                    &container,
                    &t!("animations_category.animation_settings_section_title"),
                    &t!("animations_category.animation_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "enabled",
                    &t!("animations_category.enabled_label"),
                    &t!("animations_category.enabled_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "workspace_wraparound",
                    &t!("animations_category.workspace_wraparound_label"),
                    &t!("animations_category.workspace_wraparound_description"),
                    "true",
                );
            }
            "input" => {
                add_section(
                    &container,
                    &t!("input_category.input_settings_section_title"),
                    &t!("input_category.input_settings_section_description"),
                    first_section.clone(),
                );

                add_section(
                    &container,
                    &t!("input_category.keyboard_settings_section_title"),
                    &t!("input_category.keyboard_settings_section_description"),
                    first_section.clone(),
                );
                add_string_option(
                    &container,
                    &mut options,
                    "kb_model",
                    &t!("input_category.kb_model_label"),
                    &t!("input_category.kb_model_description"),
                    "",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "kb_layout",
                    &t!("input_category.kb_layout_label"),
                    &t!("input_category.kb_layout_description"),
                    "us",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "kb_variant",
                    &t!("input_category.kb_variant_label"),
                    &t!("input_category.kb_variant_description"),
                    "",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "kb_options",
                    &t!("input_category.kb_options_label"),
                    &t!("input_category.kb_options_description"),
                    "",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "kb_rules",
                    &t!("input_category.kb_rules_label"),
                    &t!("input_category.kb_rules_description"),
                    "",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "kb_file",
                    &t!("input_category.kb_file_label"),
                    &t!("input_category.kb_file_description"),
                    "",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "numlock_by_default",
                    &t!("input_category.numlock_by_default_label"),
                    &t!("input_category.numlock_by_default_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "resolve_binds_by_sym",
                    &t!("input_category.resolve_binds_by_sym_label"),
                    &t!("input_category.resolve_binds_by_sym_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "repeat_rate",
                    &t!("input_category.repeat_rate_label"),
                    &t!("input_category.repeat_rate_description"),
                    "25",
                    (0.0, 200.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "repeat_delay",
                    &t!("input_category.repeat_delay_label"),
                    &t!("input_category.repeat_delay_description"),
                    "600",
                    (0.0, 2000.0, 20.0),
                );

                add_section(
                    &container,
                    &t!("input_category.mouse_settings_section_title"),
                    &t!("input_category.mouse_settings_section_description"),
                    first_section.clone(),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "sensitivity",
                    &t!("input_category.sensitivity_label"),
                    &t!("input_category.sensitivity_description"),
                    "0.0",
                    (-1.0, 1.0, 0.02),
                );
                add_string_option(
                    &container,
                    &mut options,
                    "accel_profile",
                    &t!("input_category.accel_profile_label"),
                    &t!("input_category.accel_profile_description"),
                    "",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "force_no_accel",
                    &t!("input_category.force_no_accel_label"),
                    &t!("input_category.force_no_accel_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "left_handed",
                    &t!("input_category.left_handed_label"),
                    &t!("input_category.left_handed_description"),
                    "false",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "scroll_method",
                    &t!("input_category.scroll_method_label"),
                    &t!("input_category.scroll_method_description"),
                    "",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "scroll_button",
                    &t!("input_category.scroll_button_label"),
                    &t!("input_category.scroll_button_description"),
                    "0",
                    (0.0, 300.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "scroll_button_lock",
                    &t!("input_category.scroll_button_lock_label"),
                    &t!("input_category.scroll_button_lock_description"),
                    "false",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "scroll_factor",
                    &t!("input_category.scroll_factor_label"),
                    &t!("input_category.scroll_factor_description"),
                    "1.0",
                    (0.1, 10.0, 0.1),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "natural_scroll",
                    &t!("input_category.natural_scroll_label"),
                    &t!("input_category.natural_scroll_description"),
                    "false",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "follow_mouse",
                    &t!("input_category.follow_mouse_label"),
                    &t!("input_category.follow_mouse_description"),
                    &[
                        &t!("input_category.follow_mouse_ignore"),
                        &t!("input_category.follow_mouse_always"),
                        &t!("input_category.follow_mouse_detach"),
                        &t!("input_category.follow_mouse_separate"),
                    ],
                    "1",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "follow_mouse_threshold",
                    &t!("input_category.follow_mouse_threshold_label"),
                    &t!("input_category.follow_mouse_threshold_description"),
                    "0.0",
                    (0.0, 500.0, 0.25),
                );
                add_string_option(
                    &container,
                    &mut options,
                    "scroll_points",
                    &t!("input_category.scroll_points_label"),
                    &t!("input_category.scroll_points_description"),
                    "",
                );

                add_section(
                    &container,
                    &t!("input_category.focus_settings_section_title"),
                    &t!("input_category.focus_settings_section_description"),
                    first_section.clone(),
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "focus_on_close",
                    &t!("input_category.focus_on_close_label"),
                    &t!("input_category.focus_on_close_description"),
                    &[
                        &t!("input_category.focus_on_close_next_window_candidate"),
                        &t!("input_category.focus_on_close_window_under_cursor"),
                    ],
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "mouse_refocus",
                    &t!("input_category.mouse_refocus_label"),
                    &t!("input_category.mouse_refocus_description"),
                    "true",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "float_switch_override_focus",
                    &t!("input_category.float_switch_override_focus_label"),
                    &t!("input_category.float_switch_override_focus_description"),
                    &[
                        &t!("input_category.float_switch_override_focus_disabled"),
                        &t!("input_category.float_switch_override_focus_enabled"),
                        &t!("input_category.float_switch_override_focus_focus_follow_mouse"),
                    ],
                    "1",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "special_fallthrough",
                    &t!("input_category.special_fallthrough_label"),
                    &t!("input_category.special_fallthrough_description"),
                    "false",
                );

                add_section(
                    &container,
                    &t!("input_category.touchpad.settings_section_title"),
                    &t!("input_category.touchpad.settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "touchpad:disable_while_typing",
                    &t!("input_category.touchpad.disable_while_typing_label"),
                    &t!("input_category.touchpad.disable_while_typing_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "touchpad:natural_scroll",
                    &t!("input_category.touchpad.natural_scroll_label"),
                    &t!("input_category.touchpad.natural_scroll_description"),
                    "false",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "touchpad:scroll_factor",
                    &t!("input_category.touchpad.scroll_factor_label"),
                    &t!("input_category.touchpad.scroll_factor_description"),
                    "1.0",
                    (0.0, 2.0, 0.02),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "touchpad:middle_button_emulation",
                    &t!("input_category.touchpad.middle_button_emulation_label"),
                    &t!("input_category.touchpad.middle_button_emulation_description"),
                    "false",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "touchpad:tap_button_map",
                    &t!("input_category.touchpad.tap_button_map_label"),
                    &t!("input_category.touchpad.tap_button_map_description"),
                    "lrm",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "touchpad:clickfinger_behavior",
                    &t!("input_category.touchpad.clickfinger_behavior_label"),
                    &t!("input_category.touchpad.clickfinger_behavior_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "touchpad:tap-to-click",
                    &t!("input_category.touchpad.tap_to_click_label"),
                    &t!("input_category.touchpad.tap_to_click_description"),
                    "true",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "touchpad:drag_lock",
                    &t!("input_category.touchpad.drag_lock_label"),
                    &t!("input_category.touchpad.drag_lock_description"),
                    &[
                        &t!("input_category.touchpad.drag_lock_disabled"),
                        &t!("input_category.touchpad.drag_lock_enabled_with_timeout"),
                        &t!("input_category.touchpad.drag_lock_enabled_sticky"),
                    ],
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "touchpad:tap-and-drag",
                    &t!("input_category.touchpad.tap_and_drag_label"),
                    &t!("input_category.touchpad.tap_and_drag_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "touchpad:flip_x",
                    &t!("input_category.touchpad.flip_x_label"),
                    &t!("input_category.touchpad.flip_x_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "touchpad:flip_y",
                    &t!("input_category.touchpad.flip_y_label"),
                    &t!("input_category.touchpad.flip_y_description"),
                    "false",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "touchpad:drag_3fg",
                    &t!("input_category.touchpad.drag_3fg_label"),
                    &t!("input_category.touchpad.drag_3fg_description"),
                    &[
                        &t!("input_category.touchpad.drag_3fg_disabled"),
                        &t!("input_category.touchpad.drag_3fg_3_fingers"),
                        &t!("input_category.touchpad.drag_3fg_4_fingers"),
                    ],
                    "0",
                );

                add_section(
                    &container,
                    &t!("input_category.touchscreen_settings_section_title"),
                    &t!("input_category.touchscreen_settings_section_description"),
                    first_section.clone(),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "touchdevice:transform",
                    &t!("input_category.touchdevice.transform_label"),
                    &t!("input_category.touchdevice.transform_description"),
                    "-1",
                    (-1.0, 7.0, 1.0),
                );
                add_string_option(
                    &container,
                    &mut options,
                    "touchdevice:output",
                    &t!("input_category.touchdevice.output_label"),
                    &t!("input_category.touchdevice.output_description"),
                    "[[Auto]]",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "touchdevice:enabled",
                    &t!("input_category.touchdevice.enabled_label"),
                    &t!("input_category.touchdevice.enabled_description"),
                    "true",
                );

                add_section(
                    &container,
                    &t!("input_category.virtual_keyboard_settings_section_title"),
                    &t!("input_category.virtual_keyboard_settings_section_description"),
                    first_section.clone(),
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "virtualkeyboard:share_states",
                    &t!("input_category.virtualkeyboard.share_states_label"),
                    &t!("input_category.virtualkeyboard.share_states_description"),
                    &[
                        &t!("input_category.virtualkeyboard.share_states_no"),
                        &t!("input_category.virtualkeyboard.share_states_yes"),
                        &t!("input_category.virtualkeyboard.share_states_yes_unless_ime_client"),
                    ],
                    "2",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "virtualkeyboard:release_pressed_on_close",
                    &t!("input_category.virtualkeyboard.release_pressed_on_close_label"),
                    &t!("input_category.virtualkeyboard.release_pressed_on_close_description"),
                    "false",
                );

                add_section(
                    &container,
                    &t!("input_category.tablet.settings_section_title"),
                    &t!("input_category.tablet.settings_section_description"),
                    first_section.clone(),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "tablet:transform",
                    &t!("input_category.tablet.transform_label"),
                    &t!("input_category.tablet.transform_description"),
                    "-1",
                    (-1.0, 7.0, 1.0),
                );
                add_string_option(
                    &container,
                    &mut options,
                    "tablet:output",
                    &t!("input_category.tablet.output_label"),
                    &t!("input_category.tablet.output_description"),
                    "",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "tablet:region_position",
                    &t!("input_category.tablet.region_position_label"),
                    &t!("input_category.tablet.region_position_description"),
                    "[0, 0]",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "tablet:absolute_position",
                    &t!("input_category.tablet.absolute_position_label"),
                    &t!("input_category.tablet.absolute_position_description"),
                    "false",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "tablet:region_size",
                    &t!("input_category.tablet.region_size_label"),
                    &t!("input_category.tablet.region_size_description"),
                    "[0, 0]",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "tablet:relative_input",
                    &t!("input_category.tablet.relative_input_label"),
                    &t!("input_category.tablet.relative_input_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "tablet:left_handed",
                    &t!("input_category.tablet.left_handed_label"),
                    &t!("input_category.tablet.left_handed_description"),
                    "false",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "tablet:active_area_size",
                    &t!("input_category.tablet.active_area_size_label"),
                    &t!("input_category.tablet.active_area_size_description"),
                    "[0, 0]",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "tablet:active_area_position",
                    &t!("input_category.tablet.active_area_position_label"),
                    &t!("input_category.tablet.active_area_position_description"),
                    "[0, 0]",
                );

                add_section(
                    &container,
                    &t!("input_category.miscellaneous_input_settings_section_title"),
                    &t!("input_category.miscellaneous_input_settings_section_description"),
                    first_section.clone(),
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "off_window_axis_events",
                    &t!("input_category.off_window_axis_events_label"),
                    &t!("input_category.off_window_axis_events_description"),
                    &[
                        &t!("input_category.off_window_axis_events_ignore_axis_events"),
                        &t!("input_category.off_window_axis_events_sends_out-of-bound_coordinates"),
                        &t!("input_category.off_window_axis_events_fakes_pointer_coordinates"),
                        &t!("input_category.off_window_axis_events_warps_the_cursor"),
                    ],
                    "1",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "emulate_discrete_scroll",
                    &t!("input_category.emulate_discrete_scroll_label"),
                    &t!("input_category.emulate_discrete_scroll_description"),
                    &[
                        &t!("input_category.emulate_discrete_scroll_disables_it"),
                        &t!("input_category.emulate_discrete_scroll_non-standard_events_only"),
                        &t!("input_category.emulate_discrete_scroll_force"),
                    ],
                    "1",
                );
            }
            "gestures" => {
                add_section(
                    &container,
                    &t!("gestures_category.gesture_settings_section_title"),
                    &t!("gestures_category.gesture_settings_section_description"),
                    first_section.clone(),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "workspace_swipe_distance",
                    &t!("gestures_category.workspace_swipe_distance_label"),
                    &t!("gestures_category.workspace_swipe_distance_description"),
                    "300",
                    (0.0, 2000.0, 10.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "workspace_swipe_touch",
                    &t!("gestures_category.workspace_swipe_touch_label"),
                    &t!("gestures_category.workspace_swipe_touch_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "workspace_swipe_invert",
                    &t!("gestures_category.workspace_swipe_invert_label"),
                    &t!("gestures_category.workspace_swipe_invert_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "workspace_swipe_touch_invert",
                    &t!("gestures_category.workspace_swipe_touch_invert_label"),
                    &t!("gestures_category.workspace_swipe_touch_invert_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "workspace_swipe_min_speed_to_force",
                    &t!("gestures_category.workspace_swipe_min_speed_to_force_label"),
                    &t!("gestures_category.workspace_swipe_min_speed_to_force_description"),
                    "30",
                    (0.0, 200.0, 1.0),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "workspace_swipe_cancel_ratio",
                    &t!("gestures_category.workspace_swipe_cancel_ratio_label"),
                    &t!("gestures_category.workspace_swipe_cancel_ratio_description"),
                    "0.5",
                    (0.0, 1.0, 0.01),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "workspace_swipe_create_new",
                    &t!("gestures_category.workspace_swipe_create_new_label"),
                    &t!("gestures_category.workspace_swipe_create_new_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "workspace_swipe_direction_lock",
                    &t!("gestures_category.workspace_swipe_direction_lock_label"),
                    &t!("gestures_category.workspace_swipe_direction_lock_description"),
                    "true",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "workspace_swipe_direction_lock_threshold",
                    &t!("gestures_category.workspace_swipe_direction_lock_threshold_label"),
                    &t!("gestures_category.workspace_swipe_direction_lock_threshold_description"),
                    "10",
                    (0.0, 200.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "workspace_swipe_forever",
                    &t!("gestures_category.workspace_swipe_forever_label"),
                    &t!("gestures_category.workspace_swipe_forever_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "workspace_swipe_use_r",
                    &t!("gestures_category.workspace_swipe_use_r_label"),
                    &t!("gestures_category.workspace_swipe_use_r_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "close_max_timeout",
                    &t!("gestures_category.close_max_timeout_label"),
                    &t!("gestures_category.close_max_timeout_description"),
                    "1000",
                    (10.0, 2000.0, 10.0),
                );
            }
            "group" => {
                add_section(
                    &container,
                    &t!("group_category.group_settings_section_title"),
                    &t!("group_category.group_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "auto_group",
                    &t!("group_category.auto_group_label"),
                    &t!("group_category.auto_group_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "insert_after_current",
                    &t!("group_category.insert_after_current_label"),
                    &t!("group_category.insert_after_current_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "focus_removed_window",
                    &t!("group_category.focus_removed_window_label"),
                    &t!("group_category.focus_removed_window_description"),
                    "true",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "drag_into_group",
                    &t!("group_category.drag_into_group_label"),
                    &t!("group_category.drag_into_group_description"),
                    &[
                        &t!("group_category.drag_into_group_disabled"),
                        &t!("group_category.drag_into_group_enabled"),
                        &t!("group_category.drag_into_group_only_into_groupbar"),
                    ],
                    "1",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "merge_groups_on_drag",
                    &t!("group_category.merge_groups_on_drag_label"),
                    &t!("group_category.merge_groups_on_drag_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "merge_groups_on_groupbar",
                    &t!("group_category.merge_groups_on_groupbar_label"),
                    &t!("group_category.merge_groups_on_groupbar_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "merge_floated_into_tiled_on_groupbar",
                    &t!("group_category.merge_floated_into_tiled_on_groupbar_label"),
                    &t!("group_category.merge_floated_into_tiled_on_groupbar_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "group_on_movetoworkspace",
                    &t!("group_category.group_on_movetoworkspace_label"),
                    &t!("group_category.group_on_movetoworkspace_description"),
                    "false",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "col.border_active",
                    &t!("group_category.col_border_active_label"),
                    &t!("group_category.col_border_active_description"),
                    "#FFFF0066",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "col.border_inactive",
                    &t!("group_category.col_border_inactive_label"),
                    &t!("group_category.col_border_inactive_description"),
                    "#77770066",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "col.border_locked_active",
                    &t!("group_category.col_border_locked_active_label"),
                    &t!("group_category.col_border_locked_active_description"),
                    "#FF550066",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "col.border_locked_inactive",
                    &t!("group_category.col_border_locked_inactive_label"),
                    &t!("group_category.col_border_locked_inactive_description"),
                    "#77550066",
                );

                add_section(
                    &container,
                    &t!("group_category.groupbar.settings_section_title"),
                    &t!("group_category.groupbar.settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "groupbar:enabled",
                    &t!("group_category.groupbar.enabled_label"),
                    &t!("group_category.groupbar.enabled_description"),
                    "true",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "groupbar:font_family",
                    &t!("group_category.groupbar.font_family_label"),
                    &t!("group_category.groupbar.font_family_description"),
                    "",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:font_size",
                    &t!("group_category.groupbar.font_size_label"),
                    &t!("group_category.groupbar.font_size_description"),
                    "8",
                    (2.0, 64.0, 1.0),
                );
                add_string_option(
                    &container,
                    &mut options,
                    "groupbar:font_weight_active",
                    &t!("group_category.groupbar.font_weight_active_label"),
                    &t!("group_category.groupbar.font_weight_active_description"),
                    "normal",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "groupbar:font_weight_inactive",
                    &t!("group_category.groupbar.font_weight_inactive_label"),
                    &t!("group_category.groupbar.font_weight_inactive_description"),
                    "normal",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "groupbar:gradients",
                    &t!("group_category.groupbar.gradients_label"),
                    &t!("group_category.groupbar.gradients_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:height",
                    &t!("group_category.groupbar.height_label"),
                    &t!("group_category.groupbar.height_description"),
                    "14",
                    (1.0, 64.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:indicator_gap",
                    &t!("group_category.groupbar.indicator_gap_label"),
                    &t!("group_category.groupbar.indicator_gap_description"),
                    "0",
                    (0.0, 64.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:indicator_height",
                    &t!("group_category.groupbar.indicator_height_label"),
                    &t!("group_category.groupbar.indicator_height_description"),
                    "3",
                    (1.0, 64.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "groupbar:stacked",
                    &t!("group_category.groupbar.stacked_label"),
                    &t!("group_category.groupbar.stacked_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:priority",
                    &t!("group_category.groupbar.priority_label"),
                    &t!("group_category.groupbar.priority_description"),
                    "3",
                    (0.0, 6.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "groupbar:render_titles",
                    &t!("group_category.groupbar.render_titles_label"),
                    &t!("group_category.groupbar.render_titles_description"),
                    "true",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:text_offset",
                    &t!("group_category.groupbar.text_offset_label"),
                    &t!("group_category.groupbar.text_offset_description"),
                    "0",
                    (-20.0, 20.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "groupbar:scrolling",
                    &t!("group_category.groupbar.scrolling_label"),
                    &t!("group_category.groupbar.scrolling_description"),
                    "true",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:rounding",
                    &t!("group_category.groupbar.rounding_label"),
                    &t!("group_category.groupbar.rounding_description"),
                    "1",
                    (0.0, 20.0, 1.0),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "groupbar:rounding_power",
                    &t!("group_category.groupbar.rounding_power_label"),
                    &t!("group_category.groupbar.rounding_power_description"),
                    "2",
                    (2.0, 10.0, 0.1),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:gradient_rounding",
                    &t!("group_category.groupbar.gradient_rounding_label"),
                    &t!("group_category.groupbar.gradient_rounding_description"),
                    "1",
                    (0.0, 20.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "groupbar:round_only_edges",
                    &t!("group_category.groupbar.round_only_edges_label"),
                    &t!("group_category.groupbar.round_only_edges_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "groupbar:gradient_round_only_edges",
                    &t!("group_category.groupbar.gradient_round_only_edges_label"),
                    &t!("group_category.groupbar.gradient_round_only_edges_description"),
                    "true",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "groupbar:text_color",
                    &t!("group_category.groupbar.text_color_label"),
                    &t!("group_category.groupbar.text_color_description"),
                    "#FFFFFFFF",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "groupbar:text_color_inactive",
                    &t!("group_category.groupbar.text_color_inactive_label"),
                    &t!("group_category.groupbar.text_color_inactive_description"),
                    "",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "groupbar:text_color_locked_active",
                    &t!("group_category.groupbar.text_color_locked_active_label"),
                    &t!("group_category.groupbar.text_color_locked_active_description"),
                    "",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "groupbar:text_color_locked_inactive",
                    &t!("group_category.groupbar.text_color_locked_inactive_label"),
                    &t!("group_category.groupbar.text_color_locked_inactive_description"),
                    "",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "groupbar:col.active",
                    &t!("group_category.groupbar.col_active_label"),
                    &t!("group_category.groupbar.col_active_description"),
                    "#66FFFF00",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "groupbar:col.inactive",
                    &t!("group_category.groupbar.col_inactive_label"),
                    &t!("group_category.groupbar.col_inactive_description"),
                    "#77770066",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "groupbar:col.locked_active",
                    &t!("group_category.groupbar.col_locked_active_label"),
                    &t!("group_category.groupbar.col_locked_active_description"),
                    "#FF550066",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "groupbar:col.locked_inactive",
                    &t!("group_category.groupbar.col_locked_inactive_label"),
                    &t!("group_category.groupbar.col_locked_inactive_description"),
                    "#77550066",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:gaps_in",
                    &t!("group_category.groupbar.gaps_in_label"),
                    &t!("group_category.groupbar.gaps_in_description"),
                    "2",
                    (0.0, 20.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "groupbar:gaps_out",
                    &t!("group_category.groupbar.gaps_out_label"),
                    &t!("group_category.groupbar.gaps_out_description"),
                    "2",
                    (0.0, 20.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "groupbar:keep_upper_gap",
                    &t!("group_category.groupbar.keep_upper_gap_label"),
                    &t!("group_category.groupbar.keep_upper_gap_description"),
                    "true",
                );
            }
            "misc" => {
                add_section(
                    &container,
                    &t!("misc_category.miscellaneous_settings_section_title"),
                    &t!("misc_category.miscellaneous_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_hyprland_logo",
                    &t!("misc_category.disable_hyprland_logo_label"),
                    &t!("misc_category.disable_hyprland_logo_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_splash_rendering",
                    &t!("misc_category.disable_splash_rendering_label"),
                    &t!("misc_category.disable_splash_rendering_description"),
                    "false",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "col.splash",
                    &t!("misc_category.col_splash_label"),
                    &t!("misc_category.col_splash_description"),
                    "#FFFFFFFF",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "font_family",
                    &t!("misc_category.font_family_label"),
                    &t!("misc_category.font_family_description"),
                    "Sans",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "splash_font_family",
                    &t!("misc_category.splash_font_family_label"),
                    &t!("misc_category.splash_font_family_description"),
                    "",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "force_default_wallpaper",
                    &t!("misc_category.force_default_wallpaper_label"),
                    &t!("misc_category.force_default_wallpaper_description"),
                    "-1",
                    (-1.0, 2.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "vfr",
                    &t!("misc_category.vfr_label"),
                    &t!("misc_category.vfr_description"),
                    "true",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "vrr",
                    &t!("misc_category.vrr_label"),
                    &t!("misc_category.vrr_description"),
                    &[
                        &t!("misc_category.vrr_off"),
                        &t!("misc_category.vrr_on"),
                        &t!("misc_category.vrr_fullscreen_only"),
                        &t!("misc_category.vrr_fullscreen_with_video/game"),
                    ],
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "mouse_move_enables_dpms",
                    &t!("misc_category.mouse_move_enables_dpms_label"),
                    &t!("misc_category.mouse_move_enables_dpms_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "key_press_enables_dpms",
                    &t!("misc_category.key_press_enables_dpms_label"),
                    &t!("misc_category.key_press_enables_dpms_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "name_vk_after_proc",
                    &t!("misc_category.name_vk_after_proc_label"),
                    &t!("misc_category.name_vk_after_proc_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "always_follow_on_dnd",
                    &t!("misc_category.always_follow_on_dnd_label"),
                    &t!("misc_category.always_follow_on_dnd_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "layers_hog_keyboard_focus",
                    &t!("misc_category.layers_hog_keyboard_focus_label"),
                    &t!("misc_category.layers_hog_keyboard_focus_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "animate_manual_resizes",
                    &t!("misc_category.animate_manual_resizes_label"),
                    &t!("misc_category.animate_manual_resizes_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "animate_mouse_windowdragging",
                    &t!("misc_category.animate_mouse_windowdragging_label"),
                    &t!("misc_category.animate_mouse_windowdragging_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_autoreload",
                    &t!("misc_category.disable_autoreload_label"),
                    &t!("misc_category.disable_autoreload_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "enable_swallow",
                    &t!("misc_category.enable_swallow_label"),
                    &t!("misc_category.enable_swallow_description"),
                    "false",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "swallow_regex",
                    &t!("misc_category.swallow_regex_label"),
                    &t!("misc_category.swallow_regex_description"),
                    "",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "swallow_exception_regex",
                    &t!("misc_category.swallow_exception_regex_label"),
                    &t!("misc_category.swallow_exception_regex_description"),
                    "",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "focus_on_activate",
                    &t!("misc_category.focus_on_activate_label"),
                    &t!("misc_category.focus_on_activate_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "mouse_move_focuses_monitor",
                    &t!("misc_category.mouse_move_focuses_monitor_label"),
                    &t!("misc_category.mouse_move_focuses_monitor_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "allow_session_lock_restore",
                    &t!("misc_category.allow_session_lock_restore_label"),
                    &t!("misc_category.allow_session_lock_restore_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "session_lock_xray",
                    &t!("misc_category.session_lock_xray_label"),
                    &t!("misc_category.session_lock_xray_description"),
                    "false",
                );
                add_color_option(
                    &container,
                    &mut options,
                    "background_color",
                    &t!("misc_category.background_color_label"),
                    &t!("misc_category.background_color_description"),
                    "#111111",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "close_special_on_empty",
                    &t!("misc_category.close_special_on_empty_label"),
                    &t!("misc_category.close_special_on_empty_description"),
                    "true",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "new_window_takes_over_fullscreen",
                    &t!("misc_category.new_window_takes_over_fullscreen_label"),
                    &t!("misc_category.new_window_takes_over_fullscreen_description"),
                    &[
                        &t!("misc_category.new_window_takes_over_fullscreen_behind"),
                        &t!("misc_category.new_window_takes_over_fullscreen_takes_over"),
                        &t!(
                            "misc_category.new_window_takes_over_fullscreen_unfullscreen/unmaximize"
                        ),
                    ],
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "exit_window_retains_fullscreen",
                    &t!("misc_category.exit_window_retains_fullscreen_label"),
                    &t!("misc_category.exit_window_retains_fullscreen_description"),
                    "false",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "initial_workspace_tracking",
                    &t!("misc_category.initial_workspace_tracking_label"),
                    &t!("misc_category.initial_workspace_tracking_description"),
                    &[
                        &t!("misc_category.initial_workspace_tracking_disabled"),
                        &t!("misc_category.initial_workspace_tracking_single-shot"),
                        &t!("misc_category.initial_workspace_tracking_persistent"),
                    ],
                    "1",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "middle_click_paste",
                    &t!("misc_category.middle_click_paste_label"),
                    &t!("misc_category.middle_click_paste_description"),
                    "true",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "render_unfocused_fps",
                    &t!("misc_category.render_unfocused_fps_label"),
                    &t!("misc_category.render_unfocused_fps_description"),
                    "15",
                    (1.0, 120.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_xdg_env_checks",
                    &t!("misc_category.disable_xdg_env_checks_label"),
                    &t!("misc_category.disable_xdg_env_checks_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_hyprland_qtutils_check",
                    &t!("misc_category.disable_hyprland_qtutils_check_label"),
                    &t!("misc_category.disable_hyprland_qtutils_check_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "lockdead_screen_delay",
                    &t!("misc_category.lockdead_screen_delay_label"),
                    &t!("misc_category.lockdead_screen_delay_description"),
                    "1000",
                    (0.0, 5000.0, 100.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "enable_anr_dialog",
                    &t!("misc_category.enable_anr_dialog_label"),
                    &t!("misc_category.enable_anr_dialog_description"),
                    "true",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "anr_missed_pings",
                    &t!("misc_category.anr_missed_pings_label"),
                    &t!("misc_category.anr_missed_pings_description"),
                    "1",
                    (1.0, 10.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "screencopy_force_8b",
                    &t!("misc_category.screencopy_force_8b_label"),
                    &t!("misc_category.screencopy_force_8b_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_scale_notification",
                    &t!("misc_category.disable_scale_notification_label"),
                    &t!("misc_category.disable_scale_notification_description"),
                    "false",
                );
            }
            "binds" => {
                add_section(
                    &container,
                    &t!("binds_category.bind_settings_section_title"),
                    &t!("binds_category.bind_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "pass_mouse_when_bound",
                    &t!("binds_category.pass_mouse_when_bound_label"),
                    &t!("binds_category.pass_mouse_when_bound_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "scroll_event_delay",
                    &t!("binds_category.scroll_event_delay_label"),
                    &t!("binds_category.scroll_event_delay_description"),
                    "300",
                    (0.0, 2000.0, 20.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "workspace_back_and_forth",
                    &t!("binds_category.workspace_back_and_forth_label"),
                    &t!("binds_category.workspace_back_and_forth_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "hide_special_on_workspace_change",
                    &t!("binds_category.hide_special_on_workspace_change_label"),
                    &t!("binds_category.hide_special_on_workspace_change_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "allow_workspace_cycles",
                    &t!("binds_category.allow_workspace_cycles_label"),
                    &t!("binds_category.allow_workspace_cycles_description"),
                    "false",
                );
                add_bool_int_option(
                    &container,
                    &mut options,
                    "workspace_center_on",
                    &t!("binds_category.workspace_center_on_label"),
                    &t!("binds_category.workspace_center_on_description"),
                    "0",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "focus_preferred_method",
                    &t!("binds_category.focus_preferred_method_label"),
                    &t!("binds_category.focus_preferred_method_description"),
                    &[
                        &t!("binds_category.focus_preferred_method_history"),
                        &t!("binds_category.focus_preferred_method_length"),
                    ],
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "ignore_group_lock",
                    &t!("binds_category.ignore_group_lock_label"),
                    &t!("binds_category.ignore_group_lock_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "movefocus_cycles_fullscreen",
                    &t!("binds_category.movefocus_cycles_fullscreen_label"),
                    &t!("binds_category.movefocus_cycles_fullscreen_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "movefocus_cycles_groupfirst",
                    &t!("binds_category.movefocus_cycles_groupfirst_label"),
                    &t!("binds_category.movefocus_cycles_groupfirst_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_keybind_grabbing",
                    &t!("binds_category.disable_keybind_grabbing_label"),
                    &t!("binds_category.disable_keybind_grabbing_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "window_direction_monitor_fallback",
                    &t!("binds_category.window_direction_monitor_fallback_label"),
                    &t!("binds_category.window_direction_monitor_fallback_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "allow_pin_fullscreen",
                    &t!("binds_category.allow_pin_fullscreen_label"),
                    &t!("binds_category.allow_pin_fullscreen_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "drag_threshold",
                    &t!("binds_category.drag_threshold_label"),
                    &t!("binds_category.drag_threshold_description"),
                    "0",
                    (0.0, MAX_SAFE_INTEGER_F64, 1.0),
                );
            }
            "xwayland" => {
                add_section(
                    &container,
                    &t!("xwayland_category.xwayland_settings_section_title"),
                    &t!("xwayland_category.xwayland_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "enabled",
                    &t!("xwayland_category.enabled_label"),
                    &t!("xwayland_category.enabled_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "use_nearest_neighbor",
                    &t!("xwayland_category.use_nearest_neighbor_label"),
                    &t!("xwayland_category.use_nearest_neighbor_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "force_zero_scaling",
                    &t!("xwayland_category.force_zero_scaling_label"),
                    &t!("xwayland_category.force_zero_scaling_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "create_abstract_socket",
                    &t!("xwayland_category.create_abstract_socket_label"),
                    &t!("xwayland_category.create_abstract_socket_description"),
                    "false",
                );
            }
            "opengl" => {
                add_section(
                    &container,
                    &t!("opengl_category.opengl_settings_section_title"),
                    &t!("opengl_category.opengl_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "nvidia_anti_flicker",
                    &t!("opengl_category.nvidia_anti_flicker_label"),
                    &t!("opengl_category.nvidia_anti_flicker_description"),
                    "true",
                );
            }
            "render" => {
                add_section(
                    &container,
                    &t!("render_category.render_settings_section_title"),
                    &t!("render_category.render_settings_section_description"),
                    first_section.clone(),
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "direct_scanout",
                    &t!("render_category.direct_scanout_label"),
                    &t!("render_category.direct_scanout_description"),
                    &[
                        &t!("render_category.direct_scanout_off"),
                        &t!("render_category.direct_scanout_on"),
                        &t!("render_category.direct_scanout_auto"),
                    ],
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "expand_undersized_textures",
                    &t!("render_category.expand_undersized_textures_label"),
                    &t!("render_category.expand_undersized_textures_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "xp_mode",
                    &t!("render_category.xp_mode_label"),
                    &t!("render_category.xp_mode_description"),
                    "false",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "ctm_animation",
                    &t!("render_category.ctm_animation_label"),
                    &t!("render_category.ctm_animation_description"),
                    &[
                        &t!("render_category.ctm_animation_off"),
                        &t!("render_category.ctm_animation_on"),
                        &t!("render_category.ctm_animation_auto"),
                    ],
                    "2",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "cm_fs_passthrough",
                    &t!("render_category.cm_fs_passthrough_label"),
                    &t!("render_category.cm_fs_passthrough_description"),
                    &[
                        &t!("render_category.cm_fs_passthrough_off"),
                        &t!("render_category.cm_fs_passthrough_always"),
                        &t!("render_category.cm_fs_passthrough_hdr_only"),
                    ],
                    "2",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "cm_enabled",
                    &t!("render_category.cm_enabled_label"),
                    &t!("render_category.cm_enabled_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "send_content_type",
                    &t!("render_category.send_content_type_label"),
                    &t!("render_category.send_content_type_description"),
                    "true",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "cm_auto_hdr",
                    &t!("render_category.cm_auto_hdr_label"),
                    &t!("render_category.cm_auto_hdr_description"),
                    &[
                        &t!("render_category.cm_auto_hdr_off"),
                        &t!("render_category.cm_auto_hdr_hdr"),
                        &t!("render_category.cm_auto_hdr_hdredid"),
                    ],
                    "1",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "new_render_scheduling",
                    &t!("render_category.new_render_scheduling_label"),
                    &t!("render_category.new_render_scheduling_description"),
                    "false",
                );
            }
            "cursor" => {
                add_section(
                    &container,
                    &t!("cursor_category.cursor_settings_section_title"),
                    &t!("cursor_category.cursor_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "sync_gsettings_theme",
                    &t!("cursor_category.sync_gsettings_theme_label"),
                    &t!("cursor_category.sync_gsettings_theme_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "invisible",
                    &t!("cursor_category.invisible_label"),
                    &t!("cursor_category.invisible_description"),
                    "false",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "no_hardware_cursors",
                    &t!("cursor_category.no_hardware_cursors_label"),
                    &t!("cursor_category.no_hardware_cursors_description"),
                    &[
                        &t!("cursor_category.no_hardware_cursors_use_hwc_if_possible"),
                        &t!("cursor_category.no_hardware_cursors_dont_use_hwc"),
                        &t!("cursor_category.no_hardware_cursors_auto"),
                    ],
                    "2",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "no_break_fs_vrr",
                    &t!("cursor_category.no_break_fs_vrr_label"),
                    &t!("cursor_category.no_break_fs_vrr_description"),
                    &[
                        &t!("cursor_category.no_break_fs_vrr_off"),
                        &t!("cursor_category.no_break_fs_vrr_on"),
                        &t!("cursor_category.no_break_fs_vrr_auto"),
                    ],
                    "2",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "min_refresh_rate",
                    &t!("cursor_category.min_refresh_rate_label"),
                    &t!("cursor_category.min_refresh_rate_description"),
                    "24",
                    (10.0, 500.0, 1.0),
                );
                add_int_option(
                    &container,
                    &mut options,
                    "hotspot_padding",
                    &t!("cursor_category.hotspot_padding_label"),
                    &t!("cursor_category.hotspot_padding_description"),
                    "1",
                    (0.0, 20.0, 1.0),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "inactive_timeout",
                    &t!("cursor_category.inactive_timeout_label"),
                    &t!("cursor_category.inactive_timeout_description"),
                    "0",
                    (0.0, 20.0, 1.0),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "no_warps",
                    &t!("cursor_category.no_warps_label"),
                    &t!("cursor_category.no_warps_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "persistent_warps",
                    &t!("cursor_category.persistent_warps_label"),
                    &t!("cursor_category.persistent_warps_description"),
                    "false",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "warp_on_change_workspace",
                    &t!("cursor_category.warp_on_change_workspace_label"),
                    &t!("cursor_category.warp_on_change_workspace_description"),
                    &[
                        &t!("cursor_category.warp_on_change_workspace_disabled"),
                        &t!("cursor_category.warp_on_change_workspace_enabled"),
                        &t!("cursor_category.warp_on_change_workspace_force"),
                    ],
                    "0",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "warp_on_toggle_special",
                    &t!("cursor_category.warp_on_toggle_special_label"),
                    &t!("cursor_category.warp_on_toggle_special_description"),
                    &[
                        &t!("cursor_category.warp_on_toggle_special_disabled"),
                        &t!("cursor_category.warp_on_toggle_special_enabled"),
                        &t!("cursor_category.warp_on_toggle_special_force"),
                    ],
                    "0",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "default_monitor",
                    &t!("cursor_category.default_monitor_label"),
                    &t!("cursor_category.default_monitor_description"),
                    "[[EMPTY]]",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "zoom_factor",
                    &t!("cursor_category.zoom_factor_label"),
                    &t!("cursor_category.zoom_factor_description"),
                    "1.0",
                    (1.0, 10.0, 0.1),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "zoom_rigid",
                    &t!("cursor_category.zoom_rigid_label"),
                    &t!("cursor_category.zoom_rigid_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "enable_hyprcursor",
                    &t!("cursor_category.enable_hyprcursor_label"),
                    &t!("cursor_category.enable_hyprcursor_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "hide_on_key_press",
                    &t!("cursor_category.hide_on_key_press_label"),
                    &t!("cursor_category.hide_on_key_press_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "hide_on_touch",
                    &t!("cursor_category.hide_on_touch_label"),
                    &t!("cursor_category.hide_on_touch_description"),
                    "true",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "use_cpu_buffer",
                    &t!("cursor_category.use_cpu_buffer_label"),
                    &t!("cursor_category.use_cpu_buffer_description"),
                    &[
                        &t!("cursor_category.use_cpu_buffer_off"),
                        &t!("cursor_category.use_cpu_buffer_on"),
                        &t!("cursor_category.use_cpu_buffer_auto"),
                    ],
                    "2",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "warp_back_after_non_mouse_input",
                    &t!("cursor_category.warp_back_after_non_mouse_input_label"),
                    &t!("cursor_category.warp_back_after_non_mouse_input_description"),
                    "false",
                );
            }
            "ecosystem" => {
                add_section(
                    &container,
                    &t!("ecosystem_category.ecosystem_settings_section_title"),
                    &t!("ecosystem_category.ecosystem_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "no_update_news",
                    &t!("ecosystem_category.no_update_news_label"),
                    &t!("ecosystem_category.no_update_news_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "no_donation_nag",
                    &t!("ecosystem_category.no_donation_nag_label"),
                    &t!("ecosystem_category.no_donation_nag_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "enforce_permissions",
                    &t!("ecosystem_category.enforce_permissions_label"),
                    &t!("ecosystem_category.enforce_permissions_description"),
                    "false",
                );
            }
            "experimental" => {
                add_section(
                    &container,
                    &t!("experimental_category.experimental_settings_section_title"),
                    &t!("experimental_category.experimental_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "xx_color_management_v4",
                    &t!("experimental_category.xx_color_management_v4_label"),
                    &t!("experimental_category.xx_color_management_v4_description"),
                    "false",
                );
            }
            "debug" => {
                add_section(
                    &container,
                    &t!("debug_category.debug_settings_section_title"),
                    &t!("debug_category.debug_settings_section_description"),
                    first_section.clone(),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "overlay",
                    &t!("debug_category.overlay_label"),
                    &t!("debug_category.overlay_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "damage_blink",
                    &t!("debug_category.damage_blink_label"),
                    &t!("debug_category.damage_blink_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_logs",
                    &t!("debug_category.disable_logs_label"),
                    &t!("debug_category.disable_logs_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_time",
                    &t!("debug_category.disable_time_label"),
                    &t!("debug_category.disable_time_description"),
                    "true",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "damage_tracking",
                    &t!("debug_category.damage_tracking_label"),
                    &t!("debug_category.damage_tracking_description"),
                    &[
                        &t!("debug_category.damage_tracking_none"),
                        &t!("debug_category.damage_tracking_monitor"),
                        &t!("debug_category.damage_tracking_full"),
                    ],
                    "2",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "enable_stdout_logs",
                    &t!("debug_category.enable_stdout_logs_label"),
                    &t!("debug_category.enable_stdout_logs_description"),
                    "false",
                );
                add_bool_int_option(
                    &container,
                    &mut options,
                    "manual_crash",
                    &t!("debug_category.manual_crash_label"),
                    &t!("debug_category.manual_crash_description"),
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "suppress_errors",
                    &t!("debug_category.suppress_errors_label"),
                    &t!("debug_category.suppress_errors_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "disable_scale_checks",
                    &t!("debug_category.disable_scale_checks_label"),
                    &t!("debug_category.disable_scale_checks_description"),
                    "false",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "error_limit",
                    &t!("debug_category.error_limit_label"),
                    &t!("debug_category.error_limit_description"),
                    "5",
                    (0.0, 20.0, 1.0),
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "error_position",
                    &t!("debug_category.error_position_label"),
                    &t!("debug_category.error_position_description"),
                    &[
                        &t!("debug_category.error_position_top"),
                        &t!("debug_category.error_position_bottom"),
                    ],
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "colored_stdout_logs",
                    &t!("debug_category.colored_stdout_logs_label"),
                    &t!("debug_category.colored_stdout_logs_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "pass",
                    &t!("debug_category.pass_label"),
                    &t!("debug_category.pass_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "full_cm_proto",
                    &t!("debug_category.full_cm_proto_label"),
                    &t!("debug_category.full_cm_proto_description"),
                    "false",
                );
            }
            "layouts" => {
                add_section(
                    &container,
                    &t!("layouts_category.layout_settings_section_title"),
                    &t!("layouts_category.layout_settings_section_description"),
                    first_section.clone(),
                );

                add_section(
                    &container,
                    &t!("layouts_category.dwindle.layout_section_title"),
                    &t!("layouts_category.dwindle.layout_section_description"),
                    first_section.clone(),
                );
                add_guide(&container, "Dwindle-Layout", false);
                add_bool_option(
                    &container,
                    &mut options,
                    "dwindle:pseudotile",
                    &t!("layouts_category.dwindle.pseudotile_label"),
                    &t!("layouts_category.dwindle.pseudotile_description"),
                    "false",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "dwindle:force_split",
                    &t!("layouts_category.dwindle.force_split_label"),
                    &t!("layouts_category.dwindle.force_split_description"),
                    &[
                        &t!("layouts_category.dwindle.force_split_split_follows_mouse"),
                        &t!("layouts_category.dwindle.force_split_always_split_left/top"),
                        &t!("layouts_category.dwindle.force_split_always_split_right/bottom"),
                    ],
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "dwindle:preserve_split",
                    &t!("layouts_category.dwindle.preserve_split_label"),
                    &t!("layouts_category.dwindle.preserve_split_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "dwindle:smart_split",
                    &t!("layouts_category.dwindle.smart_split_label"),
                    &t!("layouts_category.dwindle.smart_split_description"),
                    "false",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "dwindle:smart_resizing",
                    &t!("layouts_category.dwindle.smart_resizing_label"),
                    &t!("layouts_category.dwindle.smart_resizing_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "dwindle:permanent_direction_override",
                    &t!("layouts_category.dwindle.permanent_direction_override_label"),
                    &t!("layouts_category.dwindle.permanent_direction_override_description"),
                    "false",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "dwindle:special_scale_factor",
                    &t!("layouts_category.dwindle.special_scale_factor_label"),
                    &t!("layouts_category.dwindle.special_scale_factor_description"),
                    "1.0",
                    (0.0, 1.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "dwindle:split_width_multiplier",
                    &t!("layouts_category.dwindle.split_width_multiplier_label"),
                    &t!("layouts_category.dwindle.split_width_multiplier_description"),
                    "1.0",
                    (0.1, 3.0, 0.1),
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "dwindle:use_active_for_splits",
                    &t!("layouts_category.dwindle.use_active_for_splits_label"),
                    &t!("layouts_category.dwindle.use_active_for_splits_description"),
                    "true",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "dwindle:default_split_ratio",
                    &t!("layouts_category.dwindle.default_split_ratio_label"),
                    &t!("layouts_category.dwindle.default_split_ratio_description"),
                    "1.0",
                    (0.1, 1.9, 0.02),
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "dwindle:split_bias",
                    &t!("layouts_category.dwindle.split_bias_label"),
                    &t!("layouts_category.dwindle.split_bias_description"),
                    &[
                        &t!("layouts_category.dwindle.split_bias_directional"),
                        &t!("layouts_category.dwindle.split_bias_current_window"),
                    ],
                    "0",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "precise_mouse_move",
                    &t!("layouts_category.precise_mouse_move_label"),
                    &t!("layouts_category.precise_mouse_move_description"),
                    "false",
                );
                add_string_option(
                    &container,
                    &mut options,
                    "single_window_aspect_ratio",
                    &t!("layouts_category.single_window_aspect_ratio_label"),
                    &t!("layouts_category.single_window_aspect_ratio_description"),
                    "0 0",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "single_window_aspect_ratio_tolerance",
                    &t!("layouts_category.single_window_aspect_ratio_tolerance_label"),
                    &t!("layouts_category.single_window_aspect_ratio_tolerance_description"),
                    "0.1",
                    (0.0, 1.0, 0.01),
                );

                add_section(
                    &container,
                    &t!("layouts_category.master.layout_section_title"),
                    &t!("layouts_category.master.layout_section_description"),
                    first_section.clone(),
                );
                add_guide(&container, "Master-Layout", true);
                add_bool_option(
                    &container,
                    &mut options,
                    "master:allow_small_split",
                    &t!("layouts_category.master.allow_small_split_label"),
                    &t!("layouts_category.master.allow_small_split_description"),
                    "false",
                );
                add_float_option(
                    &container,
                    &mut options,
                    "master:special_scale_factor",
                    &t!("layouts_category.master.special_scale_factor_label"),
                    &t!("layouts_category.master.special_scale_factor_description"),
                    "1.0",
                    (0.0, 1.0, 0.01),
                );
                add_float_option(
                    &container,
                    &mut options,
                    "master:mfact",
                    &t!("layouts_category.master.mfact_label"),
                    &t!("layouts_category.master.mfact_description"),
                    "0.55",
                    (0.0, 1.0, 0.01),
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "master:new_status",
                    &t!("layouts_category.master.new_status_label"),
                    &t!("layouts_category.master.new_status_description"),
                    &["master", "slave", "inherit"],
                    "slave",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "master:new_on_top",
                    &t!("layouts_category.master.new_on_top_label"),
                    &t!("layouts_category.master.new_on_top_description"),
                    "false",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "master:new_on_active",
                    &t!("layouts_category.master.new_on_active_label"),
                    &t!("layouts_category.master.new_on_active_description"),
                    &["before", "after", "none"],
                    "none",
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "master:orientation",
                    &t!("layouts_category.master.orientation_label"),
                    &t!("layouts_category.master.orientation_description"),
                    &["left", "right", "top", "bottom", "center"],
                    "left",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "master:inherit_fullscreen",
                    &t!("layouts_category.master.inherit_fullscreen_label"),
                    &t!("layouts_category.master.inherit_fullscreen_description"),
                    "true",
                );
                add_int_option(
                    &container,
                    &mut options,
                    "master:slave_count_for_center_master",
                    &t!("layouts_category.master.slave_count_for_center_master_label"),
                    &t!("layouts_category.master.slave_count_for_center_master_description"),
                    "2",
                    (0.0, 10.0, 1.0),
                );
                add_dropdown_option(
                    &container,
                    &mut options,
                    "master:center_master_fallback",
                    &t!("layouts_category.master.center_master_fallback_label"),
                    &t!("layouts_category.master.center_master_fallback_description"),
                    &["left", "right", "top", "bottom"],
                    "left",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "master:smart_resizing",
                    &t!("layouts_category.master.smart_resizing_label"),
                    &t!("layouts_category.master.smart_resizing_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "master:drop_at_cursor",
                    &t!("layouts_category.master.drop_at_cursor_label"),
                    &t!("layouts_category.master.drop_at_cursor_description"),
                    "true",
                );
                add_bool_option(
                    &container,
                    &mut options,
                    "master:always_keep_position",
                    &t!("layouts_category.master.always_keep_position_label"),
                    &t!("layouts_category.master.always_keep_position_description"),
                    "false",
                )
            }
            "systeminfo" => {
                add_section(
                    &container,
                    &t!("system_info_category.system_info_section_title"),
                    &t!("system_info_category.system_info_section_description"),
                    first_section.clone(),
                );

                let info_box = Box::new(Orientation::Vertical, 10);
                info_box.set_margin_top(10);
                info_box.set_margin_bottom(10);
                info_box.set_margin_start(15);
                info_box.set_margin_end(15);

                let os_info_box = Box::new(Orientation::Horizontal, 5);
                os_info_box.set_margin_top(10);
                os_info_box.set_margin_bottom(10);

                if let Some(path) = get_distro_logo_path() {
                    let picture = gtk::Picture::for_filename(&path);
                    picture.set_vexpand(true);
                    picture.set_valign(gtk::Align::Fill);
                    picture.set_content_fit(gtk::ContentFit::ScaleDown);
                    picture.set_margin_end(10);
                    os_info_box.append(&picture);
                }

                let os_text_box = Box::new(Orientation::Vertical, 10);

                let (os_label, os_refresh) = add_info_row(
                    &os_text_box,
                    &t!("system_info_category.os_label"),
                    &get_os_info(),
                );
                os_refresh.connect_clicked(move |_| {
                    os_label.set_label(&get_os_info());
                });

                let (kernel_label, kernel_refresh) = add_info_row(
                    &os_text_box,
                    &t!("system_info_category.kernel_label"),
                    &get_kernel_info(),
                );
                kernel_refresh.connect_clicked(move |_| {
                    kernel_label.set_label(&get_kernel_info());
                });

                os_info_box.append(&os_text_box);

                info_box.append(&os_info_box);

                // Section for hyprland and hyprviz versions

                let hyprland_version = get_hyprland_version();

                let (hyprland_version_label, hyprland_version_refresh) = add_info_row(
                    &info_box,
                    &t!("system_info_category.hyprland_version_label"),
                    &hyprland_version,
                );
                hyprland_version_refresh.connect_clicked(move |_| {
                    update_version_label(
                        &hyprland_version_label,
                        "hyprwm/hyprland",
                        &hyprland_version,
                    );
                });

                let hyprviz_version = get_hyprviz_version();

                let (hyprviz_version_label, hyprviz_version_refresh) = add_info_row(
                    &info_box,
                    &t!("system_info_category.hyprviz_version_label"),
                    &hyprviz_version,
                );
                hyprviz_version_refresh.connect_clicked(move |_| {
                    update_version_label(
                        &hyprviz_version_label,
                        "timasoft/hyprviz",
                        &hyprviz_version,
                    );
                });

                // Section for other system info

                let (user_label, user_refresh) = add_info_row(
                    &info_box,
                    &t!("system_info_category.user_label"),
                    &get_user_info(),
                );
                user_refresh.connect_clicked(move |_| {
                    user_label.set_label(&get_user_info());
                });

                let (host_label, host_refresh) = add_info_row(
                    &info_box,
                    &t!("system_info_category.host_label"),
                    &get_host_info(),
                );
                host_refresh.connect_clicked(move |_| {
                    host_label.set_label(&get_host_info());
                });

                let (cpu_label, cpu_refresh) = add_info_row(
                    &info_box,
                    &t!("system_info_category.cpu_label"),
                    &get_cpu_info(),
                );
                cpu_refresh.connect_clicked(move |_| {
                    cpu_label.set_label(&get_cpu_info());
                });

                let (gpu_label, gpu_refresh) = add_info_row(
                    &info_box,
                    &t!("system_info_category.gpu_label"),
                    &get_gpu_info(),
                );
                gpu_refresh.connect_clicked(move |_| {
                    gpu_label.set_label(&get_gpu_info());
                });

                let (memory_label, memory_refresh) = add_info_row(
                    &info_box,
                    &t!("system_info_category.memory_label"),
                    &get_memory_info(),
                );
                memory_refresh.connect_clicked(move |_| {
                    memory_label.set_label(&get_memory_info());
                });

                let (monitors_label, monitors_refresh) = add_info_row(
                    &info_box,
                    &t!("system_info_category.monitors_label"),
                    &get_monitor_info(),
                );
                monitors_refresh.connect_clicked(move |_| {
                    monitors_label.set_label(&get_monitor_info());
                });

                container.append(&info_box);
            }
            _ => {
                match category {
                    "monitor" => {
                        add_section(
                            &container,
                            &t!("monitor_category.monitors_section_title"),
                            &t!("monitor_category.monitors_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Monitors", true);
                    }
                    "workspace" => {
                        add_section(
                            &container,
                            &t!("workspace_category.workspaces_section_title"),
                            &t!("workspace_category.workspaces_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Workspace-Rules", true);
                    }
                    "animation" => {
                        add_section(
                            &container,
                            &t!("animation_category.animations_section_title"),
                            &t!("animation_category.animations_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Animations", true);
                    }
                    "bind" => {
                        add_section(
                            &container,
                            &t!("bind_category.binds_section_title"),
                            &t!("bind_category.binds_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Binds", true);
                    }
                    "gesture" => {
                        add_section(
                            &container,
                            &t!("gesture_category.gestures_section_title"),
                            &t!("gesture_category.gestures_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Gestures", true);
                    }
                    "windowrule" => {
                        add_section(
                            &container,
                            &t!("windowrule_category.window_rules_section_title"),
                            &t!("windowrule_category.window_rules_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Window-Rules", true);
                    }
                    "layerrule" => {
                        add_section(
                            &container,
                            &t!("layerrule_category.layer_rules_section_title"),
                            &t!("layerrule_category.layer_rules_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Layer-Rules", true);
                    }
                    "exec" => {
                        add_section(
                            &container,
                            &t!("exec_category.execs_section_title"),
                            &t!("exec_category.execs_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Execs", false);
                    }
                    "env" => {
                        add_section(
                            &container,
                            &t!("env_category.envs_section_title"),
                            &t!("env_category.envs_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Envs", false);
                    }
                    "top_level" => {
                        add_section(
                            &container,
                            &t!("top_level_category.top_level_section_title"),
                            &t!("top_level_category.top_level_section_description"),
                            first_section.clone(),
                        );
                        add_guide(&container, "Dispatchers", true);
                    }
                    _ => add_section(
                        &container,
                        &t!(
                            "none_category.none_section_title",
                            category_name = display_name
                        ),
                        &t!(
                            "none_category.none_section_description",
                            category_name = display_name
                        ),
                        first_section.clone(),
                    ),
                }

                let gtkbox = Box::new(Orientation::Vertical, 0);
                container.append(&gtkbox);

                options.insert(
                    category.to_string(),
                    WidgetData {
                        widget: gtkbox.upcast(),
                        default: format!("This is a {} as widget", category),
                    },
                );
            }
        }

        ConfigWidget {
            options,
            scrolled_window,
        }
    }

    pub fn load_config(
        &self,
        window: &ApplicationWindow,
        config: &HyprlandConfig,
        profile: &str,
        category: &str,
        changed_options: Rc<RefCell<HashMap<(String, String), String>>>,
    ) {
        for (name, widget_data) in &self.options {
            let widget = &widget_data.widget;
            let default_value = &widget_data.default;
            let value = extract_value(config, category, name, default_value);
            if let Some(spin_button) = widget.downcast_ref::<SpinButton>() {
                let float_value = value.parse::<f64>().unwrap_or(0.0);
                spin_button.set_value(float_value);
                let category = category.to_string();
                let name = name.to_string();
                let changed_options = changed_options.clone();
                spin_button.connect_value_changed(move |sb| {
                    let mut changes = changed_options.borrow_mut();
                    let new_value = sb.value().to_string();
                    changes.insert((category.clone(), name.clone()), new_value);
                });
            } else if let Some(entry) = widget.downcast_ref::<Entry>() {
                entry.set_text(&value);
                let category = category.to_string();
                let name = name.to_string();
                let changed_options = changed_options.clone();
                entry.connect_changed(move |entry| {
                    let mut changes = changed_options.borrow_mut();
                    let new_value = entry.text().to_string();
                    changes.insert((category.clone(), name.clone()), new_value);
                });
            } else if let Some(switch) = widget.downcast_ref::<Switch>() {
                switch.set_active(
                    [
                        "true".to_string(),
                        "1".to_string(),
                        "on".to_string(),
                        "yes".to_string(),
                    ]
                    .contains(&value),
                );
                let category = category.to_string();
                let name = name.to_string();
                let changed_options = changed_options.clone();
                switch.connect_active_notify(move |sw| {
                    let mut changes = changed_options.borrow_mut();
                    let new_value = sw.is_active().to_string();
                    changes.insert((category.clone(), name.clone()), new_value);
                });
            } else if let Some(color_button) = widget.downcast_ref::<ColorDialogButton>() {
                if let Some((red, green, blue, alpha)) = config.parse_color(&value) {
                    color_button.set_rgba(&gdk::RGBA::new(red, green, blue, alpha));
                }
                let category = category.to_string();
                let name = name.to_string();
                let changed_options = changed_options.clone();
                color_button.connect_rgba_notify(move |cb| {
                    let mut changes = changed_options.borrow_mut();
                    let new_color = cb.rgba();
                    let new_value = format!(
                        "rgba({:02X}{:02X}{:02X}{:02X})",
                        (new_color.red() * 255.0) as u8,
                        (new_color.green() * 255.0) as u8,
                        (new_color.blue() * 255.0) as u8,
                        (new_color.alpha() * 255.0) as u8
                    );
                    changes.insert((category.clone(), name.clone()), new_value);
                });
            } else if let Some(dropdown) = widget.downcast_ref::<DropDown>() {
                let is_numeric = value.parse::<u32>().is_ok();

                if is_numeric {
                    let index: u32 = value.parse().unwrap();
                    dropdown.set_selected(index);
                } else {
                    let model = dropdown.model().unwrap();
                    for i in 0..model.n_items() {
                        if let Some(item) = model.item(i)
                            && let Some(string_object) = item.downcast_ref::<StringObject>()
                            && string_object.string() == value
                        {
                            dropdown.set_selected(i);
                            break;
                        }
                    }
                }

                let category = category.to_string();
                let name = name.to_string();
                let changed_options = changed_options.clone();

                dropdown.connect_selected_notify(move |dd| {
                    let mut changes = changed_options.borrow_mut();

                    if is_numeric {
                        let selected_index = dd.selected();
                        changes
                            .insert((category.clone(), name.clone()), selected_index.to_string());
                    } else if let Some(selected) = dd.selected_item()
                        && let Some(string_object) = selected.downcast_ref::<StringObject>()
                    {
                        let new_value = string_object.string().to_string();
                        changes.insert((category.clone(), name.clone()), new_value);
                    }
                });
            } else if let Some(gtkbox) = widget.downcast_ref::<Box>() {
                let read_only_str = &t!(
                    "this_is_a_read_only__from_main_config",
                    category_name = name
                );

                let read_only_label = Label::new(Some(read_only_str));
                read_only_label.set_halign(gtk::Align::Start);
                read_only_label.set_markup(&format!("<b>{read_only_str}</b>"));

                gtkbox.append(&read_only_label);

                let frame = Frame::new(None);
                frame.set_margin_top(10);

                gtkbox.append(&frame);

                let read_only_path = get_config_path(false, profile);
                let rw_path = get_config_path(true, profile);

                let profile_path = if profile == "Default" {
                    "./hyprviz.conf".to_string()
                } else {
                    format!("./hyprviz/{}.conf", profile)
                };

                let read_only_config_raw = match fs::read_to_string(&read_only_path) {
                    Ok(read_only_config) => read_only_config
                        .lines()
                        .filter(|line| {
                            !line
                                .trim_start()
                                .starts_with(&format!("source = {}", profile_path))
                        })
                        .filter(|line| {
                            !line
                                .trim_start()
                                .starts_with(&format!("source ={}", profile_path))
                        })
                        .filter(|line| {
                            !line
                                .trim_start()
                                .contains(&format!("source= {}", profile_path))
                        })
                        .filter(|line| {
                            !line
                                .trim_start()
                                .starts_with(&format!("source={}", profile_path))
                        })
                        .collect::<Vec<&str>>()
                        .join("\n"),
                    Err(_) => {
                        let error_label = Label::new(Some(&t!(
                            "error_reading_",
                            path = read_only_path.to_string_lossy()
                        )));

                        error_label.set_markup("<span foreground=\"red\">{read_only_path}</span>");
                        error_label.set_margin_top(5);
                        error_label.set_margin_bottom(5);
                        error_label.set_margin_start(5);
                        error_label.set_margin_end(5);

                        gtkbox.append(&error_label);

                        String::new()
                    }
                };

                let read_only_config =
                    match expand_source_str(&read_only_path, &read_only_config_raw) {
                        Ok(read_only_config) => read_only_config,
                        Err(_) => {
                            let error_label = Label::new(Some(&t!(
                                "error_reading_",
                                path = read_only_path.to_string_lossy()
                            )));

                            error_label
                                .set_markup("<span foreground=\"red\">{read_only_path}</span>");
                            error_label.set_margin_top(5);
                            error_label.set_margin_bottom(5);
                            error_label.set_margin_start(5);
                            error_label.set_margin_end(5);

                            gtkbox.append(&error_label);

                            String::new()
                        }
                    };

                let parsed_headless_readonly_options =
                    parse_top_level_options(&read_only_config, false);

                let options_grid = Grid::new();
                options_grid.set_column_spacing(10);
                options_grid.set_row_spacing(5);
                options_grid.set_margin_top(10);
                options_grid.set_margin_bottom(10);
                options_grid.set_margin_start(5);
                options_grid.set_margin_end(5);

                for (row_num, (name, value)) in parsed_headless_readonly_options.iter().enumerate()
                {
                    if !name.starts_with(category)
                        && category != "top_level"
                        && !((category == "bind" && name.starts_with("unbind"))
                            || (category == "animation" && name.starts_with("bezier")))
                    {
                        continue;
                    }

                    let name_label = Label::new(Some(name));
                    name_label.set_xalign(0.0);
                    name_label.set_size_request(name_label.width(), 1);
                    name_label.set_selectable(true);

                    let equals_label = Label::new(Some("="));
                    equals_label.set_xalign(0.5);
                    equals_label.set_markup("<b>=</b>");

                    let value_label = Label::new(Some(value));
                    value_label.set_xalign(0.0);
                    value_label.set_selectable(true);
                    value_label.set_wrap(true);

                    let row_num = row_num as i32;

                    options_grid.attach(&name_label, 0, row_num, 1, 1);
                    options_grid.attach(&equals_label, 1, row_num, 1, 1);
                    options_grid.attach(&value_label, 2, row_num, 1, 1);
                }

                let expander = Expander::new(Some(&t!("show_read_only_options")));
                expander.set_margin_top(10);
                expander.set_margin_bottom(10);
                expander.set_margin_start(5);
                expander.set_margin_end(5);

                expander.set_child(Some(&options_grid));

                gtkbox.append(&expander);

                let rw_str = &t!(
                    "this_is_a_read_write__from_your_profile",
                    category_name = name
                );

                let rw_label = Label::new(Some(rw_str));
                rw_label.set_halign(gtk::Align::Start);
                rw_label.set_margin_top(10);
                rw_label.set_markup(&format!("<b>{rw_str}</b>"));

                gtkbox.append(&rw_label);

                let frame = Frame::new(None);
                frame.set_margin_top(10);

                gtkbox.append(&frame);

                let rw_config = match expand_source(&rw_path) {
                    Ok(rw_config) => rw_config,
                    Err(_) => {
                        let error_label = Label::new(Some(&t!(
                            "error_reading_",
                            path = rw_path.to_string_lossy()
                        )));

                        error_label.set_markup("<span foreground=\"red\">{rw_path}</span>");
                        error_label.set_margin_top(5);
                        error_label.set_margin_bottom(5);
                        error_label.set_margin_start(5);
                        error_label.set_margin_end(5);

                        gtkbox.append(&error_label);

                        String::new()
                    }
                };

                let create_button = Button::with_label(&t!("create"));

                create_button.set_margin_top(10);
                create_button.set_margin_bottom(10);
                create_button.set_margin_start(5);
                create_button.set_margin_end(5);
                create_button.set_width_request(256);

                let id_new = Rc::new(RefCell::new(0));

                let window_clone = window.clone();
                let gtkbox_clone = gtkbox.clone();

                let changed_options_clone = changed_options.clone();

                let category_string = category.to_string();

                create_button.connect_clicked(move |_| {
                    let mut id = id_new.borrow_mut();
                    append_option_row(
                        &window_clone,
                        &gtkbox_clone,
                        id.to_string(),
                        "".to_string(),
                        "".to_string(),
                        &changed_options_clone,
                        &category_string,
                    );
                    *id += 1;
                });

                gtkbox.append(&create_button);

                let parsed_headless_options_raw = parse_top_level_options(&rw_config, true);
                let parsed_headless_options = parse_top_level_options(&rw_config, false);

                for ((raw, _), (name, value)) in parsed_headless_options_raw
                    .into_iter()
                    .zip(parsed_headless_options)
                {
                    if name.starts_with(category) || category == "top_level" {
                        append_option_row(
                            window,
                            gtkbox,
                            raw,
                            name,
                            value,
                            &changed_options,
                            category,
                        );
                        continue;
                    }

                    if (category == "bind" && (name.starts_with("unbind")))
                        || (category == "animation" && (name.starts_with("bezier")))
                    {
                        append_option_row(
                            window,
                            gtkbox,
                            raw,
                            name,
                            value,
                            &changed_options,
                            category,
                        );
                    }
                    continue;
                }
            }
        }
    }
}
