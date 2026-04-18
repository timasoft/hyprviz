use crate::{
    advanced_editors::create_spin_button, gtk_converters::ToGtkBox, register_togtkbox,
    utils::MARGIN_NORMAL,
};
use gtk::{Align, Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Angle {
    Degrees(u16),
}

impl Default for Angle {
    fn default() -> Self {
        Angle::Degrees(0)
    }
}

impl FromStr for Angle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Some(stripped) = s.strip_suffix("deg") {
            let degrees = stripped.parse::<u16>().unwrap_or_default();
            Ok(Angle::Degrees(degrees))
        } else {
            Err(())
        }
    }
}

impl Display for Angle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Angle::Degrees(degrees) => write!(f, "{}deg", degrees),
        }
    }
}

impl ToGtkBox for Angle {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 8);
        mother_box.set_margin_start(MARGIN_NORMAL / 3);
        mother_box.set_margin_end(MARGIN_NORMAL / 3);
        mother_box.set_margin_top(MARGIN_NORMAL / 3);
        mother_box.set_margin_bottom(MARGIN_NORMAL / 3);

        let label = Label::new(Some(&t!("hyprland.angle.degrees")));
        label.add_css_class("body");
        label.set_halign(Align::Start);
        label.set_hexpand(true);
        label.set_xalign(0.0);
        label.set_valign(Align::Center);
        mother_box.append(&label);

        let degrees_spin_button = create_spin_button(0.0, 359.0, 1.0);
        degrees_spin_button.set_halign(Align::End);
        degrees_spin_button.set_valign(Align::Center);
        mother_box.append(&degrees_spin_button);

        let degrees_spin_button_clone = degrees_spin_button.clone();
        let update_ui = move |angle: Angle| match angle {
            Angle::Degrees(degrees) => {
                degrees_spin_button_clone.set_value(degrees as f64);
            }
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        degrees_spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let angle = Angle::Degrees(spin_button.value() as u16);
            entry_clone.set_text(&angle.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let angle = entry.text().parse().unwrap_or_default();
            update_ui(angle);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(Angle);
