use crate::{
    advanced_editors::create_spin_button,
    gtk_converters::ToGtkBox,
    utils::{MAX_SAFE_STEP_0_01_F64, MIN_SAFE_STEP_0_01_F64},
};
use gtk::{Box as GtkBox, Entry, Orientation as GtkOrientation, prelude::*};
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2(pub f64, pub f64);

impl FromStr for Vec2 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (part1, part2) = s.split_once(' ').ok_or(())?;
        let (part1, part2) = (part1.parse(), part2.trim_start().parse());
        match (part1, part2) {
            (Ok(x), Ok(y)) => Ok(Vec2(x, y)),
            _ => Err(()),
        }
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

impl ToGtkBox for Vec2 {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let part1_spin_button =
            create_spin_button(MIN_SAFE_STEP_0_01_F64, MAX_SAFE_STEP_0_01_F64, 0.01);
        mother_box.append(&part1_spin_button);

        let part2_spin_button =
            create_spin_button(MIN_SAFE_STEP_0_01_F64, MAX_SAFE_STEP_0_01_F64, 0.01);
        mother_box.append(&part2_spin_button);

        let part1_spin_button_clone = part1_spin_button.clone();
        let part2_spin_button_clone = part2_spin_button.clone();
        let update_ui = move |Vec2(part1, part2)| {
            part1_spin_button_clone.set_value(part1);
            part2_spin_button_clone.set_value(part2);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        part1_spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let vec2: Vec2 = entry_clone.text().parse().unwrap_or_default();
            let new_value = Vec2(vec2.0, spin_button.value());
            entry_clone.set_text(&new_value.to_string());

            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        part2_spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let vec2: Vec2 = entry_clone.text().parse().unwrap_or_default();
            let new_value = Vec2(spin_button.value(), vec2.1);
            entry_clone.set_text(&new_value.to_string());

            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let vec2: Vec2 = entry.text().parse().unwrap_or_default();
            update_ui(vec2);

            is_updating_clone.set(false);
        });

        mother_box
    }
}
