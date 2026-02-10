use crate::{
    gtk_converters::{FieldLabel, ToGtkBox, ToGtkBoxWithSeparator, ToGtkBoxWithSeparatorAndNames},
    hyprland::{
        Animation, AnimationName, AnimationStyle, BezierCurve as HyprBezierCurve, BindFlags,
        BindFlagsEnum, BindLeft, Cm, Dispatcher, ExecWithRules, Gesture, LayerRuleWithParameter,
        Modifier, Monitor, MonitorSelector, MonitorState, Orientation, Position, Scale, Side,
        UnbindRight, WindowRuleWithParameters, Workspace, WorkspaceSelector, WorkspaceType,
        animation::parse_animation, bezier_curve::parse_bezier, bind_right::parse_bind_right,
        monitor::parse_monitor, workspace::parse_workspace,
    },
    utils::{
        MAX_SAFE_INTEGER_F64, MAX_SAFE_STEP_0_01_F64, MIN_SAFE_INTEGER_F64, after_second_comma,
        cow_to_static_str, get_available_monitors, get_available_resolutions_for_monitor,
        is_modifier, join_with_separator, keycode_to_en_key, parse_coordinates,
    },
};
use gio::glib::SignalHandlerId;
use gtk::{
    Align, ApplicationWindow, Box, Button, DrawingArea, DropDown, Entry, EventControllerKey,
    EventControllerMotion, GestureClick, Label, Orientation as GtkOrientation, Separator,
    SpinButton, StringList, StringObject, Switch, TextBuffer, TextView, prelude::*,
};
use rust_i18n::t;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    f64,
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
        .orientation(GtkOrientation::Vertical)
        .spacing(10)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .visible(false)
        .build();

    let toggle_button = Button::with_label(&t!("advanced_editors.show_editor"));

    let vbox_clone = vbox.clone();
    let button_clone = toggle_button.clone();
    let show_editor = t!("advanced_editors.show_editor");
    let hide_editor = t!("advanced_editors.hide_editor");
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
            cr.arc(p.x, p.y, 6.0 / scale_factor, 0.0, 2.0 * f64::consts::PI);
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
    let container = Box::new(GtkOrientation::Vertical, 5);

    let toggle_button = Button::with_label(&t!("advanced_editors.record_bind"));

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

            toggle_button_clone.set_label(&t!("advanced_editors.record_bind"));
        } else {
            *is_recording_mut = true;

            let vbox = Box::new(GtkOrientation::Vertical, 5);

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

            toggle_button_clone_clone.set_label(&t!("advanced_editors.stop_recording"));
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
        buffer.insert(&mut end_iter, &t!("advanced_editors.active_combinations"));

        let mut modifiers_vec: Vec<&str> = modifiers.into_iter().collect();
        modifiers_vec.sort_by_key(|&modifier| modifier_priority(&modifier));
        let modifiers_str = modifiers_vec.join(" + ");

        for key in &regular_keys {
            value_entry.set_text(&format!("{}, {}{}", modifiers_str, key, bind_action));
            buffer.insert(&mut end_iter, &format!("  {} + {}\n", modifiers_str, key));
        }
    } else if !modifiers.is_empty() {
        buffer.insert(&mut end_iter, &t!("advanced_editors.active_modifiers"));

        let mut modifiers_vec: Vec<&str> = modifiers.into_iter().collect();
        modifiers_vec.sort_by_key(|&modifier| modifier_priority(&modifier));
        let modifiers_str = modifiers_vec.join(" + ");

        for modifier in modifiers_vec {
            value_entry.set_text(&format!("{}, {}", modifiers_str, bind_action));
            buffer.insert(&mut end_iter, &format!("  {}\n", modifier));
        }
    } else if !regular_keys.is_empty() {
        buffer.insert(&mut end_iter, &t!("advanced_editors.active_keys"));

        for key in &regular_keys {
            value_entry.set_text(&format!(", {}{}", key, bind_action));
            buffer.insert(&mut end_iter, &format!("  {}\n", key));
        }
    } else {
        buffer.insert(&mut end_iter, &t!("advanced_editors.no_active_inputs"));
    }
}

pub fn create_fancy_boxline(category: &str, name_entry: &Entry, value_entry: &Entry) -> Box {
    let fancy_boxline = Box::new(GtkOrientation::Horizontal, 5);

    let fancy_name_entry = Box::new(GtkOrientation::Horizontal, 5);

    let is_updating = Rc::new(Cell::new(false));

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
        "workspace" => {
            let label = Label::new(Some("workspace"));
            label.set_width_request(100);
            label.set_selectable(true);
            fancy_name_entry.append(&label);
            name_entry.set_text("workspace");
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
            name_entry.set_text("animation");
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
            let bind_left_box = Box::new(GtkOrientation::Horizontal, 5);

            let bind_type_string_list =
                StringList::new(&[&t!("advanced_editors.bind"), &t!("advanced_editors.unbind")]);
            let bind_type_dropdown = create_dropdown(&bind_type_string_list);
            bind_left_box.append(&Label::new(Some(&t!("advanced_editors.type"))));
            bind_left_box.append(&bind_type_dropdown);

            let flags_box = Box::new(GtkOrientation::Vertical, 5);
            flags_box.set_margin_start(10);

            let flag_names = BindFlagsEnum::get_all();

            let mut switches = Vec::new();
            for flag_name in flag_names {
                let flag_box = Box::new(GtkOrientation::Horizontal, 5);
                let switch = create_switch();
                flag_box.append(&Label::new(Some(&flag_name.to_fancy_string())));
                flag_box.append(&switch);
                flags_box.append(&flag_box);
                switches.push((flag_name, switch));
            }

            bind_left_box.append(&flags_box);
            fancy_name_entry.append(&bind_left_box);

            let flags_box_clone = flags_box.clone();
            let name_entry_clone = name_entry.clone();
            let is_updating_clone = is_updating.clone();
            bind_type_dropdown.connect_selected_notify(move |dropdown| {
                if is_updating_clone.get() {
                    return;
                }

                is_updating_clone.set(true);
                flags_box_clone.set_visible(dropdown.selected() == 0);

                let new_text = if dropdown.selected() == 0 {
                    "bind"
                } else {
                    "unbind"
                };
                name_entry_clone.set_text(new_text);
                is_updating_clone.set(false);
            });
            name_entry.set_text("bind");

            let switches_clone = switches.clone();
            for (flag_name, switch) in switches_clone {
                let name_entry_clone = name_entry.clone();
                let is_updating_clone = is_updating.clone();
                let dropdown_clone = bind_type_dropdown.clone();

                switch.connect_state_notify(move |switch| {
                    if is_updating_clone.get() || dropdown_clone.selected() != 0 {
                        return;
                    }

                    is_updating_clone.set(true);

                    if let Ok(mut bind_left) = BindLeft::from_str(&name_entry_clone.text())
                        && let BindLeft::Bind(flags) = &mut bind_left
                    {
                        match flag_name {
                            BindFlagsEnum::Locked => flags.locked = switch.is_active(),
                            BindFlagsEnum::Release => flags.release = switch.is_active(),
                            BindFlagsEnum::Click => flags.click = switch.is_active(),
                            BindFlagsEnum::Drag => flags.drag = switch.is_active(),
                            BindFlagsEnum::LongPress => flags.long_press = switch.is_active(),
                            BindFlagsEnum::Repeat => flags.repeat = switch.is_active(),
                            BindFlagsEnum::NonConsuming => flags.non_consuming = switch.is_active(),
                            BindFlagsEnum::Mouse => flags.mouse = switch.is_active(),
                            BindFlagsEnum::Transparent => flags.transparent = switch.is_active(),
                            BindFlagsEnum::IgnoreMods => flags.ignore_mods = switch.is_active(),
                            BindFlagsEnum::Separate => flags.separate = switch.is_active(),
                            BindFlagsEnum::HasDescription => {
                                flags.has_description = switch.is_active()
                            }
                            BindFlagsEnum::Bypass => flags.bypass = switch.is_active(),
                        }

                        name_entry_clone.set_text(&bind_left.to_string());
                    }

                    is_updating_clone.set(false);
                });
            }

            let dropdown_clone = bind_type_dropdown.clone();
            let flags_box_clone = flags_box.clone();
            let switches_clone = switches.clone();
            let is_updating_clone = is_updating.clone();
            name_entry.connect_changed(move |entry| {
                if is_updating_clone.get() {
                    return;
                }

                is_updating_clone.set(true);
                let text = entry.text();

                if let Ok(bind_left) = BindLeft::from_str(&text) {
                    match bind_left {
                        BindLeft::Bind(flags) => {
                            dropdown_clone.set_selected(0);
                            flags_box_clone.set_visible(true);

                            for (flag_name, switch) in &switches_clone {
                                let flag_value = match flag_name {
                                    BindFlagsEnum::Locked => flags.locked,
                                    BindFlagsEnum::Release => flags.release,
                                    BindFlagsEnum::Click => flags.click,
                                    BindFlagsEnum::Drag => flags.drag,
                                    BindFlagsEnum::LongPress => flags.long_press,
                                    BindFlagsEnum::Repeat => flags.repeat,
                                    BindFlagsEnum::NonConsuming => flags.non_consuming,
                                    BindFlagsEnum::Mouse => flags.mouse,
                                    BindFlagsEnum::Transparent => flags.transparent,
                                    BindFlagsEnum::IgnoreMods => flags.ignore_mods,
                                    BindFlagsEnum::Separate => flags.separate,
                                    BindFlagsEnum::HasDescription => flags.has_description,
                                    BindFlagsEnum::Bypass => flags.bypass,
                                };
                                switch.set_active(flag_value);
                            }
                        }
                        BindLeft::Unbind => {
                            dropdown_clone.set_selected(1);
                            flags_box_clone.set_visible(false);
                        }
                    }
                }

                is_updating_clone.set(false);
            });

            if let Ok(bind_left) = BindLeft::from_str(&name_entry.text()) {
                match bind_left {
                    BindLeft::Bind(_) => bind_type_dropdown.set_selected(0),
                    BindLeft::Unbind => bind_type_dropdown.set_selected(1),
                }
            }
        }
        "gesture" => {
            let label = Label::new(Some("gesture"));
            label.set_width_request(100);
            label.set_selectable(true);
            fancy_name_entry.append(&label);
            name_entry.set_text("gesture");
            name_entry.connect_changed(move |entry| {
                label.set_text(&entry.text());
            });
        }
        "windowrule" => {
            let label = Label::new(Some("windowrule"));
            label.set_width_request(100);
            label.set_selectable(true);
            fancy_name_entry.append(&label);
            name_entry.set_text("windowrule");
            name_entry.connect_changed(move |entry| {
                label.set_text(&entry.text());
            });
        }
        "layerrule" => {
            let label = Label::new(Some("layerrule"));
            label.set_width_request(100);
            label.set_selectable(true);
            fancy_name_entry.append(&label);
            name_entry.set_text("layerrule");
            name_entry.connect_changed(move |entry| {
                label.set_text(&entry.text());
            });
        }
        "exec" => {
            let string_list =
                StringList::new(&["exec-once", "execr-once", "exec", "execr", "exec-shutdown"]);
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
            name_entry.set_text("exec-once");
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
        "env" => {
            let string_list = StringList::new(&["env", "envd"]);
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
            name_entry.set_text("env");
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
        "top_level" => {
            // maybe in future i will implement this
        }
        e => {
            dbg!(e);
            unreachable!()
        }
    }

    fancy_boxline.append(&fancy_name_entry);

    let equals_label = Label::new(Some("="));
    equals_label.set_xalign(0.5);
    fancy_boxline.append(&equals_label);

    let fancy_value_entry = Box::new(GtkOrientation::Horizontal, 5);

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

pub fn create_entry() -> Entry {
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

pub fn create_spin_button(min: f64, max: f64, step: f64) -> SpinButton {
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

pub fn create_dropdown(string_list: &StringList) -> DropDown {
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

pub fn create_switch() -> Switch {
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

pub fn create_button(text: &str) -> Button {
    Button::builder()
        .width_request(100)
        .hexpand(true)
        .halign(Align::Fill)
        .margin_top(5)
        .margin_bottom(5)
        .vexpand(false)
        .valign(Align::Center)
        .label(text)
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
            let (monitor_selector, monitor) = parse_monitor(&value_entry.text());

            let name_box = Box::new(GtkOrientation::Horizontal, 5);
            name_box.append(&Label::new(Some(&t!("advanced_editors.name"))));
            let monitors = get_available_monitors(false);
            let mut monitor_selector_list =
                monitors.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
            let all = t!("advanced_editors.all");
            monitor_selector_list.insert(0, &all);
            let monitor_selector_string_list = StringList::new(&monitor_selector_list);
            let monitor_selector_dropdown = create_dropdown(&monitor_selector_string_list);
            name_box.append(&monitor_selector_dropdown);
            fancy_value_entry.append(&name_box);

            let resolution_box = Box::new(GtkOrientation::Horizontal, 5);
            resolution_box.append(&Label::new(Some(&t!("advanced_editors.resolution/mode"))));
            let resolution_string_list = StringList::new(
                &get_available_resolutions_for_monitor(&monitor_selector)
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>(),
            );
            let monitor_resolution_dropdown = create_dropdown(&resolution_string_list);
            resolution_box.append(&monitor_resolution_dropdown);
            fancy_value_entry.append(&resolution_box);

            let enabled_box = Box::new(GtkOrientation::Vertical, 5);

            let position_box = Box::new(GtkOrientation::Horizontal, 5);
            position_box.append(&Label::new(Some(&t!("advanced_editors.position"))));
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

            let monitor_scale_box = Box::new(GtkOrientation::Horizontal, 5);
            monitor_scale_box.append(&Label::new(Some(&t!("advanced_editors.scale"))));
            let scale_string_list =
                StringList::new(&Scale::get_fancy_list().each_ref().map(|s| s.as_str()));
            let monitor_scale_dropdown = create_dropdown(&scale_string_list);
            monitor_scale_box.append(&monitor_scale_dropdown);
            let monitor_scale_spin = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
            monitor_scale_spin.set_digits(2);
            monitor_scale_spin.set_value(1.0);
            monitor_scale_box.append(&monitor_scale_spin);
            enabled_box.append(&monitor_scale_box);

            let monitor_mirror_box = Box::new(GtkOrientation::Horizontal, 5);
            monitor_mirror_box.append(&Label::new(Some(&t!("advanced_editors.mirror"))));
            let monitor_mirror_onoff_switch = create_switch();
            monitor_mirror_box.append(&monitor_mirror_onoff_switch);
            let monitor_mirror_selector_string_list = StringList::new(
                &get_available_monitors(true)
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>(),
            );
            let monitor_mirror_selector_dropdown =
                create_dropdown(&monitor_mirror_selector_string_list);
            monitor_mirror_box.append(&monitor_mirror_selector_dropdown);
            enabled_box.append(&monitor_mirror_box);

            let monitor_bitdepth_box = Box::new(GtkOrientation::Horizontal, 5);
            monitor_bitdepth_box.append(&Label::new(Some(&t!("advanced_editors.bitdepth"))));
            let monitor_bitdepth_onoff_switch = create_switch();
            monitor_bitdepth_box.append(&monitor_bitdepth_onoff_switch);
            let monitor_bitdepth_spin = create_spin_button(1.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_bitdepth_spin.set_value(10.0);
            monitor_bitdepth_box.append(&monitor_bitdepth_spin);
            enabled_box.append(&monitor_bitdepth_box);

            let monitor_cm_box = Box::new(GtkOrientation::Horizontal, 5);
            monitor_cm_box.append(&Label::new(Some(&t!("advanced_editors.color_management"))));
            let monitor_cm_onoff_switch = create_switch();
            monitor_cm_box.append(&monitor_cm_onoff_switch);
            let cm_string_list = StringList::new(&Cm::get_fancy_list());
            let monitor_cm_dropdown = create_dropdown(&cm_string_list);
            monitor_cm_box.append(&monitor_cm_dropdown);
            enabled_box.append(&monitor_cm_box);

            let monitor_sdrbrightness_box = Box::new(GtkOrientation::Horizontal, 5);
            monitor_sdrbrightness_box
                .append(&Label::new(Some(&t!("advanced_editors.sdr_brightness"))));
            let monitor_sdrbrightness_onoff_switch = create_switch();
            monitor_sdrbrightness_box.append(&monitor_sdrbrightness_onoff_switch);
            let monitor_sdrbrightness_spin = create_spin_button(0.0, f64::MAX, 0.01);
            monitor_sdrbrightness_spin.set_value(1.0);
            monitor_sdrbrightness_box.append(&monitor_sdrbrightness_spin);
            enabled_box.append(&monitor_sdrbrightness_box);

            let monitor_sdrsaturation_box = Box::new(GtkOrientation::Horizontal, 5);
            monitor_sdrsaturation_box
                .append(&Label::new(Some(&t!("advanced_editors.sdr_saturation"))));
            let monitor_sdrsaturation_onoff_switch = create_switch();
            monitor_sdrsaturation_box.append(&monitor_sdrsaturation_onoff_switch);
            let monitor_sdrsaturation_spin = create_spin_button(0.0, f64::MAX, 0.01);
            monitor_sdrsaturation_spin.set_value(1.0);
            monitor_sdrsaturation_box.append(&monitor_sdrsaturation_spin);
            enabled_box.append(&monitor_sdrsaturation_box);

            let monitor_vrr_box = Box::new(GtkOrientation::Horizontal, 5);
            monitor_vrr_box.append(&Label::new(Some(&t!("advanced_editors.vrr"))));
            let monitor_vrr_onoff_switch = create_switch();
            monitor_vrr_box.append(&monitor_vrr_onoff_switch);
            let vrr_string_list = StringList::new(&[
                &t!("widget.misc_category.vrr_off"),
                &t!("widget.misc_category.vrr_on"),
                &t!("widget.misc_category.vrr_fullscreen_only"),
                &t!("widget.misc_category.vrr_fullscreen_with_video/game"),
            ]);
            let monitor_vrr_dropdown = create_dropdown(&vrr_string_list);
            monitor_vrr_box.append(&monitor_vrr_dropdown);
            enabled_box.append(&monitor_vrr_box);

            let monitor_transform_box = Box::new(GtkOrientation::Horizontal, 5);
            monitor_transform_box.append(&Label::new(Some(&t!("advanced_editors.transform"))));
            let monitor_transform_onoff_switch = create_switch();
            monitor_transform_box.append(&monitor_transform_onoff_switch);
            let transform_string_list = StringList::new(&[
                &t!("advanced_editors.normal"),
                &t!("advanced_editors.rotate_90"),
                &t!("advanced_editors.rotate_180"),
                &t!("advanced_editors.rotate_270"),
                &t!("advanced_editors.flip"),
                &t!("advanced_editors.flip_rotate_90"),
                &t!("advanced_editors.flip_rotate_180"),
                &t!("advanced_editors.flip_rotate_270"),
            ]);
            let monitor_transform_dropdown = create_dropdown(&transform_string_list);
            monitor_transform_box.append(&monitor_transform_dropdown);
            enabled_box.append(&monitor_transform_box);

            fancy_value_entry.append(&enabled_box);

            let monitor_addreserved_box = Box::new(GtkOrientation::Vertical, 5);

            monitor_addreserved_box.append(&Label::new(Some(&t!("advanced_editors.top"))));
            let monitor_addreserved_up_spin = create_spin_button(0.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_addreserved_box.append(&monitor_addreserved_up_spin);

            monitor_addreserved_box.append(&Label::new(Some(&t!("advanced_editors.bottom"))));
            let monitor_addreserved_down_spin = create_spin_button(0.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_addreserved_box.append(&monitor_addreserved_down_spin);

            monitor_addreserved_box.append(&Label::new(Some(&t!("advanced_editors.left"))));
            let monitor_addreserved_left_spin = create_spin_button(0.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_addreserved_box.append(&monitor_addreserved_left_spin);

            monitor_addreserved_box.append(&Label::new(Some(&t!("advanced_editors.right"))));
            let monitor_addreserved_right_spin = create_spin_button(0.0, MAX_SAFE_INTEGER_F64, 1.0);
            monitor_addreserved_box.append(&monitor_addreserved_right_spin);

            fancy_value_entry.append(&monitor_addreserved_box);

            let monitor_selector_string_list_clone = monitor_selector_string_list.clone();
            let monitor_selector_dropdown_clone = monitor_selector_dropdown.clone();
            let monitor_resolution_dropdown_clone = monitor_resolution_dropdown.clone();
            let monitor_position_dropdown_clone = monitor_position_dropdown.clone();
            let monitor_position_x_spin_clone = monitor_position_x_spin.clone();
            let monitor_position_y_spin_clone = monitor_position_y_spin.clone();
            let monitor_scale_dropdown_clone = monitor_scale_dropdown.clone();
            let monitor_scale_spin_clone = monitor_scale_spin.clone();
            let monitor_mirror_onoff_switch_clone = monitor_mirror_onoff_switch.clone();
            let monitor_mirror_selector_dropdown_clone = monitor_mirror_selector_dropdown.clone();
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

            let load_monitor_values = move |monitor_selector: &MonitorSelector,
                                            monitor: &Monitor,
                                            skip: &str| {
                if skip != "selector" {
                    match monitor_selector {
                        MonitorSelector::All => {
                            monitor_selector_dropdown_clone.set_selected(0);
                        }
                        MonitorSelector::Name(name) => {
                            let mut selected = false;
                            for idx in 0..monitor_selector_string_list_clone.n_items() {
                                if let Some(item) = monitor_selector_string_list_clone.item(idx) {
                                    let item_str = item.property::<String>("string");
                                    if &item_str == name {
                                        monitor_selector_dropdown_clone.set_selected(idx);
                                        selected = true;
                                        break;
                                    }
                                }
                            }
                            if !selected {
                                monitor_selector_dropdown_clone.set_selected(0);
                            }
                        }
                        MonitorSelector::Description(desc) => {
                            let mut selected = false;
                            for idx in 0..monitor_selector_string_list_clone.n_items() {
                                if let Some(item) = monitor_selector_string_list_clone.item(idx) {
                                    let item_str = item.property::<String>("string");
                                    if &item_str == desc {
                                        monitor_selector_dropdown_clone.set_selected(idx);
                                        selected = true;
                                        break;
                                    }
                                }
                            }
                            if !selected {
                                monitor_selector_dropdown_clone.set_selected(0);
                            }
                        }
                    }
                }

                if skip != "resolution" {
                    let resolution_string_list = StringList::new(
                        &get_available_resolutions_for_monitor(monitor_selector)
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
                            match monitor_state.position {
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
                            match monitor_state.scale {
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
                                    for idx in 0..monitor_mirror_selector_string_list.n_items() {
                                        if let Some(item) =
                                            monitor_mirror_selector_string_list.item(idx)
                                        {
                                            let item_str = item.property::<String>("string");
                                            if &item_str == mirror {
                                                monitor_mirror_selector_dropdown_clone
                                                    .set_selected(idx);
                                                break;
                                            }
                                        }
                                    }
                                    monitor_mirror_selector_dropdown_clone.set_visible(true);
                                }
                                None => {
                                    monitor_mirror_onoff_switch_clone.set_active(false);
                                    monitor_mirror_selector_dropdown_clone.set_visible(false);
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

            load_monitor_values(&monitor_selector, &monitor, "");

            let load_monitor_values_clone = load_monitor_values.clone();
            let monitor_selector_string_list_clone = monitor_selector_string_list.clone();

            widget_connector!(
                is_updating,
                value_entry,
                monitor_selector_dropdown,
                connect_selected_notify,
                dropdown,
                dropdown.selected(),
                parse_monitor,
                |(_selector, monitor), new_selected: u32| {
                    let monitor_selector = match new_selected {
                        0 => MonitorSelector::All,
                        id => {
                            let item = monitor_selector_string_list_clone.item(id);
                            if let Some(obj) = item
                                && let Some(string_obj) = obj.downcast_ref::<gtk::StringObject>()
                            {
                                let string = string_obj.string();
                                let monitor_name = string.as_str();
                                MonitorSelector::from_str(monitor_name).unwrap_or_default()
                            } else {
                                MonitorSelector::All
                            }
                        }
                    };

                    load_monitor_values_clone(&monitor_selector, &monitor, "selector");

                    match monitor {
                        Monitor::Enabled(monitor_state) => {
                            format!("{}, {}", monitor_selector, monitor_state)
                        }
                        Monitor::AddReserved(
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right,
                        ) => format!(
                            "{}, addreserved, {}, {}, {}, {}",
                            monitor_selector,
                            monitor_addreserved_up,
                            monitor_addreserved_down,
                            monitor_addreserved_left,
                            monitor_addreserved_right
                        ),
                        Monitor::Disabled => format!("{}, disable", monitor_selector),
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
            let monitor_mirror_selector_dropdown_clone = monitor_mirror_selector_dropdown.clone();
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
                |(name, _monitor): (MonitorSelector, _), new_resolution: String| {
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
                                true => {
                                    if let Some(selected_item) =
                                        monitor_mirror_selector_dropdown_clone.selected_item()
                                        && let Some(string_obj) =
                                            selected_item.downcast_ref::<gtk::StringObject>()
                                    {
                                        Some(string_obj.string().to_string())
                                    } else {
                                        None
                                    }
                                }
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
                monitor_mirror_selector_dropdown,
                connect_selected_notify,
                dropdown,
                dropdown.selected(),
                parse_monitor,
                |(name, monitor), mirror_onoff: bool, _mirror_id: u32| {
                    let mirror = if let Some(selected_item) = dropdown.selected_item()
                        && let Some(string_obj) = selected_item.downcast_ref::<gtk::StringObject>()
                    {
                        Some(string_obj.string().to_string())
                    } else {
                        None
                    };

                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.mirror = if mirror_onoff { mirror } else { None };
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
                |(name, monitor), _mirror_id: u32| {
                    let mirror = if let Some(selected_item) = dropdown.selected_item()
                        && let Some(string_obj) = selected_item.downcast_ref::<gtk::StringObject>()
                    {
                        Some(string_obj.string().to_string())
                    } else {
                        None
                    };
                    match monitor {
                        Monitor::Enabled(mut monitor_state) => {
                            monitor_state.mirror = mirror;
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
        "workspace" => {
            let workspace_selector_box = Box::new(GtkOrientation::Horizontal, 5);

            let workspace_selector_type_box = Box::new(GtkOrientation::Horizontal, 5);
            workspace_selector_type_box.append(&Label::new(Some(&t!("advanced_editors.type"))));
            let workspace_selector_type_string_list = StringList::new(
                &WorkspaceType::get_fancy_list()
                    .each_ref()
                    .map(|s| s.as_str()),
            );
            let workspace_selector_type_dropdown =
                create_dropdown(&workspace_selector_type_string_list);
            workspace_selector_type_dropdown.set_selected(0);
            workspace_selector_type_box.append(&workspace_selector_type_dropdown);
            workspace_selector_box.append(&workspace_selector_type_box);

            let workspace_selector_name_entry = create_entry();
            workspace_selector_box.append(&workspace_selector_name_entry);

            let workspace_selector_number_spin = create_spin_button(1.0, i32::MAX as f64, 1.0);
            workspace_selector_number_spin.set_visible(false);
            workspace_selector_box.append(&workspace_selector_number_spin);

            let selectors_entry = create_entry();
            let selectors_ui_box = Vec::<WorkspaceSelector>::to_gtk_box(&selectors_entry, ' ');
            selectors_ui_box.set_visible(false);
            workspace_selector_box.append(&selectors_ui_box);

            fancy_value_entry.append(&workspace_selector_box);

            let is_updating_clone = is_updating.clone();
            let value_entry_clone = value_entry.clone();
            let workspace_selector_name_entry_clone = workspace_selector_name_entry.clone();
            let workspace_selector_number_spin_clone = workspace_selector_number_spin.clone();
            let selectors_ui_box_clone = selectors_ui_box.clone();
            let selectors_entry_clone = selectors_entry.clone();
            workspace_selector_type_dropdown.connect_selected_notify(move |dropdown| {
                if is_updating_clone.get() {
                    return;
                }
                is_updating_clone.set(true);

                let value = value_entry_clone.text().to_string();
                let mut workspace = parse_workspace(&value);
                workspace.workspace_type = match dropdown.selected() {
                    0 => {
                        workspace_selector_name_entry_clone.set_visible(true);
                        workspace_selector_number_spin_clone.set_visible(false);
                        selectors_ui_box_clone.set_visible(false);

                        WorkspaceType::Named(workspace_selector_name_entry_clone.text().to_string())
                    }
                    1 => {
                        workspace_selector_name_entry_clone.set_visible(true);
                        workspace_selector_number_spin_clone.set_visible(false);
                        selectors_ui_box_clone.set_visible(false);

                        WorkspaceType::Special(
                            workspace_selector_name_entry_clone.text().to_string(),
                        )
                    }
                    2 => {
                        workspace_selector_name_entry_clone.set_visible(false);
                        workspace_selector_number_spin_clone.set_visible(true);
                        selectors_ui_box_clone.set_visible(false);

                        WorkspaceType::Numbered(workspace_selector_number_spin_clone.value() as u32)
                    }
                    3 => {
                        workspace_selector_name_entry_clone.set_visible(false);
                        workspace_selector_number_spin_clone.set_visible(false);
                        selectors_ui_box_clone.set_visible(true);

                        let selectors = match workspace.workspace_type {
                            WorkspaceType::Selector(s) => s,
                            _ => Vec::new(),
                        };
                        let selectors_text = join_with_separator(&selectors, " ");
                        selectors_entry_clone.set_text(&selectors_text);

                        WorkspaceType::Selector(selectors)
                    }
                    _ => unreachable!(),
                };

                value_entry_clone.set_text(&workspace.to_string());

                is_updating_clone.set(false);
            });

            let is_updating_clone = is_updating.clone();
            let selectors_entry_clone = selectors_entry.clone();
            let workspace_selector_name_entry_clone = workspace_selector_name_entry.clone();
            let workspace_selector_number_spin_clone = workspace_selector_number_spin.clone();
            let workspace_selector_type_dropdown_clone = workspace_selector_type_dropdown.clone();
            let selectors_ui_box_clone = selectors_ui_box.clone();
            value_entry.connect_changed(move |entry| {
                if is_updating_clone.get() {
                    return;
                }
                is_updating_clone.set(true);

                let value = entry.text().to_string();
                let workspace = parse_workspace(&value);
                match workspace.workspace_type {
                    WorkspaceType::Named(name) => {
                        workspace_selector_name_entry_clone.set_visible(true);
                        workspace_selector_number_spin_clone.set_visible(false);
                        selectors_ui_box_clone.set_visible(false);

                        workspace_selector_name_entry_clone.set_text(&name);

                        workspace_selector_type_dropdown_clone.set_selected(0);
                    }
                    WorkspaceType::Special(name) => {
                        workspace_selector_name_entry_clone.set_visible(true);
                        workspace_selector_number_spin_clone.set_visible(false);
                        selectors_ui_box_clone.set_visible(false);

                        workspace_selector_name_entry_clone.set_text(&name);

                        workspace_selector_type_dropdown_clone.set_selected(1);
                    }
                    WorkspaceType::Numbered(number) => {
                        workspace_selector_name_entry_clone.set_visible(false);
                        workspace_selector_number_spin_clone.set_visible(true);
                        selectors_ui_box_clone.set_visible(false);

                        workspace_selector_number_spin_clone.set_value(number as f64);

                        workspace_selector_type_dropdown_clone.set_selected(2);
                    }
                    WorkspaceType::Selector(selectors) => {
                        workspace_selector_name_entry_clone.set_visible(false);
                        workspace_selector_number_spin_clone.set_visible(false);
                        selectors_ui_box_clone.set_visible(true);

                        let selectors_text = join_with_separator(&selectors, " ");
                        selectors_entry_clone.set_text(&selectors_text);

                        workspace_selector_type_dropdown_clone.set_selected(3);
                    }
                }
                is_updating_clone.set(false);
            });

            let is_updating_clone = is_updating.clone();
            let value_entry_clone = value_entry.clone();
            selectors_entry.connect_changed(move |entry| {
                if is_updating_clone.get() {
                    return;
                }
                is_updating_clone.set(true);

                let workspace_str = value_entry_clone.text().to_string();
                let (_selectors, rules) = workspace_str.split_once(',').unwrap_or(("", ""));
                value_entry_clone.set_text(&format!("{}, {}", entry.text(), rules));

                is_updating_clone.set(false);
            });

            let workspace_rules_box = Box::new(GtkOrientation::Vertical, 5);
            workspace_rules_box.set_margin_top(10);
            fancy_value_entry.append(&workspace_rules_box);

            let workspace_rules_label = Label::new(Some(&t!("advanced_editors.workspace_rules")));
            workspace_rules_label.set_markup(&format!(
                "<b>{}</b>",
                t!("advanced_editors.workspace_rules")
            ));
            workspace_rules_label.set_halign(gtk::Align::Start);
            workspace_rules_label.set_margin_bottom(5);
            workspace_rules_box.append(&workspace_rules_label);

            let separator = Separator::new(GtkOrientation::Horizontal);
            separator.set_margin_bottom(10);
            workspace_rules_box.append(&separator);

            let monitor_box = Box::new(GtkOrientation::Horizontal, 5);
            monitor_box.append(&Label::new(Some(&t!("advanced_editors.monitor"))));
            let monitor_onoff_switch = create_switch();
            monitor_box.append(&monitor_onoff_switch);
            let available_monitors = get_available_monitors(true);
            let monitor_list = available_monitors
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>();
            let monitor_string_list = StringList::new(&monitor_list);
            let monitor_dropdown = create_dropdown(&monitor_string_list);
            monitor_dropdown.set_visible(false);
            monitor_box.append(&monitor_dropdown);
            workspace_rules_box.append(&monitor_box);

            let default_box = Box::new(GtkOrientation::Horizontal, 5);
            default_box.append(&Label::new(Some(&t!("advanced_editors.default"))));
            let default_onoff_switch = create_switch();
            default_box.append(&default_onoff_switch);
            let default_value_switch = create_switch();
            default_value_switch.set_visible(false);
            default_box.append(&default_value_switch);
            workspace_rules_box.append(&default_box);

            let gaps_in_box = Box::new(GtkOrientation::Horizontal, 5);
            gaps_in_box.append(&Label::new(Some(&t!(
                "widget.general_category.gaps_in_label"
            ))));
            let gaps_in_onoff_switch = create_switch();
            gaps_in_box.append(&gaps_in_onoff_switch);
            let gaps_in_spin = create_spin_button(0.0, 200.0, 1.0);
            gaps_in_spin.set_visible(false);
            gaps_in_box.append(&gaps_in_spin);
            workspace_rules_box.append(&gaps_in_box);

            let gaps_out_box = Box::new(GtkOrientation::Horizontal, 5);
            gaps_out_box.append(&Label::new(Some(&t!(
                "widget.general_category.gaps_out_label"
            ))));
            let gaps_out_onoff_switch = create_switch();
            gaps_out_box.append(&gaps_out_onoff_switch);
            let gaps_out_spin = create_spin_button(0.0, 200.0, 1.0);
            gaps_out_spin.set_visible(false);
            gaps_out_box.append(&gaps_out_spin);
            workspace_rules_box.append(&gaps_out_box);

            let border_size_box = Box::new(GtkOrientation::Horizontal, 5);
            border_size_box.append(&Label::new(Some(&t!(
                "widget.general_category.border_size_label"
            ))));
            let border_size_onoff_switch = create_switch();
            border_size_box.append(&border_size_onoff_switch);
            let border_size_spin = create_spin_button(0.0, 20.0, 1.0);
            border_size_spin.set_visible(false);
            border_size_box.append(&border_size_spin);
            workspace_rules_box.append(&border_size_box);

            let border_box = Box::new(GtkOrientation::Horizontal, 5);
            border_box.append(&Label::new(Some(&t!("advanced_editors.border"))));
            let border_onoff_switch = create_switch();
            border_box.append(&border_onoff_switch);
            let border_value_switch = create_switch();
            border_value_switch.set_visible(false);
            border_box.append(&border_value_switch);
            workspace_rules_box.append(&border_box);

            let shadow_box = Box::new(GtkOrientation::Horizontal, 5);
            shadow_box.append(&Label::new(Some(&t!("advanced_editors.shadow"))));
            let shadow_onoff_switch = create_switch();
            shadow_box.append(&shadow_onoff_switch);
            let shadow_value_switch = create_switch();
            shadow_value_switch.set_visible(false);
            shadow_box.append(&shadow_value_switch);
            workspace_rules_box.append(&shadow_box);

            let rounding_box = Box::new(GtkOrientation::Horizontal, 5);
            rounding_box.append(&Label::new(Some(&t!("advanced_editors.rounding"))));
            let rounding_onoff_switch = create_switch();
            rounding_box.append(&rounding_onoff_switch);
            let rounding_value_switch = create_switch();
            rounding_value_switch.set_visible(false);
            rounding_box.append(&rounding_value_switch);
            workspace_rules_box.append(&rounding_box);

            let decorate_box = Box::new(GtkOrientation::Horizontal, 5);
            decorate_box.append(&Label::new(Some(&t!("advanced_editors.decorate"))));
            let decorate_onoff_switch = create_switch();
            decorate_box.append(&decorate_onoff_switch);
            let decorate_value_switch = create_switch();
            decorate_value_switch.set_visible(false);
            decorate_box.append(&decorate_value_switch);
            workspace_rules_box.append(&decorate_box);

            let persistent_box = Box::new(GtkOrientation::Horizontal, 5);
            persistent_box.append(&Label::new(Some(&t!("advanced_editors.persistent"))));
            let persistent_onoff_switch = create_switch();
            persistent_box.append(&persistent_onoff_switch);
            let persistent_value_switch = create_switch();
            persistent_value_switch.set_visible(false);
            persistent_box.append(&persistent_value_switch);
            workspace_rules_box.append(&persistent_box);

            let on_created_empty_box = Box::new(GtkOrientation::Horizontal, 5);
            on_created_empty_box
                .append(&Label::new(Some(&t!("advanced_editors.on_created_empty"))));
            let on_created_empty_onoff_switch = create_switch();
            on_created_empty_box.append(&on_created_empty_onoff_switch);
            let on_created_empty_entry = create_entry();
            on_created_empty_entry
                .set_placeholder_text(Some(&t!("advanced_editors.command_to_execute")));
            on_created_empty_entry.set_visible(false);
            on_created_empty_box.append(&on_created_empty_entry);
            workspace_rules_box.append(&on_created_empty_box);

            let default_name_box = Box::new(GtkOrientation::Horizontal, 5);
            default_name_box.append(&Label::new(Some(&t!("advanced_editors.default_name"))));
            let default_name_onoff_switch = create_switch();
            default_name_box.append(&default_name_onoff_switch);
            let default_name_entry = create_entry();
            default_name_entry
                .set_placeholder_text(Some(&t!("advanced_editors.default_workspace_name")));
            default_name_entry.set_visible(false);
            default_name_box.append(&default_name_entry);
            workspace_rules_box.append(&default_name_box);

            let layoutopt_orientation_box = Box::new(GtkOrientation::Horizontal, 5);
            layoutopt_orientation_box.append(&Label::new(Some(&t!(
                "advanced_editors.layoutopt_orientation"
            ))));
            let layoutopt_orientation_onoff_switch = create_switch();
            layoutopt_orientation_box.append(&layoutopt_orientation_onoff_switch);
            let orientation_string_list = StringList::new(&[
                &t!("advanced_editors.left"),
                &t!("advanced_editors.right"),
                &t!("advanced_editors.top"),
                &t!("advanced_editors.bottom"),
                &t!("advanced_editors.center"),
            ]);
            let layoutopt_orientation_dropdown = create_dropdown(&orientation_string_list);
            layoutopt_orientation_dropdown.set_visible(false);
            layoutopt_orientation_box.append(&layoutopt_orientation_dropdown);
            workspace_rules_box.append(&layoutopt_orientation_box);

            let is_updating = Rc::new(Cell::new(false));

            optional_widget_connector!(
                is_updating,
                value_entry,
                monitor_onoff_switch,
                monitor_dropdown,
                connect_selected_notify,
                dropdown,
                dropdown
                    .selected_item()
                    .and_then(|item| item
                        .downcast_ref::<StringObject>()
                        .map(|obj| obj.string().to_string()))
                    .unwrap_or_default(),
                parse_workspace,
                |mut workspace: Workspace, monitor_onoff: bool, monitor_value: String| {
                    if monitor_onoff && !monitor_value.is_empty() {
                        workspace.rules.monitor = Some(monitor_value);
                    } else {
                        workspace.rules.monitor = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, monitor_value: String| {
                    if !monitor_value.is_empty() {
                        workspace.rules.monitor = Some(monitor_value);
                    }
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                default_onoff_switch,
                default_value_switch,
                connect_state_notify,
                switch,
                switch.is_active(),
                parse_workspace,
                |mut workspace: Workspace, default_onoff: bool, default_value: bool| {
                    if default_onoff {
                        workspace.rules.default = Some(default_value);
                    } else {
                        workspace.rules.default = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, default_value: bool| {
                    workspace.rules.default = Some(default_value);
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                gaps_in_onoff_switch,
                gaps_in_spin,
                connect_value_changed,
                spin,
                spin.value() as i32,
                parse_workspace,
                |mut workspace: Workspace, gaps_in_onoff: bool, gaps_in_value: i32| {
                    if gaps_in_onoff {
                        workspace.rules.gaps_in = Some(gaps_in_value);
                    } else {
                        workspace.rules.gaps_in = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, gaps_in_value: i32| {
                    workspace.rules.gaps_in = Some(gaps_in_value);
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                gaps_out_onoff_switch,
                gaps_out_spin,
                connect_value_changed,
                spin,
                spin.value() as i32,
                parse_workspace,
                |mut workspace: Workspace, gaps_out_onoff: bool, gaps_out_value: i32| {
                    if gaps_out_onoff {
                        workspace.rules.gaps_out = Some(gaps_out_value);
                    } else {
                        workspace.rules.gaps_out = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, gaps_out_value: i32| {
                    workspace.rules.gaps_out = Some(gaps_out_value);
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                border_size_onoff_switch,
                border_size_spin,
                connect_value_changed,
                spin,
                spin.value() as i32,
                parse_workspace,
                |mut workspace: Workspace, border_size_onoff: bool, border_size_value: i32| {
                    if border_size_onoff {
                        workspace.rules.border_size = Some(border_size_value);
                    } else {
                        workspace.rules.border_size = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, border_size_value: i32| {
                    workspace.rules.border_size = Some(border_size_value);
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                border_onoff_switch,
                border_value_switch,
                connect_state_notify,
                switch,
                switch.is_active(),
                parse_workspace,
                |mut workspace: Workspace, border_onoff: bool, border_value: bool| {
                    if border_onoff {
                        workspace.rules.border = Some(border_value);
                    } else {
                        workspace.rules.border = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, border_value: bool| {
                    workspace.rules.border = Some(border_value);
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                shadow_onoff_switch,
                shadow_value_switch,
                connect_state_notify,
                switch,
                switch.is_active(),
                parse_workspace,
                |mut workspace: Workspace, shadow_onoff: bool, shadow_value: bool| {
                    if shadow_onoff {
                        workspace.rules.shadow = Some(shadow_value);
                    } else {
                        workspace.rules.shadow = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, shadow_value: bool| {
                    workspace.rules.shadow = Some(shadow_value);
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                rounding_onoff_switch,
                rounding_value_switch,
                connect_state_notify,
                switch,
                switch.is_active(),
                parse_workspace,
                |mut workspace: Workspace, rounding_onoff: bool, rounding_value: bool| {
                    if rounding_onoff {
                        workspace.rules.rounding = Some(rounding_value);
                    } else {
                        workspace.rules.rounding = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, rounding_value: bool| {
                    workspace.rules.rounding = Some(rounding_value);
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                decorate_onoff_switch,
                decorate_value_switch,
                connect_state_notify,
                switch,
                switch.is_active(),
                parse_workspace,
                |mut workspace: Workspace, decorate_onoff: bool, decorate_value: bool| {
                    if decorate_onoff {
                        workspace.rules.decorate = Some(decorate_value);
                    } else {
                        workspace.rules.decorate = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, decorate_value: bool| {
                    workspace.rules.decorate = Some(decorate_value);
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                persistent_onoff_switch,
                persistent_value_switch,
                connect_state_notify,
                switch,
                switch.is_active(),
                parse_workspace,
                |mut workspace: Workspace, persistent_onoff: bool, persistent_value: bool| {
                    if persistent_onoff {
                        workspace.rules.persistent = Some(persistent_value);
                    } else {
                        workspace.rules.persistent = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, persistent_value: bool| {
                    workspace.rules.persistent = Some(persistent_value);
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                on_created_empty_onoff_switch,
                on_created_empty_entry,
                connect_changed,
                entry,
                entry.text().to_string(),
                parse_workspace,
                |mut workspace: Workspace,
                 on_created_empty_onoff: bool,
                 on_created_empty_value: String| {
                    if on_created_empty_onoff && !on_created_empty_value.is_empty() {
                        workspace.rules.on_created_empty = Some(on_created_empty_value);
                    } else {
                        workspace.rules.on_created_empty = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, on_created_empty_value: String| {
                    if !on_created_empty_value.is_empty() {
                        workspace.rules.on_created_empty = Some(on_created_empty_value);
                    }
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                default_name_onoff_switch,
                default_name_entry,
                connect_changed,
                entry,
                entry.text().to_string(),
                parse_workspace,
                |mut workspace: Workspace, default_name_onoff: bool, default_name_value: String| {
                    if default_name_onoff && !default_name_value.is_empty() {
                        workspace.rules.default_name = Some(default_name_value);
                    } else {
                        workspace.rules.default_name = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, default_name_value: String| {
                    if !default_name_value.is_empty() {
                        workspace.rules.default_name = Some(default_name_value);
                    }
                    workspace.to_string()
                }
            );

            optional_widget_connector!(
                is_updating,
                value_entry,
                layoutopt_orientation_onoff_switch,
                layoutopt_orientation_dropdown,
                connect_selected_notify,
                dropdown,
                match dropdown.selected() {
                    0 => Orientation::Left,
                    1 => Orientation::Right,
                    2 => Orientation::Top,
                    3 => Orientation::Bottom,
                    _ => Orientation::Center,
                },
                parse_workspace,
                |mut workspace: Workspace,
                 layoutopt_orientation_onoff: bool,
                 layoutopt_orientation_value: Orientation| {
                    if layoutopt_orientation_onoff {
                        workspace.rules.layoutopt_orientation = Some(layoutopt_orientation_value);
                    } else {
                        workspace.rules.layoutopt_orientation = None;
                    }
                    workspace.to_string()
                },
                |mut workspace: Workspace, layoutopt_orientation_value: Orientation| {
                    workspace.rules.layoutopt_orientation = Some(layoutopt_orientation_value);
                    workspace.to_string()
                }
            );

            value_entry.connect_changed(move |entry| {
                if is_updating.get() {
                    return;
                }
                is_updating.set(true);

                let workspace = parse_workspace(&entry.text());

                if let Some(monitor) = &workspace.rules.monitor {
                    monitor_onoff_switch.set_active(true);
                    monitor_dropdown.set_visible(true);
                    for idx in 0..monitor_string_list.n_items() {
                        if let Some(item) = monitor_string_list.item(idx)
                            && let Some(string_object) = item.downcast_ref::<StringObject>()
                            && &string_object.string() == monitor
                        {
                            monitor_dropdown.set_selected(idx);
                            break;
                        }
                    }
                } else {
                    monitor_onoff_switch.set_active(false);
                    monitor_dropdown.set_visible(false);
                }

                if let Some(default) = workspace.rules.default {
                    default_onoff_switch.set_active(true);
                    default_value_switch.set_visible(true);
                    default_value_switch.set_active(default);
                } else {
                    default_onoff_switch.set_active(false);
                    default_value_switch.set_visible(false);
                }

                if let Some(gaps_in) = workspace.rules.gaps_in {
                    gaps_in_onoff_switch.set_active(true);
                    gaps_in_spin.set_visible(true);
                    gaps_in_spin.set_value(gaps_in as f64);
                } else {
                    gaps_in_onoff_switch.set_active(false);
                    gaps_in_spin.set_visible(false);
                }

                if let Some(gaps_out) = workspace.rules.gaps_out {
                    gaps_out_onoff_switch.set_active(true);
                    gaps_out_spin.set_visible(true);
                    gaps_out_spin.set_value(gaps_out as f64);
                } else {
                    gaps_out_onoff_switch.set_active(false);
                    gaps_out_spin.set_visible(false);
                }

                if let Some(border_size) = workspace.rules.border_size {
                    border_size_onoff_switch.set_active(true);
                    border_size_spin.set_visible(true);
                    border_size_spin.set_value(border_size as f64);
                } else {
                    border_size_onoff_switch.set_active(false);
                    border_size_spin.set_visible(false);
                }

                if let Some(border) = workspace.rules.border {
                    border_onoff_switch.set_active(true);
                    border_value_switch.set_visible(true);
                    border_value_switch.set_active(border);
                } else {
                    border_onoff_switch.set_active(false);
                    border_value_switch.set_visible(false);
                }

                if let Some(shadow) = workspace.rules.shadow {
                    shadow_onoff_switch.set_active(true);
                    shadow_value_switch.set_visible(true);
                    shadow_value_switch.set_active(shadow);
                } else {
                    shadow_onoff_switch.set_active(false);
                    shadow_value_switch.set_visible(false);
                }

                if let Some(rounding) = workspace.rules.rounding {
                    rounding_onoff_switch.set_active(true);
                    rounding_value_switch.set_visible(true);
                    rounding_value_switch.set_active(rounding);
                } else {
                    rounding_onoff_switch.set_active(false);
                    rounding_value_switch.set_visible(false);
                }

                if let Some(decorate) = workspace.rules.decorate {
                    decorate_onoff_switch.set_active(true);
                    decorate_value_switch.set_visible(true);
                    decorate_value_switch.set_active(decorate);
                } else {
                    decorate_onoff_switch.set_active(false);
                    decorate_value_switch.set_visible(false);
                }

                if let Some(persistent) = workspace.rules.persistent {
                    persistent_onoff_switch.set_active(true);
                    persistent_value_switch.set_visible(true);
                    persistent_value_switch.set_active(persistent);
                } else {
                    persistent_onoff_switch.set_active(false);
                    persistent_value_switch.set_visible(false);
                }

                if let Some(on_created_empty) = &workspace.rules.on_created_empty {
                    on_created_empty_onoff_switch.set_active(true);
                    on_created_empty_entry.set_visible(true);
                    on_created_empty_entry.set_text(on_created_empty);
                } else {
                    on_created_empty_onoff_switch.set_active(false);
                    on_created_empty_entry.set_visible(false);
                }

                if let Some(default_name) = &workspace.rules.default_name {
                    default_name_onoff_switch.set_active(true);
                    default_name_entry.set_visible(true);
                    default_name_entry.set_text(default_name);
                } else {
                    default_name_onoff_switch.set_active(false);
                    default_name_entry.set_visible(false);
                }

                if let Some(layoutopt_orientation) = &workspace.rules.layoutopt_orientation {
                    layoutopt_orientation_onoff_switch.set_active(true);
                    layoutopt_orientation_dropdown.set_visible(true);
                    let index = match layoutopt_orientation {
                        Orientation::Left => 0,
                        Orientation::Right => 1,
                        Orientation::Top => 2,
                        Orientation::Bottom => 3,
                        Orientation::Center => 4,
                    };
                    layoutopt_orientation_dropdown.set_selected(index);
                } else {
                    layoutopt_orientation_onoff_switch.set_active(false);
                    layoutopt_orientation_dropdown.set_visible(false);
                }

                is_updating.set(false);
            });

            value_entry.set_text(&value_entry.text());
        }
        "animation" => {
            if name == "bezier" {
                let bezier = parse_bezier(&value_entry.text());
                let bezier_name_entry = create_entry();
                bezier_name_entry.set_text(&bezier.name);
                let bezier_x0_spin = create_spin_button(0.0, 1.0, 0.01);
                bezier_x0_spin.set_value(bezier.x0);
                bezier_x0_spin.set_digits(3);
                let bezier_y0_spin = create_spin_button(-10.0, 10.0, 0.01);
                bezier_y0_spin.set_value(bezier.y0);
                bezier_y0_spin.set_digits(3);
                let bezier_x1_spin = create_spin_button(0.0, 1.0, 0.01);
                bezier_x1_spin.set_value(bezier.x1);
                bezier_x1_spin.set_digits(3);
                let bezier_y1_spin = create_spin_button(-10.0, 10.0, 0.01);
                bezier_y1_spin.set_value(bezier.y1);
                bezier_y1_spin.set_digits(3);

                widget_connector!(
                    is_updating,
                    value_entry,
                    bezier_name_entry,
                    connect_changed,
                    entry,
                    entry.text().to_string(),
                    parse_bezier,
                    |bezier, new_name: String| HyprBezierCurve {
                        name: new_name,
                        ..bezier
                    }
                    .to_string()
                );
                widget_connector!(
                    is_updating,
                    value_entry,
                    bezier_x0_spin,
                    connect_value_changed,
                    spin,
                    spin.value(),
                    parse_bezier,
                    |mut bezier: HyprBezierCurve, new_x0: f64| {
                        bezier.x0 = new_x0;
                        bezier.to_string()
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
                    |mut bezier: HyprBezierCurve, new_y0: f64| {
                        bezier.y0 = new_y0;
                        bezier.to_string()
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
                    |mut bezier: HyprBezierCurve, new_x1: f64| {
                        bezier.x1 = new_x1;
                        bezier.to_string()
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
                    |mut bezier: HyprBezierCurve, new_y1: f64| {
                        bezier.y1 = new_y1;
                        bezier.to_string()
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
                        let bezier = parse_bezier(&entry.text());
                        bezier_name_entry_clone.set_text(&bezier.name);
                        bezier_x0_spin_clone.set_value(bezier.x0);
                        bezier_y0_spin_clone.set_value(bezier.y0);
                        bezier_x1_spin_clone.set_value(bezier.x1);
                        bezier_y1_spin_clone.set_value(bezier.y1);
                        is_updating_clone.set(false);
                    });
                }

                fancy_value_entry.append(&Label::new(Some(&t!("advanced_editors.name"))));
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
                let animation = parse_animation(&value_entry.text());

                let animation_names = AnimationName::get_list();
                let fancy_animation_names = AnimationName::get_fancy_list();
                let animation_name_string_list = StringList::new(
                    &fancy_animation_names
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>(),
                );
                let animation_name_dropdown = create_dropdown(&animation_name_string_list);

                let current_name_idx = animation_names
                    .iter()
                    .position(|&name| name == animation.name.to_string())
                    .unwrap_or(0);
                animation_name_dropdown.set_selected(current_name_idx as u32);

                let animation_onoff_switch = create_switch();
                animation_onoff_switch.set_active(animation.enabled);

                let animation_speed_spin = create_spin_button(0.1, 100.0, 0.1);
                animation_speed_spin.set_value(animation.speed);
                animation_speed_spin.set_digits(1);

                let animation_curve_entry = create_entry();
                animation_curve_entry.set_text(&animation.curve);

                let style_label = Label::new(Some(&t!("advanced_editors.style")));
                style_label.set_visible(false);

                let animation_style_box = Box::new(GtkOrientation::Vertical, 5);
                animation_style_box.set_visible(false);

                let available_styles = animation
                    .name
                    .get_fancy_available_styles()
                    .unwrap_or_else(|| vec![t!("advanced_editors.none").to_string()]);
                let style_string_list = StringList::new(
                    &available_styles
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>(),
                );
                let animation_style_dropdown = create_dropdown(&style_string_list);

                let current_style_idx =
                    animation.name.get_id_of_style(animation.style).unwrap_or(0);
                animation_style_dropdown.set_selected(current_style_idx as u32);

                animation_style_box.append(&animation_style_dropdown);

                let style_params_box = Box::new(GtkOrientation::Vertical, 5);
                style_params_box.set_margin_start(20);

                let side_label = Label::new(Some(&t!("advanced_editors.side")));
                side_label.set_halign(Align::Start);
                let side_string_list =
                    StringList::new(&Side::get_fancy_list().each_ref().map(|s| s.as_str()));
                let side_dropdown = create_dropdown(&side_string_list);

                let percent_label = Label::new(Some(&t!("advanced_editors.percentage")));
                percent_label.set_halign(Align::Start);
                let percent_spin = create_spin_button(0.0, 100.0, 1.0);
                percent_spin.set_digits(1);

                let side_label_clone = side_label.clone();
                let side_dropdown_clone = side_dropdown.clone();
                let percent_label_clone = percent_label.clone();
                let percent_spin_clone = percent_spin.clone();
                let style_params_box_clone = style_params_box.clone();

                let update_style_params_visibility = move |style: &AnimationStyle| {
                    side_dropdown_clone.set_visible(false);
                    percent_spin_clone.set_visible(false);

                    match style {
                        AnimationStyle::SlideSide(_) => {
                            side_label_clone.set_visible(true);
                            side_dropdown_clone.set_visible(true);
                            style_params_box_clone.set_visible(true);

                            percent_label_clone.set_visible(false);
                            percent_spin_clone.set_visible(false);
                        }
                        AnimationStyle::PopinPercent(_)
                        | AnimationStyle::SlidePercent(_)
                        | AnimationStyle::SlideVertPercent(_)
                        | AnimationStyle::SlideFadePercent(_)
                        | AnimationStyle::SlideFadeVertPercent(_) => {
                            percent_label_clone.set_visible(true);
                            percent_spin_clone.set_visible(true);
                            style_params_box_clone.set_visible(true);

                            side_label_clone.set_visible(false);
                            side_dropdown_clone.set_visible(false);
                        }
                        _ => {
                            style_params_box_clone.set_visible(false);
                        }
                    }
                };

                update_style_params_visibility(&animation.style);

                style_params_box.append(&side_label);
                style_params_box.append(&side_dropdown);
                style_params_box.append(&percent_label);
                style_params_box.append(&percent_spin);

                animation_style_box.append(&style_params_box);

                let style_label_clone = style_label.clone();
                let animation_style_box_clone = animation_style_box.clone();
                let animation_style_dropdown_clone = animation_style_dropdown.clone();
                let update_style_params_visibility_clone = update_style_params_visibility.clone();

                let update_available_styles =
                    move |animation_name: AnimationName| match animation_name
                        .get_fancy_available_styles()
                    {
                        Some(styles) => {
                            let style_string_list = StringList::new(
                                &styles.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
                            );

                            animation_style_dropdown_clone.set_model(Some(&style_string_list));
                            animation_style_dropdown_clone.set_selected(0);

                            if let Some(styles) = animation_name.get_available_styles()
                                && let Some(first_style) = styles.first()
                            {
                                update_style_params_visibility_clone(first_style);
                            }
                            style_label_clone.set_visible(true);
                            animation_style_box_clone.set_visible(true);
                        }
                        None => {
                            style_label_clone.set_visible(false);
                            animation_style_box_clone.set_visible(false);
                        }
                    };

                if let AnimationStyle::SlideSide(side) = animation.style {
                    side_dropdown.set_selected(side.get_id() as u32);
                }

                match animation.style {
                    AnimationStyle::PopinPercent(percent)
                    | AnimationStyle::SlidePercent(percent)
                    | AnimationStyle::SlideVertPercent(percent)
                    | AnimationStyle::SlideFadePercent(percent)
                    | AnimationStyle::SlideFadeVertPercent(percent) => {
                        percent_spin.set_value(percent);
                    }
                    _ => {}
                }

                let value_entry_clone = value_entry.clone();
                let is_updating_clone = is_updating.clone();
                let update_style_params_visibility_clone = update_style_params_visibility.clone();
                let update_available_styles_clone = update_available_styles.clone();

                animation_name_dropdown.connect_selected_notify(move |dropdown| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);

                    let selected_idx = dropdown.selected() as usize;
                    if selected_idx < animation_names.len() {
                        let selected_name = &animation_names[selected_idx];
                        let animation_name =
                            AnimationName::from_str(selected_name).unwrap_or_default();

                        update_available_styles_clone(animation_name);

                        let current_animation = parse_animation(&value_entry_clone.text());
                        let mut new_animation = current_animation;
                        new_animation.name = animation_name;

                        if let Some(styles) = new_animation.name.get_available_styles()
                            && let Some(first_style) = styles.first()
                        {
                            new_animation.style = *first_style;
                            update_style_params_visibility_clone(&new_animation.style);
                        }

                        value_entry_clone.set_text(&new_animation.to_string());
                    }

                    is_updating_clone.set(false);
                });

                let animation_name_dropdown_clone = animation_name_dropdown.clone();
                let value_entry_clone = value_entry.clone();
                let is_updating_clone = is_updating.clone();
                let side_dropdown_clone = side_dropdown.clone();
                let percent_spin_clone = percent_spin.clone();
                let update_style_params_visibility_clone = update_style_params_visibility.clone();

                animation_style_dropdown.connect_selected_notify(move |dropdown| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);

                    let current_animation = parse_animation(&value_entry_clone.text());
                    let mut new_animation = current_animation;

                    let selected_name_idx = animation_name_dropdown_clone.selected() as usize;
                    if selected_name_idx < animation_names.len() {
                        let selected_name = &animation_names[selected_name_idx];
                        let animation_name =
                            AnimationName::from_str(selected_name).unwrap_or_default();
                        new_animation.name = animation_name;

                        if let Some(styles) = new_animation.name.get_available_styles() {
                            let selected_style_idx = dropdown.selected() as usize;
                            if selected_style_idx < styles.len() {
                                new_animation.style = styles[selected_style_idx];
                                update_style_params_visibility_clone(&new_animation.style);

                                match &mut new_animation.style {
                                    AnimationStyle::SlideSide(side) => {
                                        side_dropdown_clone.set_selected(side.get_id() as u32);
                                    }
                                    AnimationStyle::PopinPercent(percent)
                                    | AnimationStyle::SlidePercent(percent)
                                    | AnimationStyle::SlideVertPercent(percent)
                                    | AnimationStyle::SlideFadePercent(percent)
                                    | AnimationStyle::SlideFadeVertPercent(percent) => {
                                        percent_spin_clone.set_value(*percent);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    value_entry_clone.set_text(&new_animation.to_string());
                    is_updating_clone.set(false);
                });

                let value_entry_clone = value_entry.clone();
                let is_updating_clone = is_updating.clone();
                let update_style_params_visibility_clone = update_style_params_visibility.clone();

                side_dropdown.connect_selected_notify(move |dropdown| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);

                    let current_animation = parse_animation(&value_entry_clone.text());
                    let mut new_animation = current_animation;

                    if let AnimationStyle::SlideSide(_) = new_animation.style {
                        let selected_side = Side::get_list()[dropdown.selected() as usize];
                        new_animation.style =
                            AnimationStyle::SlideSide(Side::from_str(selected_side).unwrap());
                        update_style_params_visibility_clone(&new_animation.style);
                        value_entry_clone.set_text(&new_animation.to_string());
                    }

                    is_updating_clone.set(false);
                });

                let value_entry_clone = value_entry.clone();
                let is_updating_clone = is_updating.clone();
                let update_style_params_visibility_clone = update_style_params_visibility.clone();

                percent_spin.connect_value_changed(move |spin| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);

                    let current_animation = parse_animation(&value_entry_clone.text());
                    let mut new_animation = current_animation;
                    let new_percent = spin.value();

                    match &mut new_animation.style {
                        AnimationStyle::PopinPercent(p)
                        | AnimationStyle::SlidePercent(p)
                        | AnimationStyle::SlideVertPercent(p)
                        | AnimationStyle::SlideFadePercent(p)
                        | AnimationStyle::SlideFadeVertPercent(p) => {
                            *p = new_percent;
                            update_style_params_visibility_clone(&new_animation.style);
                            value_entry_clone.set_text(&new_animation.to_string());
                        }
                        _ => {}
                    }

                    is_updating_clone.set(false);
                });

                widget_connector!(
                    is_updating,
                    value_entry,
                    animation_onoff_switch,
                    connect_state_set,
                    _switch_widget,
                    state,
                    state,
                    parse_animation,
                    |mut animation: Animation, new_enabled: bool| {
                        animation.enabled = new_enabled;
                        animation.to_string()
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
                    |mut animation: Animation, new_speed: f64| {
                        animation.speed = new_speed;
                        animation.to_string()
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
                    |mut animation: Animation, new_curve: String| {
                        animation.curve = new_curve;
                        animation.to_string()
                    }
                );

                let is_updating_clone = is_updating.clone();
                let animation_name_dropdown_clone = animation_name_dropdown.clone();
                let animation_onoff_switch_clone = animation_onoff_switch.clone();
                let animation_speed_spin_clone = animation_speed_spin.clone();
                let animation_curve_entry_clone = animation_curve_entry.clone();
                let animation_style_dropdown_clone = animation_style_dropdown.clone();
                let side_dropdown_clone = side_dropdown.clone();
                let percent_spin_clone = percent_spin.clone();
                let update_style_params_visibility_clone = update_style_params_visibility.clone();
                let update_available_styles_clone = update_available_styles.clone();

                value_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);

                    let animation = parse_animation(&entry.text());

                    let name_idx = animation_names
                        .iter()
                        .position(|&name| name == animation.name.to_string())
                        .unwrap_or(0);
                    animation_name_dropdown_clone.set_selected(name_idx as u32);
                    animation_onoff_switch_clone.set_active(animation.enabled);
                    animation_speed_spin_clone.set_value(animation.speed);
                    animation_curve_entry_clone.set_text(&animation.curve);

                    let style_idx = animation.name.get_id_of_style(animation.style).unwrap_or(0);
                    animation_style_dropdown_clone.set_selected(style_idx as u32);

                    update_style_params_visibility_clone(&animation.style);

                    if let AnimationStyle::SlideSide(side) = animation.style {
                        side_dropdown_clone.set_selected(side.get_id() as u32);
                    }

                    match animation.style {
                        AnimationStyle::PopinPercent(percent)
                        | AnimationStyle::SlidePercent(percent)
                        | AnimationStyle::SlideVertPercent(percent)
                        | AnimationStyle::SlideFadePercent(percent)
                        | AnimationStyle::SlideFadeVertPercent(percent) => {
                            percent_spin_clone.set_value(percent);
                        }
                        _ => {}
                    }

                    update_available_styles_clone(animation.name);

                    is_updating_clone.set(false);
                });

                fancy_value_entry.append(&Label::new(Some(&t!("advanced_editors.name"))));
                fancy_value_entry.append(&animation_name_dropdown);

                fancy_value_entry.append(&Label::new(Some(&t!("advanced_editors.onoff"))));
                fancy_value_entry.append(&animation_onoff_switch);

                fancy_value_entry.append(&Label::new(Some(&t!("advanced_editors.speed"))));
                fancy_value_entry.append(&animation_speed_spin);

                fancy_value_entry.append(&Label::new(Some(&t!("advanced_editors.curve"))));
                fancy_value_entry.append(&animation_curve_entry);

                fancy_value_entry.append(&style_label);
                fancy_value_entry.append(&animation_style_box);
            }
        }
        "bind" => {
            let bind_box = Box::new(GtkOrientation::Vertical, 5);

            let bind_left =
                BindLeft::from_str(name).unwrap_or(BindLeft::Bind(BindFlags::default()));

            let (is_unbind, has_description) = if let BindLeft::Bind(flags) = bind_left {
                (false, flags.has_description)
            } else {
                (true, false)
            };

            if is_unbind {
                let unbind_right = match UnbindRight::from_str(&value_entry.text()) {
                    Ok(unbind) => unbind,
                    Err(_) => UnbindRight {
                        mods: HashSet::new(),
                        key: "".to_string(),
                    },
                };

                let mods_box = Box::new(GtkOrientation::Horizontal, 5);
                mods_box.append(&Label::new(Some(&t!("advanced_editors.modifiers"))));

                let modifier_names = [
                    Modifier::Shift,
                    Modifier::Ctrl,
                    Modifier::Alt,
                    Modifier::Super,
                    Modifier::Caps,
                    Modifier::Mod2,
                    Modifier::Mod3,
                    Modifier::Mod5,
                ];

                let mut modifier_switches = Vec::new();
                for modifier in &modifier_names {
                    let mod_box = Box::new(GtkOrientation::Horizontal, 5);
                    let switch = create_switch();
                    switch.set_active(unbind_right.mods.contains(modifier));
                    mod_box.append(&Label::new(Some(&modifier.to_string())));
                    mod_box.append(&switch);
                    mods_box.append(&mod_box);
                    modifier_switches.push((*modifier, switch));
                }

                bind_box.append(&mods_box);

                let key_box = Box::new(GtkOrientation::Horizontal, 5);
                key_box.append(&Label::new(Some(&t!("advanced_editors.key"))));
                let key_entry = create_entry();
                key_entry.set_text(&unbind_right.key);
                key_box.append(&key_entry);
                bind_box.append(&key_box);

                for (modifier, switch) in &modifier_switches {
                    let modifier = *modifier;
                    let value_entry_clone = value_entry.clone();
                    let is_updating_clone = is_updating.clone();
                    switch.connect_state_notify(move |switch| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        if let Ok(mut unbind_right) =
                            UnbindRight::from_str(&value_entry_clone.text())
                        {
                            if switch.is_active() {
                                unbind_right.mods.insert(modifier);
                            } else {
                                unbind_right.mods.remove(&modifier);
                            }
                            value_entry_clone.set_text(&unbind_right.to_string());
                        }
                        is_updating_clone.set(false);
                    });
                }

                let value_entry_clone = value_entry.clone();
                let is_updating_clone = is_updating.clone();
                key_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    if let Ok(mut unbind_right) = UnbindRight::from_str(&value_entry_clone.text()) {
                        unbind_right.key = entry.text().to_string();
                        value_entry_clone.set_text(&unbind_right.to_string());
                    }
                    is_updating_clone.set(false);
                });

                let is_updating_clone = is_updating.clone();
                let key_entry_clone = key_entry.clone();
                value_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    if let Ok(unbind_right) = UnbindRight::from_str(&entry.text()) {
                        for (modifier, switch) in &modifier_switches {
                            switch.set_active(unbind_right.mods.contains(modifier));
                        }
                        key_entry_clone.set_text(&unbind_right.key);
                    }
                    is_updating_clone.set(false);
                });
            } else {
                let bind_right = parse_bind_right(has_description, &value_entry.text());

                let mods_box = Box::new(GtkOrientation::Horizontal, 5);
                mods_box.append(&Label::new(Some(&t!("advanced_editors.modifiers"))));

                let modifier_names = [
                    Modifier::Shift,
                    Modifier::Ctrl,
                    Modifier::Alt,
                    Modifier::Super,
                    Modifier::Caps,
                    Modifier::Mod2,
                    Modifier::Mod3,
                    Modifier::Mod5,
                ];

                let mut modifier_switches = Vec::new();
                for modifier in &modifier_names {
                    let mod_box = Box::new(GtkOrientation::Horizontal, 5);
                    let switch = create_switch();
                    switch.set_active(bind_right.mods.contains(modifier));
                    mod_box.append(&Label::new(Some(&modifier.to_string())));
                    mod_box.append(&switch);
                    mods_box.append(&mod_box);
                    modifier_switches.push((*modifier, switch));
                }

                bind_box.append(&mods_box);

                let key_box = Box::new(GtkOrientation::Horizontal, 5);
                key_box.append(&Label::new(Some(&t!("advanced_editors.key"))));
                let key_entry = create_entry();
                key_entry.set_text(&bind_right.key);
                key_box.append(&key_entry);
                bind_box.append(&key_box);

                let dispatcher_box = Box::new(GtkOrientation::Vertical, 5);
                dispatcher_box.append(&Label::new(Some(&t!("advanced_editors.dispatcher"))));

                let dispatcher_entry = create_entry();
                dispatcher_entry.set_text(&bind_right.dispatcher.to_string());
                let dispatcher_visualizer = Dispatcher::to_gtk_box(&dispatcher_entry);
                dispatcher_box.append(&dispatcher_visualizer);
                bind_box.append(&dispatcher_box);

                let description_box = Box::new(GtkOrientation::Horizontal, 5);
                description_box.append(&Label::new(Some(&t!("advanced_editors.description"))));
                let description_entry = create_entry();
                if let Some(desc) = &bind_right.description {
                    description_entry.set_text(desc);
                }
                description_box.append(&description_entry);
                if has_description {
                    bind_box.append(&description_box);
                }

                for (modifier, switch) in &modifier_switches {
                    let modifier = *modifier;
                    let value_entry_clone = value_entry.clone();
                    let is_updating_clone = is_updating.clone();
                    switch.connect_state_notify(move |switch| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let mut bind_right =
                            parse_bind_right(has_description, &value_entry_clone.text());

                        if switch.is_active() {
                            bind_right.mods.insert(modifier);
                        } else {
                            bind_right.mods.remove(&modifier);
                        }
                        value_entry_clone.set_text(&bind_right.to_string());

                        is_updating_clone.set(false);
                    });
                }

                let value_entry_clone = value_entry.clone();
                let is_updating_clone = is_updating.clone();
                key_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let mut bind_right =
                        parse_bind_right(has_description, &value_entry_clone.text());

                    bind_right.key = entry.text().to_string();
                    value_entry_clone.set_text(&bind_right.to_string());

                    is_updating_clone.set(false);
                });

                let value_entry_clone = value_entry.clone();
                let is_updating_clone = is_updating.clone();
                dispatcher_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let mut bind_right =
                        parse_bind_right(has_description, &value_entry_clone.text());

                    bind_right.dispatcher = entry.text().parse().unwrap_or_default();
                    value_entry_clone.set_text(&bind_right.to_string());

                    is_updating_clone.set(false);
                });

                if has_description {
                    let value_entry_clone = value_entry.clone();
                    let is_updating_clone = is_updating.clone();
                    description_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let mut bind_right =
                            parse_bind_right(has_description, &value_entry_clone.text());

                        bind_right.description = if !entry.text().is_empty() {
                            Some(entry.text().to_string())
                        } else {
                            None
                        };
                        value_entry_clone.set_text(&bind_right.to_string());

                        is_updating_clone.set(false);
                    });
                }

                let is_updating_clone = is_updating.clone();
                let key_entry_clone = key_entry.clone();
                let dispatcher_entry_clone = dispatcher_entry.clone();
                let description_entry_clone = description_entry.clone();
                value_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let bind_right = parse_bind_right(has_description, &entry.text());
                    for (modifier, switch) in &modifier_switches {
                        switch.set_active(bind_right.mods.contains(modifier));
                    }
                    key_entry_clone.set_text(&bind_right.key);
                    dispatcher_entry_clone.set_text(&bind_right.dispatcher.to_string());
                    if let Some(desc) = &bind_right.description {
                        description_entry_clone.set_text(desc);
                    } else {
                        description_entry_clone.set_text("");
                    }

                    is_updating_clone.set(false);
                });
            }

            fancy_value_entry.append(&bind_box);
        }
        "gesture" => {
            let gesture_box = Gesture::to_gtk_box(value_entry);
            fancy_value_entry.append(&gesture_box);
        }
        "windowrule" => {
            let window_rule_box = WindowRuleWithParameters::to_gtk_box(value_entry);
            fancy_value_entry.append(&window_rule_box);
        }
        "layerrule" => {
            let layer_rule_box = LayerRuleWithParameter::to_gtk_box(value_entry);
            fancy_value_entry.append(&layer_rule_box);
        }
        "exec" => match name {
            "exec-once" | "exec" => {
                let exec_with_rules_box = ExecWithRules::to_gtk_box(value_entry);
                fancy_value_entry.append(&exec_with_rules_box);
            }
            _ => {
                let exec_box = String::to_gtk_box(value_entry);
                fancy_value_entry.append(&exec_box);
            }
        },
        "env" => {
            let env_box = <(String, String)>::to_gtk_box(
                value_entry,
                ',',
                &[
                    FieldLabel::Named(cow_to_static_str(t!("advanced_editors.key"))),
                    FieldLabel::Named(cow_to_static_str(t!("advanced_editors.value"))),
                ],
                None,
            );
            fancy_value_entry.append(&env_box);
        }
        "top_level" => {
            // maybe in future i will implement this
        }
        e => {
            dbg!(e);
            unreachable!()
        }
    }
}
