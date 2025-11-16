use crate::utils::{
    Cm, MAX_SAFE_INTEGER_F64, MAX_SAFE_STEP_0_01_F64, MIN_SAFE_INTEGER_F64, Monitor, MonitorState,
    Position, Scale, after_second_comma, get_available_resolutions_for_monitor, is_modifier,
    keycode_to_en_key, parse_animation, parse_bezier, parse_coordinates, parse_monitor,
};
use core::f64;
use gio::glib::SignalHandlerId;
use gtk::{
    Align, ApplicationWindow, Box, Button, DrawingArea, DropDown, Entry, EventControllerKey,
    EventControllerMotion, GestureClick, Label, Orientation, SpinButton, StringList, StringObject,
    Switch, TextBuffer, TextView, prelude::*,
};
use rust_i18n::t;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
    str::FromStr,
};

const SIZE: f64 = 300.0;

#[derive(Clone, Copy)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

pub struct BezierCurve {
    points: [Point; 4],
    dragging_point_index: Option<usize>,
}

impl BezierCurve {
    fn new() -> Self {
        Self {
            points: [
                Point::new(0.0, SIZE + (SIZE / 3.0)),
                Point::new(SIZE / 3.0, SIZE),
                Point::new(SIZE / 3.0 * 2.0, SIZE / 3.0 * 2.0),
                Point::new(SIZE, SIZE / 3.0),
            ],
            dragging_point_index: None,
        }
    }

    fn new_from_points(points: [Point; 2]) -> Self {
        Self {
            points: [
                Point::new(0.0, SIZE + (SIZE / 3.0)),
                points[0],
                points[1],
                Point::new(SIZE, SIZE / 3.0),
            ],
            dragging_point_index: None,
        }
    }

    fn point_at(&self, index: usize) -> Point {
        self.points[index]
    }

    /// Set point at index to point (if it's C0 or C1)
    fn set_point(&mut self, index: usize, point: Point) {
        if index == 1 || index == 2 {
            let new_x = point.x.clamp(0.0, SIZE);
            let new_y = point.y.clamp(0.0, SIZE / 3.0 * 5.0);
            self.points[index] = Point::new(new_x, new_y);
        }
    }

    fn is_point_near(&self, x: f64, y: f64, index: usize, tolerance: f64) -> bool {
        if index != 1 && index != 2 {
            return false;
        }
        let p = self.point_at(index);
        let dx = p.x - x;
        let dy = p.y - y;
        dx * dx + dy * dy <= tolerance * tolerance
    }

    fn set_points_from_coordinates(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.points[1].x = x0;
        self.points[1].y = y0;
        self.points[2].x = x1;
        self.points[2].y = y1;
    }

    fn find_point_index_at(&self, x: f64, y: f64, tolerance: f64) -> Option<usize> {
        (1..3).find(|&i| self.is_point_near(x, y, i, tolerance))
    }
}

pub fn create_curve_editor(value_entry: &Entry) -> (Box, Button) {
    let bezier = match parse_coordinates(&value_entry.text()) {
        (_, Ok((c0_x, c0_y, c1_x, c1_y))) => Rc::new(RefCell::new(BezierCurve::new_from_points([
            Point::new(c0_x * SIZE, 100.0 + SIZE * (1.0 - c0_y)),
            Point::new(c1_x * SIZE, 100.0 + SIZE * (1.0 - c1_y)),
        ]))),
        (_, Err(_)) => Rc::new(RefCell::new(BezierCurve::new())),
    };

    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .visible(false)
        .build();

    let toggle_button = Button::with_label(&t!("show_editor"));

    let vbox_clone = vbox.clone();
    let button_clone = toggle_button.clone();
    let show_editor = t!("show_editor");
    let hide_editor = t!("hide_editor");
    toggle_button.connect_clicked(move |_| {
        let is_visible = vbox_clone.is_visible();
        vbox_clone.set_visible(!is_visible);
        button_clone.set_label(if is_visible {
            &show_editor
        } else {
            &hide_editor
        });
    });

    let drawing_area = DrawingArea::builder()
        .hexpand(false)
        .vexpand(false)
        .halign(Align::Center)
        .valign(Align::Start)
        .build();

    drawing_area.set_size_request(SIZE as i32, (SIZE / 3.0 * 5.0) as i32);

    let bezier_clone = bezier.clone();
    let da_clone = drawing_area.clone();
    value_entry.connect_changed(move |entry| {
        if let (_, Ok((x0, y0, x1, y1))) = parse_coordinates(&entry.text()) {
            bezier_clone.borrow_mut().set_points_from_coordinates(
                x0 * SIZE,
                100.0 + SIZE * (1.0 - y0),
                x1 * SIZE,
                100.0 + SIZE * (1.0 - y1),
            );
        }
        da_clone.queue_draw();
    });

    let bezier_clone = bezier.clone();
    drawing_area.set_draw_func(move |widget, cr, _width, _height| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().unwrap();

        let scale_factor = widget.scale_factor() as f64;
        cr.scale(scale_factor, scale_factor);

        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(2.0 / scale_factor);
        cr.move_to(0.0, SIZE / 3.0);
        cr.line_to(SIZE, SIZE / 3.0);
        cr.move_to(0.0, SIZE / 3.0 * 4.0);
        cr.line_to(SIZE, SIZE / 3.0 * 4.0);
        cr.stroke().unwrap();

        cr.set_source_rgba(0.8, 0.8, 0.8, 0.6);
        cr.set_line_width(1.0 / scale_factor);

        let grid_step = SIZE / 8.0;

        for i in 0..=8 {
            let x = i as f64 * grid_step;
            cr.move_to(x, 0.0);
            cr.line_to(x, 5.0 * SIZE / 3.0);
        }

        for i in -2..=10 {
            let y = SIZE / 3.0 + i as f64 * grid_step;
            cr.move_to(0.0, y);
            cr.line_to(SIZE, y);
        }

        cr.stroke().unwrap();

        let bez = bezier_clone.borrow();

        cr.set_source_rgba(0.7, 0.7, 0.7, 0.5);
        cr.move_to(bez.point_at(0).x, bez.point_at(0).y);
        cr.line_to(bez.point_at(1).x, bez.point_at(1).y);
        cr.move_to(bez.point_at(2).x, bez.point_at(2).y);
        cr.line_to(bez.point_at(3).x, bez.point_at(3).y);
        cr.stroke().unwrap();

        cr.set_source_rgb(0.15, 0.15, 0.99);
        cr.move_to(bez.point_at(0).x, bez.point_at(0).y);
        cr.curve_to(
            bez.point_at(1).x,
            bez.point_at(1).y,
            bez.point_at(2).x,
            bez.point_at(2).y,
            bez.point_at(3).x,
            bez.point_at(3).y,
        );
        cr.set_line_width(2.0 / scale_factor);
        cr.stroke().unwrap();

        for (i, p) in bez.points.iter().enumerate() {
            match i {
                0 | 3 => cr.set_source_rgb(0.0, 1.0, 0.0),
                1 | 2 => cr.set_source_rgb(1.0, 0.0, 0.0),
                _ => {}
            }
            cr.arc(
                p.x,
                p.y,
                6.0 / scale_factor,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            cr.fill().unwrap();
        }
    });

    let click_gesture = GestureClick::builder().button(0).build();

    let da_clone = drawing_area.clone();
    let bezier_clone = bezier.clone();
    click_gesture.connect_pressed(move |_gesture, _n_press, x, y| {
        let scale_factor = da_clone.scale_factor() as f64;
        let x_scaled = x / scale_factor;
        let y_scaled = y / scale_factor;

        let mut bez = bezier_clone.borrow_mut();
        if let Some(index) = bez.find_point_index_at(x_scaled, y_scaled, 10.0) {
            bez.dragging_point_index = Some(index);
        }
    });

    let bezier_clone = bezier.clone();
    click_gesture.connect_released(move |_gesture, _n_press, _x, _y| {
        let mut bez = bezier_clone.borrow_mut();
        bez.dragging_point_index = None;
    });

    drawing_area.add_controller(click_gesture);

    let update_value_entry_from_coords =
        |entry: &Entry, c0_x: f64, c0_y: f64, c1_x: f64, c1_y: f64| {
            let (name, _) = parse_coordinates(entry.text().as_str());
            let new_value = format!(
                "{}, {:.3}, {:.3}, {:.3}, {:.3}",
                name,
                c0_x / SIZE,
                1.0 - ((c0_y - 100.0) / SIZE),
                c1_x / SIZE,
                1.0 - ((c1_y - 100.0) / SIZE)
            );
            entry.set_text(&new_value);
        };

    let motion_controller = EventControllerMotion::new();
    let bezier_clone = bezier.clone();
    let da_clone = drawing_area.clone();
    let value_entry_clone = value_entry.clone();
    motion_controller.connect_motion(move |_, x, y| {
        let (c0_x, c0_y, c1_x, c1_y) = {
            let bez = bezier_clone.borrow();
            (
                bez.points[1].x,
                bez.points[1].y,
                bez.points[2].x,
                bez.points[2].y,
            )
        };

        update_value_entry_from_coords(&value_entry_clone, c0_x, c0_y, c1_x, c1_y);

        let scale_factor = da_clone.scale_factor() as f64;
        let x_scaled = x / scale_factor;
        let y_scaled = y / scale_factor;

        let mut bez = bezier_clone.borrow_mut();
        if let Some(index) = bez.dragging_point_index
            && (index == 1 || index == 2)
        {
            let new_x = x_scaled.clamp(0.0, SIZE);
            let new_y = y_scaled.clamp(0.0, SIZE / 3.0 * 5.0);
            bez.set_point(index, Point::new(new_x, new_y));
            da_clone.queue_draw();
        }
    });

    drawing_area.add_controller(motion_controller);

    vbox.append(&drawing_area);

    (vbox, toggle_button)
}

pub fn create_bind_editor(window: &ApplicationWindow, value_entry: &Entry) -> (Box, Button) {
    let container = Box::new(Orientation::Vertical, 5);

    let toggle_button = Button::with_label(&t!("record_bind"));

    let is_recording = Rc::new(RefCell::new(false));

    let is_recording_clone = Rc::clone(&is_recording);
    let window_clone = window.clone();
    let value_entry_clone = value_entry.clone();
    let container_clone = container.clone();

    let toggle_button_clone = toggle_button.clone();

    let key_controller = EventControllerKey::new();
    let key_controller_handlers_ids: Rc<RefCell<Vec<SignalHandlerId>>> =
        Rc::new(RefCell::new(Vec::new()));

    toggle_button.connect_clicked(move |_| {
        let mut is_recording_mut = is_recording_clone.borrow_mut();

        if *is_recording_mut {
            *is_recording_mut = false;

            while let Some(child) = container_clone.first_child() {
                container_clone.remove(&child);
            }

            window_clone.remove_controller(&key_controller);

            for handler_id in key_controller_handlers_ids.replace(Vec::new()) {
                key_controller.disconnect(handler_id);
            }

            key_controller_handlers_ids.borrow_mut().clear();

            toggle_button_clone.set_label(&t!("record_bind"));
        } else {
            *is_recording_mut = true;

            let vbox = Box::new(Orientation::Vertical, 5);

            let text_view = TextView::new();
            text_view.set_vexpand(true);
            text_view.set_editable(false);
            text_view.set_cursor_visible(false);

            vbox.append(&text_view);

            let buffer = text_view.buffer();

            let active_inputs: Rc<RefCell<HashMap<u32, String>>> =
                Rc::new(RefCell::new(HashMap::new()));

            container_clone.append(&vbox);

            let active_inputs_clone = Rc::clone(&active_inputs);
            let buffer_clone = buffer.clone();
            let value_entry_clone_clone = value_entry_clone.clone();
            let toggle_button_clone_clone = toggle_button_clone.clone();

            key_controller_handlers_ids
                .borrow_mut()
                .push(
                    key_controller.connect_key_pressed(move |_, _keyval, keycode, _state| {
                        let key_str = keycode_to_en_key(keycode);
                        active_inputs_clone.borrow_mut().insert(keycode, key_str);
                        update_display(
                            &active_inputs_clone,
                            &buffer_clone,
                            &value_entry_clone_clone,
                        );
                        true.into()
                    }),
                );

            let active_inputs_clone = Rc::clone(&active_inputs);
            let buffer_clone = buffer.clone();
            let value_entry_clone_clone = value_entry_clone.clone();
            key_controller_handlers_ids
                .borrow_mut()
                .push(
                    key_controller.connect_key_released(move |_, _keyval, keycode, _state| {
                        active_inputs_clone.borrow_mut().remove(&keycode);
                        update_display(
                            &active_inputs_clone,
                            &buffer_clone,
                            &value_entry_clone_clone,
                        );
                    }),
                );

            let key_controller_clone = key_controller.clone();

            window_clone.add_controller(key_controller_clone);

            update_display(&active_inputs, &buffer, &value_entry_clone);

            toggle_button_clone_clone.set_label(&t!("stop_recording"));
        }
    });

    (container, toggle_button)
}

fn update_display(
    active_inputs: &RefCell<HashMap<u32, String>>,
    buffer: &TextBuffer,
    value_entry: &Entry,
) {
    let value_entry_text = value_entry.text();
    let bind_action = after_second_comma(value_entry_text.as_str());

    let inputs = active_inputs.borrow();
    buffer.set_text("");
    let mut end_iter = buffer.end_iter();

    let mut modifiers = HashSet::new();
    let mut regular_keys = Vec::new();

    for (_, key_str) in inputs.iter() {
        if is_modifier(key_str) {
            modifiers.insert(key_str.as_str());
        } else {
            regular_keys.push(key_str.clone());
        }
    }

    let modifier_priority = |modifier: &&str| -> usize {
        match *modifier {
            "SHIFT" => 0,
            "CAPS" => 1,
            "CTRL" => 2,
            "ALT" => 3,
            "MOD2" => 4,
            "MOD3" => 5,
            "SUPER" => 6,
            "MOD5" => 7,
            _ => 8,
        }
    };

    if !modifiers.is_empty() && !regular_keys.is_empty() {
        buffer.insert(&mut end_iter, &t!("active_combinations"));

        let mut modifiers_vec: Vec<&str> = modifiers.into_iter().collect();
        modifiers_vec.sort_by_key(|&modifier| modifier_priority(&modifier));
        let modifiers_str = modifiers_vec.join(" + ");

        for key in &regular_keys {
            value_entry.set_text(&format!("{}, {}{}", modifiers_str, key, bind_action));
            buffer.insert(&mut end_iter, &format!("  {} + {}\n", modifiers_str, key));
        }
    } else if !modifiers.is_empty() {
        buffer.insert(&mut end_iter, &t!("active_modifiers"));

        let mut modifiers_vec: Vec<&str> = modifiers.into_iter().collect();
        modifiers_vec.sort_by_key(|&modifier| modifier_priority(&modifier));
        let modifiers_str = modifiers_vec.join(" + ");

        for modifier in modifiers_vec {
            value_entry.set_text(&format!("{}, {}", modifiers_str, bind_action));
            buffer.insert(&mut end_iter, &format!("  {}\n", modifier));
        }
    } else if !regular_keys.is_empty() {
        buffer.insert(&mut end_iter, &t!("active_keys"));

        for key in &regular_keys {
            value_entry.set_text(&format!(", {}{}", key, bind_action));
            buffer.insert(&mut end_iter, &format!("  {}\n", key));
        }
    } else {
        buffer.insert(&mut end_iter, &t!("no_active_inputs"));
    }
}

pub fn create_fancy_boxline(category: &str, name_entry: &Entry, value_entry: &Entry) -> Box {
    let fancy_boxline = Box::new(Orientation::Horizontal, 5);

    let fancy_name_entry = Box::new(Orientation::Horizontal, 5);
    match category {
        "monitor" => {
            let label = Label::new(Some("monitor"));
            label.set_width_request(100);
            label.set_selectable(true);
            fancy_name_entry.append(&label);
            name_entry.set_text("monitor");
            name_entry.connect_changed(move |entry| {
                label.set_text(&entry.text());
            });
        }
        "animation" => {
            let string_list = StringList::new(&["animation", "bezier"]);
            let dropdown = create_dropdown(&string_list);
            dropdown.set_width_request(100);

            let name_entry_clone = name_entry.clone();
            dropdown.connect_selected_notify(move |dd| {
                if let Some(selected) = dd.selected_item()
                    && let Some(string_object) = selected.downcast_ref::<StringObject>()
                {
                    let new_name = string_object.string().to_string();
                    name_entry_clone.set_text(&new_name);
                }
            });
            let dropdown_clone = dropdown.clone();
            name_entry.connect_changed(move |entry| {
                let new_name = entry.text().to_string();

                for idx in 0..string_list.n_items() {
                    if let Some(item) = string_list.item(idx) {
                        let item_str = item.property::<String>("string");

                        if item_str == new_name {
                            dropdown_clone.set_selected(idx);
                            break;
                        }
                    }
                }
            });
            fancy_name_entry.append(&dropdown);
        }
        "bind" => {
            todo!()
        }
        _ => {}
    }

    fancy_boxline.append(&fancy_name_entry);

    let equals_label = Label::new(Some("="));
    equals_label.set_xalign(0.5);
    fancy_boxline.append(&equals_label);

    let fancy_value_entry = Box::new(Orientation::Horizontal, 5);

    fill_fancy_value_entry(
        &fancy_value_entry,
        value_entry,
        category,
        &name_entry.text(),
    );
    fancy_boxline.append(&fancy_value_entry);

    let fancy_value_entry_clone = fancy_value_entry.clone();
    let value_entry_clone = value_entry.clone();
    let category_clone = category.to_string();
    name_entry.connect_changed(move |entry| {
        while let Some(child) = fancy_value_entry.first_child() {
            fancy_value_entry.remove(&child);
        }
        let new_name = entry.text().to_string();
        fill_fancy_value_entry(
            &fancy_value_entry_clone,
            &value_entry_clone,
            &category_clone,
            &new_name,
        );
    });

    fancy_boxline
}

macro_rules! widget_connector {
    (
        $is_updating:expr,
        $value_entry:expr,
        $widget:expr,
        $connect_method:ident,
        $param_name:ident,
        $extractor:expr,
        $parser:ident,
        $formatter:expr
    ) => {{
        let is_updating_clone = $is_updating.clone();
        let value_entry_clone = $value_entry.clone();

        $widget.$connect_method(move |$param_name| {
            if is_updating_clone.get() {
                return;
            }

            is_updating_clone.set(true);

            let new_value = $extractor;
            let parsed_values = $parser(&value_entry_clone.text());
            let new_value_str = $formatter(parsed_values, new_value);

            value_entry_clone.set_text(&new_value_str);
            is_updating_clone.set(false);
        });
    }};

    (
        $is_updating:expr,
        $value_entry:expr,
        $widget:expr,
        $connect_method:ident,
        $param1:ident,
        $param2:ident,
        $extractor:expr,
        $parser:ident,
        $formatter:expr
    ) => {{
        let is_updating_clone = $is_updating.clone();
        let value_entry_clone = $value_entry.clone();

        $widget.$connect_method(move |$param1, $param2| {
            if is_updating_clone.get() {
                return false.into();
            }

            is_updating_clone.set(true);

            let new_value = $extractor;
            let parsed_values = $parser(&value_entry_clone.text());
            let new_value_str = $formatter(parsed_values, new_value);

            value_entry_clone.set_text(&new_value_str);
            is_updating_clone.set(false);

            false.into()
        });
    }};
}

macro_rules! optional_widget_connector {
    (
        $is_updating:expr,
        $value_entry:expr,
        $onoff_switch:expr,
        $widget:expr,
        $connect_widget_method:ident,
        $widget_param:ident,
        $widget_extractor:expr,
        $parser:ident,
        $formatter_switch:expr,
        $formatter_widget:expr
    ) => {{
        let widget_clone = $widget.clone();
        let value_entry_clone_for_switch = $value_entry.clone();
        let is_updating_clone_for_switch = $is_updating.clone();
        let $widget_param = $widget.clone();

        widget_connector!(
            is_updating_clone_for_switch,
            value_entry_clone_for_switch,
            $onoff_switch,
            connect_state_set,
            _switch_widget,
            state,
            state,
            $parser,
            |parsed_values, new_onoff: bool| {
                if new_onoff {
                    widget_clone.set_visible(true);
                } else {
                    widget_clone.set_visible(false);
                }
                $formatter_switch(parsed_values, new_onoff, $widget_extractor)
            }
        );
    }
    {
        let onoff_switch_clone = $onoff_switch.clone();
        let value_entry_clone_for_widget = $value_entry.clone();
        let is_updating_clone_for_widget = $is_updating.clone();

        widget_connector!(
            is_updating_clone_for_widget,
            value_entry_clone_for_widget,
            $widget,
            $connect_widget_method,
            $widget_param,
            $widget_extractor,
            $parser,
            |parsed_values, new_value| {
                if onoff_switch_clone.is_active() {
                    $formatter_widget(parsed_values, new_value)
                } else {
                    value_entry_clone_for_widget.text().to_string()
                }
            }
        );
    }};
}
fn create_entry() -> Entry {
    Entry::builder()
        .width_request(100)
        .hexpand(true)
        .halign(Align::Fill)
        .margin_top(5)
        .margin_bottom(5)
        .vexpand(false)
        .valign(Align::Center)
        .build()
}

fn create_spin_button(min: f64, max: f64, step: f64) -> SpinButton {
    let spin_button = SpinButton::with_range(min, max, step);
    spin_button.set_width_request(100);
    spin_button.set_hexpand(true);
    spin_button.set_halign(Align::Fill);
    spin_button.set_margin_top(5);
    spin_button.set_margin_bottom(5);
    spin_button.set_vexpand(false);
    spin_button.set_valign(Align::Center);
    spin_button
}

fn create_dropdown(string_list: &StringList) -> DropDown {
    let dropdown = DropDown::new(Some(string_list.clone()), None::<gtk::Expression>);
    dropdown.set_width_request(100);
    dropdown.set_hexpand(true);
    dropdown.set_halign(Align::Fill);
    dropdown.set_margin_top(5);
    dropdown.set_margin_bottom(5);
    dropdown.set_vexpand(false);
    dropdown.set_valign(Align::Center);
    dropdown
}

fn create_switch() -> Switch {
    Switch::builder()
        .width_request(50)
        .hexpand(false)
        .halign(Align::Center)
        .margin_top(5)
        .margin_bottom(5)
        .vexpand(false)
        .valign(Align::Center)
        .build()
}

fn fill_fancy_value_entry(
    fancy_value_entry: &Box,
    value_entry: &Entry,
    category: &str,
    name: &str,
) {
    let is_updating = Rc::new(Cell::new(false));

    match category {
        "monitor" => {
            let (monitor_name, monitor) = parse_monitor(&value_entry.text());

            let name_box = Box::new(Orientation::Horizontal, 5);
            name_box.append(&Label::new(Some(&t!("name"))));
            let monitor_name_entry = create_entry();
            monitor_name_entry.set_text(&monitor_name);
            name_box.append(&monitor_name_entry);
            fancy_value_entry.append(&name_box);

            let resolution_box = Box::new(Orientation::Horizontal, 5);
            resolution_box.append(&Label::new(Some(&t!("resolution/mode"))));
            let resolution_string_list = StringList::new(
                &get_available_resolutions_for_monitor(&monitor_name)
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>(),
            );
            let monitor_resolution_dropdown = create_dropdown(&resolution_string_list);
            resolution_box.append(&monitor_resolution_dropdown);
            fancy_value_entry.append(&resolution_box);

            let enabled_box = Box::new(Orientation::Vertical, 5);

            let position_box = Box::new(Orientation::Horizontal, 5);
            position_box.append(&Label::new(Some(&t!("position"))));
            let position_str_list = Position::get_list();
            let position_string_list =
                StringList::new(&Position::get_fancy_list().each_ref().map(|s| s.as_str()));
            let monitor_position_dropdown = create_dropdown(&position_string_list);
            position_box.append(&monitor_position_dropdown);
            let monitor_position_x_spin =
                create_spin_button(MIN_SAFE_INTEGER_F64, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_position_x_spin.set_value(0.0);
            monitor_position_x_spin.set_digits(0);
            position_box.append(&monitor_position_x_spin);
            let monitor_position_y_spin =
                create_spin_button(MIN_SAFE_INTEGER_F64, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_position_y_spin.set_value(0.0);
            monitor_position_y_spin.set_digits(0);
            position_box.append(&monitor_position_y_spin);
            enabled_box.append(&position_box);

            let monitor_scale_box = Box::new(Orientation::Horizontal, 5);
            monitor_scale_box.append(&Label::new(Some(&t!("scale"))));
            let scale_string_list =
                StringList::new(&Scale::get_fancy_list().each_ref().map(|s| s.as_str()));
            let monitor_scale_dropdown = create_dropdown(&scale_string_list);
            monitor_scale_box.append(&monitor_scale_dropdown);
            let monitor_scale_spin = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
            monitor_scale_spin.set_digits(2);
            monitor_scale_spin.set_value(1.0);
            monitor_scale_box.append(&monitor_scale_spin);
            enabled_box.append(&monitor_scale_box);

            let monitor_mirror_box = Box::new(Orientation::Horizontal, 5);
            monitor_mirror_box.append(&Label::new(Some(&t!("mirror"))));
            let monitor_mirror_onoff_switch = create_switch();
            monitor_mirror_box.append(&monitor_mirror_onoff_switch);
            let monitor_mirror_entry = create_entry();
            monitor_mirror_box.append(&monitor_mirror_entry);
            enabled_box.append(&monitor_mirror_box);

            let monitor_bitdepth_box = Box::new(Orientation::Horizontal, 5);
            monitor_bitdepth_box.append(&Label::new(Some(&t!("bitdepth"))));
            let monitor_bitdepth_onoff_switch = create_switch();
            monitor_bitdepth_box.append(&monitor_bitdepth_onoff_switch);
            let monitor_bitdepth_spin = create_spin_button(1.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_bitdepth_spin.set_value(10.0);
            monitor_bitdepth_box.append(&monitor_bitdepth_spin);
            enabled_box.append(&monitor_bitdepth_box);

            let monitor_cm_box = Box::new(Orientation::Horizontal, 5);
            monitor_cm_box.append(&Label::new(Some(&t!("color_management"))));
            let monitor_cm_onoff_switch = create_switch();
            monitor_cm_box.append(&monitor_cm_onoff_switch);
            let cm_string_list = StringList::new(&Cm::get_fancy_list());
            let monitor_cm_dropdown = create_dropdown(&cm_string_list);
            monitor_cm_box.append(&monitor_cm_dropdown);
            enabled_box.append(&monitor_cm_box);

            let monitor_sdrbrightness_box = Box::new(Orientation::Horizontal, 5);
            monitor_sdrbrightness_box.append(&Label::new(Some(&t!("sdr_brightness"))));
            let monitor_sdrbrightness_onoff_switch = create_switch();
            monitor_sdrbrightness_box.append(&monitor_sdrbrightness_onoff_switch);
            let monitor_sdrbrightness_spin = create_spin_button(0.0, f64::MAX, 0.01);
            monitor_sdrbrightness_spin.set_value(1.0);
            monitor_sdrbrightness_box.append(&monitor_sdrbrightness_spin);
            enabled_box.append(&monitor_sdrbrightness_box);

            let monitor_sdrsaturation_box = Box::new(Orientation::Horizontal, 5);
            monitor_sdrsaturation_box.append(&Label::new(Some(&t!("sdr_saturation"))));
            let monitor_sdrsaturation_onoff_switch = create_switch();
            monitor_sdrsaturation_box.append(&monitor_sdrsaturation_onoff_switch);
            let monitor_sdrsaturation_spin = create_spin_button(0.0, f64::MAX, 0.01);
            monitor_sdrsaturation_spin.set_value(1.0);
            monitor_sdrsaturation_box.append(&monitor_sdrsaturation_spin);
            enabled_box.append(&monitor_sdrsaturation_box);

            let monitor_vrr_box = Box::new(Orientation::Horizontal, 5);
            monitor_vrr_box.append(&Label::new(Some(&t!("vrr"))));
            let monitor_vrr_onoff_switch = create_switch();
            monitor_vrr_box.append(&monitor_vrr_onoff_switch);
            let vrr_string_list = StringList::new(&[
                &t!("misc_category.vrr_off"),
                &t!("misc_category.vrr_on"),
                &t!("misc_category.vrr_fullscreen_only"),
                &t!("misc_category.vrr_fullscreen_with_video/game"),
            ]);
            let monitor_vrr_dropdown = create_dropdown(&vrr_string_list);
            monitor_vrr_box.append(&monitor_vrr_dropdown);
            enabled_box.append(&monitor_vrr_box);

            let monitor_transform_box = Box::new(Orientation::Horizontal, 5);
            monitor_transform_box.append(&Label::new(Some(&t!("transform"))));
            let monitor_transform_onoff_switch = create_switch();
            monitor_transform_box.append(&monitor_transform_onoff_switch);
            let transform_string_list = StringList::new(&[
                &t!("normal"),
                &t!("rotate_90"),
                &t!("rotate_180"),
                &t!("rotate_270"),
                &t!("flip"),
                &t!("flip_rotate_90"),
                &t!("flip_rotate_180"),
                &t!("flip_rotate_270"),
            ]);
            let monitor_transform_dropdown = create_dropdown(&transform_string_list);
            monitor_transform_box.append(&monitor_transform_dropdown);
            enabled_box.append(&monitor_transform_box);

            fancy_value_entry.append(&enabled_box);

            let monitor_addreserved_box = Box::new(Orientation::Vertical, 5);

            monitor_addreserved_box.append(&Label::new(Some(&t!("top"))));
            let monitor_addreserved_up_spin = create_spin_button(0.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_addreserved_box.append(&monitor_addreserved_up_spin);

            monitor_addreserved_box.append(&Label::new(Some(&t!("bottom"))));
            let monitor_addreserved_down_spin = create_spin_button(0.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_addreserved_box.append(&monitor_addreserved_down_spin);

            monitor_addreserved_box.append(&Label::new(Some(&t!("left"))));
            let monitor_addreserved_left_spin = create_spin_button(0.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_addreserved_box.append(&monitor_addreserved_left_spin);

            monitor_addreserved_box.append(&Label::new(Some(&t!("right"))));
            let monitor_addreserved_right_spin = create_spin_button(0.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_addreserved_box.append(&monitor_addreserved_right_spin);

            fancy_value_entry.append(&monitor_addreserved_box);

            let monitor_name_entry_clone = monitor_name_entry.clone();
            let monitor_resolution_dropdown_clone = monitor_resolution_dropdown.clone();
            let monitor_position_dropdown_clone = monitor_position_dropdown.clone();
            let monitor_position_x_spin_clone = monitor_position_x_spin.clone();
            let monitor_position_y_spin_clone = monitor_position_y_spin.clone();
            let monitor_scale_dropdown_clone = monitor_scale_dropdown.clone();
            let monitor_scale_spin_clone = monitor_scale_spin.clone();
            let monitor_mirror_onoff_switch_clone = monitor_mirror_onoff_switch.clone();
            let monitor_mirror_entry_clone = monitor_mirror_entry.clone();
            let monitor_bitdepth_onoff_switch_clone = monitor_bitdepth_onoff_switch.clone();
            let monitor_bitdepth_spin_clone = monitor_bitdepth_spin.clone();
            let monitor_cm_onoff_switch_clone = monitor_cm_onoff_switch.clone();
            let cm_string_list_clone = cm_string_list.clone();
            let monitor_cm_dropdown_clone = monitor_cm_dropdown.clone();
            let monitor_sdrbrightness_onoff_switch_clone =
                monitor_sdrbrightness_onoff_switch.clone();
            let monitor_sdrbrightness_spin_clone = monitor_sdrbrightness_spin.clone();
            let monitor_sdrsaturation_onoff_switch_clone =
                monitor_sdrsaturation_onoff_switch.clone();
            let monitor_sdrsaturation_spin_clone = monitor_sdrsaturation_spin.clone();
            let monitor_vrr_onoff_switch_clone = monitor_vrr_onoff_switch.clone();
            let monitor_vrr_dropdown_clone = monitor_vrr_dropdown.clone();
            let monitor_transform_onoff_switch_clone = monitor_transform_onoff_switch.clone();
            let monitor_transform_dropdown_clone = monitor_transform_dropdown.clone();
            let monitor_addreserved_up_spin_clone = monitor_addreserved_up_spin.clone();
            let monitor_addreserved_down_spin_clone = monitor_addreserved_down_spin.clone();
            let monitor_addreserved_left_spin_clone = monitor_addreserved_left_spin.clone();
            let monitor_addreserved_right_spin_clone = monitor_addreserved_right_spin.clone();

            let load_monitor_values = move |monitor_name: &str, monitor: &Monitor, skip: &str| {
                if skip != "name" {
                    monitor_name_entry_clone.set_text(monitor_name);
                }

                if skip != "resolution" {
                    let resolution_string_list = StringList::new(
                        &get_available_resolutions_for_monitor(monitor_name)
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>(),
                    );
                    monitor_resolution_dropdown_clone.set_model(Some(&resolution_string_list));
                }
                match &monitor {
                    Monitor::Enabled(monitor_state) => {
                        if skip != "resolution" {
                            let resolution = monitor_state.resolution.clone();
                            let mut selected = false;
                            for idx in 0..resolution_string_list.n_items() {
                                if let Some(item) = resolution_string_list.item(idx) {
                                    let item_str = item.property::<String>("string");
                                    if item_str == resolution {
                                        monitor_resolution_dropdown_clone.set_selected(idx);
                                        selected = true;
                                        break;
                                    }
                                }
                            }
                            if !selected {
                                monitor_resolution_dropdown_clone.set_selected(2);
                            }
                        }

                        if skip != "position" {
                            match monitor_state.position.clone() {
                                Position::Coordinates(x, y) => {
                                    monitor_position_x_spin_clone.set_value(x as f64);
                                    monitor_position_x_spin_clone.set_visible(true);
                                    monitor_position_y_spin_clone.set_value(y as f64);
                                    monitor_position_y_spin_clone.set_visible(true);
                                }
                                position => {
                                    let mut selected = false;
                                    for idx in 0..position_string_list.n_items() {
                                        if let Some(item) = position_str_list.get(idx as usize) {
                                            let item_str = item.to_string();
                                            if item_str == position.to_string() {
                                                monitor_position_dropdown_clone.set_selected(idx);
                                                selected = true;

                                                break;
                                            }
                                        }
                                    }

                                    if !selected {
                                        monitor_position_dropdown_clone.set_selected(0);
                                    }
                                    monitor_position_x_spin_clone.set_visible(false);
                                    monitor_position_y_spin_clone.set_visible(false);
                                }
                            }
                        }

                        if skip != "scale" {
                            match monitor_state.scale.clone() {
                                Scale::Auto => {
                                    monitor_scale_dropdown_clone.set_selected(0);
                                    monitor_scale_spin_clone.set_visible(false);
                                }
                                Scale::Manual(scale) => {
                                    monitor_scale_dropdown_clone.set_selected(1);
                                    monitor_scale_spin_clone.set_value(scale);
                                    monitor_scale_spin_clone.set_visible(true);
                                }
                            }
                        }

                        if skip != "mirror" {
                            match &monitor_state.mirror {
                                Some(mirror) => {
                                    monitor_mirror_onoff_switch_clone.set_active(true);
                                    monitor_mirror_entry_clone.set_text(mirror);
                                    monitor_mirror_entry_clone.set_visible(true);
                                }
                                None => {
                                    monitor_mirror_onoff_switch_clone.set_active(false);
                                    monitor_mirror_entry_clone.set_visible(false);
                                }
                            }
                        }

                        if skip != "bitdepth" {
                            match &monitor_state.bitdepth {
                                Some(bitdepth) => {
                                    monitor_bitdepth_onoff_switch_clone.set_active(true);
                                    monitor_bitdepth_spin_clone
                                        .set_value(bitdepth.to_owned() as f64);
                                    monitor_bitdepth_spin_clone.set_visible(true);
                                }
                                None => {
                                    monitor_bitdepth_onoff_switch_clone.set_active(false);
                                    monitor_bitdepth_spin_clone.set_visible(false);
                                }
                            }
                        }

                        if skip != "cm" {
                            match &monitor_state.cm {
                                Some(cm) => {
                                    monitor_cm_onoff_switch_clone.set_active(true);
                                    for idx in 0..cm_string_list_clone.n_items() {
                                        if let Some(item) = cm_string_list_clone.item(idx) {
                                            let item_str = item.property::<String>("string");
                                            if item_str == cm.to_string() {
                                                monitor_cm_dropdown_clone.set_selected(idx);
                                                break;
                                            }
                                        }
                                    }
                                    monitor_cm_dropdown_clone.set_visible(true);
                                }
                                None => {
                                    monitor_cm_onoff_switch_clone.set_active(false);
                                    monitor_cm_dropdown_clone.set_visible(false);
                                }
                            }
                        }

                        if skip != "sdrbrightness" {
                            match &monitor_state.sdrbrightness {
                                Some(sdrbrightness) => {
                                    monitor_sdrbrightness_onoff_switch_clone.set_active(true);
                                    monitor_sdrbrightness_spin_clone
                                        .set_value(sdrbrightness.to_owned());
                                    monitor_sdrbrightness_spin_clone.set_visible(true);
                                }
                                None => {
                                    monitor_sdrbrightness_onoff_switch_clone.set_active(false);
                                    monitor_sdrbrightness_spin_clone.set_visible(false);
                                }
                            }
                        }

                        if skip != "sdrsaturation" {
                            match &monitor_state.sdrsaturation {
                                Some(sdrsaturation) => {
                                    monitor_sdrsaturation_onoff_switch_clone.set_active(true);
                                    monitor_sdrsaturation_spin_clone
                                        .set_value(sdrsaturation.to_owned());
                                    monitor_sdrsaturation_spin_clone.set_visible(true);
                                }
                                None => {
                                    monitor_sdrsaturation_onoff_switch_clone.set_active(false);
                                    monitor_sdrsaturation_spin_clone.set_visible(false);
                                }
                            }
                        }

                        if skip != "vrr" {
                            match &monitor_state.vrr {
                                Some(vrr) => {
                                    monitor_vrr_onoff_switch_clone.set_active(true);
                                    monitor_vrr_dropdown_clone.set_selected(vrr.to_owned() as u32);
                                    monitor_vrr_dropdown_clone.set_visible(true);
                                }
                                None => {
                                    monitor_vrr_onoff_switch_clone.set_active(false);
                                    monitor_vrr_dropdown_clone.set_visible(false);
                                }
                            }
                        }

                        if skip != "transform" {
                            match &monitor_state.transform {
                                Some(transform) => {
                                    monitor_transform_onoff_switch_clone.set_active(true);
                                    monitor_transform_dropdown_clone
                                        .set_selected(transform.to_owned() as u32);
                                    monitor_transform_dropdown_clone.set_visible(true);
                                }
                                None => {
                                    monitor_transform_onoff_switch_clone.set_active(false);
                                    monitor_transform_dropdown_clone.set_visible(false);
                                }
                            }
                        }

                        enabled_box.set_visible(true);
                        monitor_addreserved_box.set_visible(false);
                    }
                    Monitor::AddReserved(
                        monitor_addreserved_up,
                        monitor_addreserved_down,
                        monitor_addreserved_left,
                        monitor_addreserved_right,
                    ) => {
                        if skip != "resolution" {
                            monitor_resolution_dropdown_clone.set_selected(1);
                        }
                        monitor_addreserved_up_spin_clone
                            .set_value(monitor_addreserved_up.to_owned() as f64);
                        monitor_addreserved_down_spin_clone
                            .set_value(monitor_addreserved_down.to_owned() as f64);
                        monitor_addreserved_left_spin_clone
                            .set_value(monitor_addreserved_left.to_owned() as f64);
                        monitor_addreserved_right_spin_clone
                            .set_value(monitor_addreserved_right.to_owned() as f64);

                        enabled_box.set_visible(false);
                        monitor_addreserved_box.set_visible(true);
                    }
                    Monitor::Disabled => {
                        if skip != "resolution" {
                            monitor_resolution_dropdown_clone.set_selected(0);
                        }

                        enabled_box.set_visible(false);
                        monitor_addreserved_box.set_visible(false);
                    }
                }
            };

            load_monitor_values(monitor_name_entry.text().as_ref(), &monitor, "");

            let load_monitor_values_clone = load_monitor_values.clone();

            widget_connector!(
                is_updating,
                value_entry,
                monitor_name_entry,
                connect_changed,
                entry,
                entry.text().to_string(),
                parse_monitor,
                |(_name, monitor), new_name: String| {
                    load_monitor_values_clone(&new_name, &monitor, "name");
                    match monitor {
                        Monitor::Enabled(monitor_state) => {
                            format!("{}, {}", new_name, monitor_state)
                        }
                        Monitor::AddReserved(
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right,
                        ) => format!(
                            "{}, addreserved, {}, {}, {}, {}",
                            new_name,
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right
                        ),
                        Monitor::Disabled => format!("{}, disable", new_name),
                    }
                }
            );

            let load_monitor_values_clone = load_monitor_values.clone();
            let monitor_position_dropdown_clone = monitor_position_dropdown.clone();
            let monitor_position_x_spin_clone = monitor_position_x_spin.clone();
            let monitor_position_y_spin_clone = monitor_position_y_spin.clone();
            let monitor_scale_dropdown_clone = monitor_scale_dropdown.clone();
            let monitor_scale_spin_clone = monitor_scale_spin.clone();
            let monitor_mirror_onoff_switch_clone = monitor_mirror_onoff_switch.clone();
            let monitor_mirror_entry_clone = monitor_mirror_entry.clone();
            let monitor_bitdepth_onoff_switch_clone = monitor_bitdepth_onoff_switch.clone();
            let monitor_bitdepth_spin_clone = monitor_bitdepth_spin.clone();
            let monitor_cm_onoff_switch_clone = monitor_cm_onoff_switch.clone();
            let monitor_cm_dropdown_clone = monitor_cm_dropdown.clone();
            let monitor_sdrbrightness_onoff_switch_clone =
                monitor_sdrbrightness_onoff_switch.clone();
            let monitor_sdrbrightness_spin_clone = monitor_sdrbrightness_spin.clone();
            let monitor_sdrsaturation_onoff_switch_clone =
                monitor_sdrsaturation_onoff_switch.clone();
            let monitor_sdrsaturation_spin_clone = monitor_sdrsaturation_spin.clone();
            let monitor_vrr_onoff_switch_clone = monitor_vrr_onoff_switch.clone();
            let monitor_vrr_dropdown_clone = monitor_vrr_dropdown.clone();
            let monitor_transform_onoff_switch_clone = monitor_transform_onoff_switch.clone();
            let monitor_transform_dropdown_clone = monitor_transform_dropdown.clone();
            let monitor_addreserved_up_spin_clone = monitor_addreserved_up_spin.clone();
            let monitor_addreserved_down_spin_clone = monitor_addreserved_down_spin.clone();
            let monitor_addreserved_left_spin_clone = monitor_addreserved_left_spin.clone();
            let monitor_addreserved_right_spin_clone = monitor_addreserved_right_spin.clone();

            widget_connector!(
                is_updating,
                value_entry,
                monitor_resolution_dropdown,
                connect_selected_notify,
                dropdown,
                dropdown
                    .selected_item()
                    .map(|item| item.property::<String>("string"))
                    .unwrap_or("preferred".to_string()),
                parse_monitor,
                |(name, _monitor): (String, _), new_resolution: String| {
                    let new_monitor_state = match new_resolution.as_str() {
                        "disable" => Monitor::Disabled,
                        "addreserved" => {
                            let monitor_addreserved_up =
                                monitor_addreserved_up_spin_clone.value() as i64;
                            let monitor_addreserved_down =
                                monitor_addreserved_down_spin_clone.value() as i64;
                            let monitor_addreserved_left =
                                monitor_addreserved_left_spin_clone.value() as i64;
                            let monitor_addreserved_right =
                                monitor_addreserved_right_spin_clone.value() as i64;
                            Monitor::AddReserved(
                                monitor_addreserved_up,
                                monitor_addreserved_down,
                                monitor_addreserved_left,
                                monitor_addreserved_right,
                            )
                        }
                        resolution => {
                            let resolution = resolution.to_string();
                            let position = match monitor_position_dropdown_clone.selected() {
                                0 => Position::Auto,
                                1 => Position::AutoRight,
                                2 => Position::AutoLeft,
                                3 => Position::AutoUp,
                                4 => Position::AutoDown,
                                5 => Position::AutoCenterRight,
                                6 => Position::AutoCenterLeft,
                                7 => Position::AutoCenterUp,
                                8 => Position::AutoCenterDown,
                                _ => Position::Coordinates(
                                    monitor_position_x_spin_clone.value() as i64,
                                    monitor_position_y_spin_clone.value() as i64,
                                ),
                            };
                            let scale = match monitor_scale_dropdown_clone.selected() {
                                0 => Scale::Auto,
                                _ => Scale::Manual(monitor_scale_spin_clone.value() as f64),
                            };
                            let mirror = match monitor_mirror_onoff_switch_clone.is_active() {
                                true => Some(monitor_mirror_entry_clone.text().to_string()),
                                false => None,
                            };
                            let bitdepth = match monitor_bitdepth_onoff_switch_clone.is_active() {
                                true => Some(monitor_bitdepth_spin_clone.value() as u8),
                                false => None,
                            };
                            let cm = match monitor_cm_onoff_switch_clone.is_active() {
                                true => Some(
                                    Cm::from_str(
                                        &monitor_cm_dropdown_clone
                                            .selected_item()
                                            .map(|item| item.property::<String>("string"))
                                            .unwrap_or("auto".to_string()),
                                    )
                                    .unwrap_or(Cm::Auto),
                                ),
                                false => None,
                            };
                            let sdrbrightness =
                                match monitor_sdrbrightness_onoff_switch_clone.is_active() {
                                    true => Some(monitor_sdrbrightness_spin_clone.value() as f64),
                                    false => None,
                                };
                            let sdrsaturation =
                                match monitor_sdrsaturation_onoff_switch_clone.is_active() {
                                    true => Some(monitor_sdrsaturation_spin_clone.value() as f64),
                                    false => None,
                                };
                            let vrr = match monitor_vrr_onoff_switch_clone.is_active() {
                                true => Some(monitor_vrr_dropdown_clone.selected() as u8),
                                false => None,
                            };
                            let transform = match monitor_transform_onoff_switch_clone.is_active() {
                                true => Some(monitor_transform_dropdown_clone.selected() as u8),
                                false => None,
                            };

                            Monitor::Enabled(MonitorState {
                                resolution,
                                position,
                                scale,
                                mirror,
                                bitdepth,
                                cm,
                                sdrbrightness,
                                sdrsaturation,
                                vrr,
                                transform,
                            })
                        }
                    };

                    load_monitor_values_clone(&name, &new_monitor_state, "resolution");

                    match new_monitor_state {
                        Monitor::Enabled(monitor_state) => {
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right,
                        ) => format!(
                            "{}, addreserved, {}, {}, {}, {}",
                            name,
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right
                        ),
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            let monitor_position_x_spin_clone = monitor_position_x_spin.clone();
            let monitor_position_y_spin_clone = monitor_position_y_spin.clone();

            widget_connector!(
                is_updating,
                value_entry,
                monitor_position_dropdown,
                connect_selected_notify,
                dropdown,
                dropdown.selected(),
                parse_monitor,
                |(name, monitor), new_position: u32| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.position = Position::from_id(
                                new_position as usize,
                                Some(monitor_position_x_spin_clone.value() as i64),
                                Some(monitor_position_y_spin_clone.value() as i64),
                            );
                            match monitor_state.position {
                                Position::Coordinates(_, _) => {
                                    monitor_position_x_spin_clone.set_visible(true);
                                    monitor_position_y_spin_clone.set_visible(true);
                                }
                                _ => {
                                    monitor_position_x_spin_clone.set_visible(false);
                                    monitor_position_y_spin_clone.set_visible(false);
                                }
                            }
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right,
                        ) => format!(
                            "{}, addreserved, {}, {}, {}, {}",
                            name,
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right
                        ),
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            widget_connector!(
                is_updating,
                value_entry,
                monitor_position_x_spin,
                connect_value_changed,
                spin,
                spin.value(),
                parse_monitor,
                |(name, monitor), new_x: f64| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.position = match monitor_state.position {
                                Position::Coordinates(_x, y) => {
                                    Position::Coordinates(new_x as i64, y)
                                }
                                position => position,
                            };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right,
                        ) => format!(
                            "{}, addreserved, {}, {}, {}, {}",
                            name,
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right
                        ),
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            widget_connector!(
                is_updating,
                value_entry,
                monitor_position_y_spin,
                connect_value_changed,
                spin,
                spin.value(),
                parse_monitor,
                |(name, monitor), new_y: f64| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.position = match monitor_state.position {
                                Position::Coordinates(x, _y) => {
                                    Position::Coordinates(x, new_y as i64)
                                }
                                position => position,
                            };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right,
                        ) => format!(
                            "{}, addreserved, {}, {}, {}, {}",
                            name,
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right
                        ),
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            let monitor_scale_spin_clone = monitor_scale_spin.clone();
            widget_connector!(
                is_updating,
                value_entry,
                monitor_scale_dropdown,
                connect_selected_notify,
                dropdown,
                dropdown.selected(),
                parse_monitor,
                |(name, monitor), new_scale: u32| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.scale = Scale::from_id(
                                new_scale as usize,
                                Some(monitor_scale_spin_clone.value()),
                            );
                            match monitor_state.scale {
                                Scale::Manual(_scale) => {
                                    monitor_scale_spin_clone.set_visible(true);
                                }
                                _ => {
                                    monitor_scale_spin_clone.set_visible(false);
                                }
                            }
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right,
                        ) => format!(
                            "{}, addreserved, {}, {}, {}, {}",
                            name,
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right
                        ),
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            widget_connector!(
                is_updating,
                value_entry,
                monitor_scale_spin,
                connect_value_changed,
                spin,
                spin.value(),
                parse_monitor,
                |(name, monitor), new_scale: f64| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.scale = match monitor_state.scale {
                                Scale::Manual(_scale) => Scale::Manual(new_scale),
                                scale => scale,
                            };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right,
                        ) => format!(
                            "{}, addreserved, {}, {}, {}, {}",
                            name,
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right
                        ),
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                monitor_mirror_onoff_switch,
                monitor_mirror_entry,
                connect_changed,
                entry,
                entry.text().to_string(),
                parse_monitor,
                |(name, monitor), mirror_onoff: bool, mirror_value: String| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.mirror = if mirror_onoff {
                                Some(mirror_value)
                            } else {
                                None
                            };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                },
                |(name, monitor), mirror_entry: String| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.mirror = Some(mirror_entry);
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                monitor_bitdepth_onoff_switch,
                monitor_bitdepth_spin,
                connect_value_changed,
                spin,
                spin.value() as u8,
                parse_monitor,
                |(name, monitor), bitdepth_onoff: bool, bitdepth_value: u8| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.bitdepth = if bitdepth_onoff {
                                Some(bitdepth_value)
                            } else {
                                None
                            };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                },
                |(name, monitor), bitdepth_spin: u8| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.bitdepth = Some(bitdepth_spin);
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                monitor_cm_onoff_switch,
                monitor_cm_dropdown,
                connect_selected_notify,
                dropdown,
                Cm::from_id(dropdown.selected()),
                parse_monitor,
                |(name, monitor), cm_onoff: bool, cm_value: Cm| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.cm = if cm_onoff { Some(cm_value) } else { None };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                },
                |(name, monitor), cm_dropdown: Cm| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.cm = Some(cm_dropdown);
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                monitor_sdrbrightness_onoff_switch,
                monitor_sdrbrightness_spin,
                connect_value_changed,
                spin,
                spin.value(),
                parse_monitor,
                |(name, monitor), sdrbrightness_onoff: bool, sdrbrightness_value: f64| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.sdrbrightness = if sdrbrightness_onoff {
                                Some(sdrbrightness_value)
                            } else {
                                None
                            };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                },
                |(name, monitor), sdrbrightness_spin: f64| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.sdrbrightness = Some(sdrbrightness_spin);
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                monitor_sdrsaturation_onoff_switch,
                monitor_sdrsaturation_spin,
                connect_value_changed,
                spin,
                spin.value(),
                parse_monitor,
                |(name, monitor), sdrsaturation_onoff: bool, sdrsaturation_value: f64| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.sdrsaturation = if sdrsaturation_onoff {
                                Some(sdrsaturation_value)
                            } else {
                                None
                            };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                },
                |(name, monitor), sdrsaturation_spin: f64| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.sdrsaturation = Some(sdrsaturation_spin);
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                monitor_vrr_onoff_switch,
                monitor_vrr_dropdown,
                connect_selected_notify,
                dropdown,
                dropdown.selected() as u8,
                parse_monitor,
                |(name, monitor), vrr_onoff: bool, vrr_value: u8| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.vrr = if vrr_onoff { Some(vrr_value) } else { None };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                },
                |(name, monitor), vrr_dropdown: u8| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.vrr = Some(vrr_dropdown);
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                monitor_transform_onoff_switch,
                monitor_transform_dropdown,
                connect_selected_notify,
                dropdown,
                dropdown.selected() as u8,
                parse_monitor,
                |(name, monitor), transform_onoff: bool, transform_value: u8| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.transform = if transform_onoff {
                                Some(transform_value)
                            } else {
                                None
                            };
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                },
                |(name, monitor), transform_dropdown: u8| {
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.transform = Some(transform_dropdown);
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::AddReserved(up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right
                            )
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            widget_connector!(
                is_updating,
                value_entry,
                monitor_addreserved_up_spin,
                connect_value_changed,
                spin,
                spin.value() as u64,
                parse_monitor,
                |(name, monitor), up_spin: u64| {
                    match monitor {
                        Monitor::AddReserved(_up, down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up_spin, down, left, right
                            )
                        }
                        Monitor::Enabled(monitor_state) => {
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            widget_connector!(
                is_updating,
                value_entry,
                monitor_addreserved_down_spin,
                connect_value_changed,
                spin,
                spin.value() as u64,
                parse_monitor,
                |(name, monitor), down_spin: u64| {
                    match monitor {
                        Monitor::AddReserved(up, _down, left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down_spin, left, right
                            )
                        }
                        Monitor::Enabled(monitor_state) => {
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            widget_connector!(
                is_updating,
                value_entry,
                monitor_addreserved_left_spin,
                connect_value_changed,
                spin,
                spin.value() as u64,
                parse_monitor,
                |(name, monitor), left_spin: u64| {
                    match monitor {
                        Monitor::AddReserved(up, down, _left, right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left_spin, right
                            )
                        }
                        Monitor::Enabled(monitor_state) => {
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            widget_connector!(
                is_updating,
                value_entry,
                monitor_addreserved_right_spin,
                connect_value_changed,
                spin,
                spin.value() as u64,
                parse_monitor,
                |(name, monitor), right_spin: u64| {
                    match monitor {
                        Monitor::AddReserved(up, down, left, _right) => {
                            format!(
                                "{}, addreserved, {}, {}, {}, {}",
                                name, up, down, left, right_spin
                            )
                        }
                        Monitor::Enabled(monitor_state) => {
                            format!("{}, {}", name, monitor_state)
                        }
                        Monitor::Disabled => format!("{}, disable", name),
                    }
                }
            );

            {
                let is_updating_clone = is_updating.clone();
                let load_monitor_values_clone = load_monitor_values.clone();
                value_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);

                    let value = entry.text().to_string();
                    let (name, monitor) = parse_monitor(&value);

                    load_monitor_values_clone(&name, &monitor, "");

                    is_updating_clone.set(false);
                });
            }
        }
        "animation" => {
            if name == "bezier" {
                let (bezier_name, bezier_x0, bezier_y0, bezier_x1, bezier_y1) =
                    parse_bezier(&value_entry.text());

                let bezier_name_entry = create_entry();
                bezier_name_entry.set_text(&bezier_name);

                let bezier_x0_spin = create_spin_button(0.0, 1.0, 0.01);
                bezier_x0_spin.set_value(bezier_x0);
                bezier_x0_spin.set_digits(2);

                let bezier_y0_spin = create_spin_button(-10.0, 10.0, 0.01);
                bezier_y0_spin.set_value(bezier_y0);
                bezier_y0_spin.set_digits(2);

                let bezier_x1_spin = create_spin_button(0.0, 1.0, 0.01);
                bezier_x1_spin.set_value(bezier_x1);
                bezier_x1_spin.set_digits(2);

                let bezier_y1_spin = create_spin_button(-10.0, 10.0, 0.01);
                bezier_y1_spin.set_value(bezier_y1);
                bezier_y1_spin.set_digits(2);

                widget_connector!(
                    is_updating,
                    value_entry,
                    bezier_name_entry,
                    connect_changed,
                    entry,
                    entry.text().to_string(),
                    parse_bezier,
                    |(_name, x0, y0, x1, y1), new_name: String| {
                        format!("{}, {:.2}, {:.2}, {:.2}, {:.2}", new_name, x0, y0, x1, y1)
                    }
                );

                widget_connector!(
                    is_updating,
                    value_entry,
                    bezier_x0_spin,
                    connect_value_changed,
                    spin,
                    spin.value(),
                    parse_bezier,
                    |(name, _x0, y0, x1, y1), new_x0: f64| {
                        format!("{}, {:.2}, {:.2}, {:.2}, {:.2}", name, new_x0, y0, x1, y1)
                    }
                );

                widget_connector!(
                    is_updating,
                    value_entry,
                    bezier_y0_spin,
                    connect_value_changed,
                    spin,
                    spin.value(),
                    parse_bezier,
                    |(name, x0, _y0, x1, y1), new_y0: f64| {
                        format!("{}, {:.2}, {:.2}, {:.2}, {:.2}", name, x0, new_y0, x1, y1)
                    }
                );

                widget_connector!(
                    is_updating,
                    value_entry,
                    bezier_x1_spin,
                    connect_value_changed,
                    spin,
                    spin.value(),
                    parse_bezier,
                    |(name, x0, y0, _x1, y1), new_x1: f64| {
                        format!("{}, {:.2}, {:.2}, {:.2}, {:.2}", name, x0, y0, new_x1, y1)
                    }
                );

                widget_connector!(
                    is_updating,
                    value_entry,
                    bezier_y1_spin,
                    connect_value_changed,
                    spin,
                    spin.value(),
                    parse_bezier,
                    |(name, x0, y0, x1, _y1), new_y1: f64| {
                        format!("{}, {:.2}, {:.2}, {:.2}, {:.2}", name, x0, y0, x1, new_y1)
                    }
                );

                {
                    let is_updating_clone = is_updating.clone();
                    let bezier_name_entry_clone = bezier_name_entry.clone();
                    let bezier_x0_spin_clone = bezier_x0_spin.clone();
                    let bezier_y0_spin_clone = bezier_y0_spin.clone();
                    let bezier_x1_spin_clone = bezier_x1_spin.clone();
                    let bezier_y1_spin_clone = bezier_y1_spin.clone();

                    value_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }

                        is_updating_clone.set(true);
                        let (name, x0, y0, x1, y1) = parse_bezier(&entry.text());

                        bezier_name_entry_clone.set_text(&name);
                        bezier_x0_spin_clone.set_value(x0);
                        bezier_y0_spin_clone.set_value(y0);
                        bezier_x1_spin_clone.set_value(x1);
                        bezier_y1_spin_clone.set_value(y1);

                        is_updating_clone.set(false);
                    });
                }

                fancy_value_entry.append(&Label::new(Some(&t!("name"))));
                fancy_value_entry.append(&bezier_name_entry);
                fancy_value_entry.append(&Label::new(Some("X0")));
                fancy_value_entry.append(&bezier_x0_spin);
                fancy_value_entry.append(&Label::new(Some("Y0")));
                fancy_value_entry.append(&bezier_y0_spin);
                fancy_value_entry.append(&Label::new(Some("X1")));
                fancy_value_entry.append(&bezier_x1_spin);
                fancy_value_entry.append(&Label::new(Some("Y1")));
                fancy_value_entry.append(&bezier_y1_spin);
            } else {
                let (
                    animation_name,
                    animation_onoff,
                    animation_speed,
                    animation_curve,
                    animation_style,
                ) = parse_animation(&value_entry.text());

                let is_updating = Rc::new(Cell::new(false));

                let animation_name_entry = create_entry();
                animation_name_entry.set_text(&animation_name);

                let animation_onoff_switch = create_switch();
                animation_onoff_switch.set_active(animation_onoff);

                let animation_speed_spin = create_spin_button(0.1, 100.0, 0.1);
                animation_speed_spin.set_value(animation_speed);
                animation_speed_spin.set_digits(1);

                let animation_curve_entry = create_entry();
                animation_curve_entry.set_text(&animation_curve);

                let animation_style_entry = create_entry();
                if let Some(style) = &animation_style {
                    animation_style_entry.set_text(style);
                }

                widget_connector!(
                    is_updating,
                    value_entry,
                    animation_name_entry,
                    connect_changed,
                    entry,
                    entry.text().to_string(),
                    parse_animation,
                    |(_name, onoff, speed, curve, style), new_name: String| {
                        match style {
                            Some(s) => format!(
                                "{}, {}, {:.1}, {}, {}",
                                new_name, onoff as u8, speed, curve, s
                            ),
                            None => {
                                format!("{}, {}, {:.1}, {}", new_name, onoff as u8, speed, curve)
                            }
                        }
                    }
                );

                widget_connector!(
                    is_updating,
                    value_entry,
                    animation_onoff_switch,
                    connect_state_set,
                    _switch_widget,
                    state,
                    state,
                    parse_animation,
                    |(name, _onoff, speed, curve, style), new_onoff: bool| {
                        match style {
                            Some(s) => {
                                format!(
                                    "{}, {}, {:.1}, {}, {}",
                                    name, new_onoff as u8, speed, curve, s
                                )
                            }
                            None => {
                                format!("{}, {}, {:.1}, {}", name, new_onoff as u8, speed, curve)
                            }
                        }
                    }
                );

                widget_connector!(
                    is_updating,
                    value_entry,
                    animation_speed_spin,
                    connect_value_changed,
                    spin,
                    spin.value(),
                    parse_animation,
                    |(name, onoff, _speed, curve, style), new_speed: f64| {
                        match style {
                            Some(s) => format!(
                                "{}, {}, {:.1}, {}, {}",
                                name, onoff as u8, new_speed, curve, s
                            ),
                            None => {
                                format!("{}, {}, {:.1}, {}", name, onoff as u8, new_speed, curve)
                            }
                        }
                    }
                );

                widget_connector!(
                    is_updating,
                    value_entry,
                    animation_curve_entry,
                    connect_changed,
                    entry,
                    entry.text().to_string(),
                    parse_animation,
                    |(name, onoff, speed, _curve, style), new_curve: String| {
                        match style {
                            Some(s) => format!(
                                "{}, {}, {:.1}, {}, {}",
                                name, onoff as u8, speed, new_curve, s
                            ),
                            None => {
                                format!("{}, {}, {:.1}, {}", name, onoff as u8, speed, new_curve)
                            }
                        }
                    }
                );

                widget_connector!(
                    is_updating,
                    value_entry,
                    animation_style_entry,
                    connect_changed,
                    entry,
                    entry.text().to_string(),
                    parse_animation,
                    |(name, onoff, speed, curve, _style), new_style: String| {
                        if new_style.is_empty() {
                            format!("{}, {}, {:.1}, {}", name, onoff as u8, speed, curve)
                        } else {
                            format!(
                                "{}, {}, {:.1}, {}, {}",
                                name, onoff as u8, speed, curve, new_style
                            )
                        }
                    }
                );

                {
                    let is_updating_clone = is_updating.clone();
                    let animation_name_entry_clone = animation_name_entry.clone();
                    let animation_onoff_switch_clone = animation_onoff_switch.clone();
                    let animation_speed_spin_clone = animation_speed_spin.clone();
                    let animation_curve_entry_clone = animation_curve_entry.clone();
                    let animation_style_entry_clone = animation_style_entry.clone();

                    value_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }

                        is_updating_clone.set(true);
                        let (name, onoff, speed, curve, style) = parse_animation(&entry.text());

                        animation_name_entry_clone.set_text(&name);
                        animation_onoff_switch_clone.set_active(onoff);
                        animation_speed_spin_clone.set_value(speed);
                        animation_curve_entry_clone.set_text(&curve);

                        match style {
                            Some(s) => {
                                animation_style_entry_clone.set_text(&s);
                            }
                            None => {
                                animation_style_entry_clone.set_text("");
                            }
                        }

                        is_updating_clone.set(false);
                    });
                }

                fancy_value_entry.append(&Label::new(Some(&t!("name"))));
                fancy_value_entry.append(&animation_name_entry);
                fancy_value_entry.append(&Label::new(Some(&t!("onoff"))));
                fancy_value_entry.append(&animation_onoff_switch);
                fancy_value_entry.append(&Label::new(Some(&t!("speed"))));
                fancy_value_entry.append(&animation_speed_spin);
                fancy_value_entry.append(&Label::new(Some(&t!("curve"))));
                fancy_value_entry.append(&animation_curve_entry);
                fancy_value_entry.append(&Label::new(Some(&t!("style"))));
                fancy_value_entry.append(&animation_style_entry);
            }
        }
        "bind" => {
            todo!()
        }
        _ => {}
    }
}
