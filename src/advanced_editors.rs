use crate::utils::{
    after_second_comma, is_modifier, keycode_to_en_key, parse_animation, parse_bezier,
    parse_coordinates,
};
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
    match category {
        "monitor" => {
            todo!()
        }
        "animation" => {
            if name == "bezier" {
                let is_updating = Rc::new(Cell::new(false));

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
