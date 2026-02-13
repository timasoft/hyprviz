use super::PixelOrPercent;
use crate::{
    advanced_editors::{create_entry, create_spin_button, create_switch},
    gtk_converters::ToGtkBox,
    register_togtkbox,
};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HyprCoord {
    pub x: PixelOrPercent,
    pub y: PixelOrPercent,
    pub x_sub: u32,
    pub y_sub: u32,
    pub under_cursor: bool,
    pub on_screen: bool,
}

impl FromStr for HyprCoord {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let mut result = HyprCoord::default();

        let parts: Vec<&str> = s.split(' ').collect();

        if parts.is_empty() {
            return Err(());
        }

        let mut is_x = true;

        for part in parts {
            let part = part.trim();
            if part == "onscreen" {
                result.on_screen = true;
            } else if part == "undercursor" {
                result.under_cursor = true;
            } else {
                // parse "100", "100%", "100%-100"
                let (num_or_percent, sub) = part.split_once('-').unwrap_or((part, ""));
                let num_or_percent: PixelOrPercent =
                    PixelOrPercent::from_str(num_or_percent).unwrap_or_default();
                let sub: u32 = sub.parse().unwrap_or_default();
                if is_x {
                    result.x = num_or_percent;
                    result.x_sub = sub;
                    is_x = false;
                } else {
                    result.y = num_or_percent;
                    result.y_sub = sub;
                    break;
                }
            }
        }

        Ok(result)
    }
}

impl Display for HyprCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        if self.on_screen {
            result.push_str("onscreen ");
        }

        if self.under_cursor {
            result.push_str("undercursor ");
        }

        match self.x {
            PixelOrPercent::Percent(p) => {
                result.push_str(&p.to_string());
                result.push('%');
                if self.x_sub > 0 {
                    result.push('-');
                    result.push_str(&self.x_sub.to_string());
                }
            }
            PixelOrPercent::Pixel(p) => {
                result.push_str(&p.to_string());
            }
        }

        match self.y {
            PixelOrPercent::Percent(p) => {
                result.push_str(&p.to_string());
                result.push('%');
                if self.y_sub > 0 {
                    result.push('-');
                    result.push_str(&self.y_sub.to_string());
                }
            }
            PixelOrPercent::Pixel(p) => {
                result.push_str(&p.to_string());
            }
        }

        write!(f, "{}", result)
    }
}

impl ToGtkBox for HyprCoord {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);

        let x_box_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        x_box_box.append(&Label::new(Some("X")));
        let x_entry = create_entry();
        let x_box = PixelOrPercent::to_gtk_box(&x_entry);
        x_box_box.append(&x_box);
        mother_box.append(&x_box_box);

        let y_box_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        y_box_box.append(&Label::new(Some("Y")));
        let y_entry = create_entry();
        let y_box = PixelOrPercent::to_gtk_box(&y_entry);
        y_box_box.append(&y_box);
        mother_box.append(&y_box_box);

        let x_sub_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        x_sub_box.append(&Label::new(Some(&t!(
            "hyprland.hypr_coord.subtrahend_of_x"
        ))));
        let x_sub_spin_button = create_spin_button(0.0, i32::MAX as f64, 1.0);
        x_sub_box.append(&x_sub_spin_button);
        mother_box.append(&x_sub_box);

        let y_sub_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        y_sub_box.append(&Label::new(Some(&t!(
            "hyprland.hypr_coord.subtrahend_of_y"
        ))));
        let y_sub_spin_button = create_spin_button(0.0, i32::MAX as f64, 1.0);
        y_sub_box.append(&y_sub_spin_button);
        mother_box.append(&y_sub_box);

        let under_cursor_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        under_cursor_box.append(&Label::new(Some(&t!("hyprland.hypr_coord.under_cursor"))));
        let under_cursor_switch = create_switch();
        under_cursor_box.append(&under_cursor_switch);
        mother_box.append(&under_cursor_box);

        let on_screen_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        on_screen_box.append(&Label::new(Some(&t!("hyprland.hypr_coord.on_screen"))));
        let on_screen_switch = create_switch();
        on_screen_box.append(&on_screen_switch);
        mother_box.append(&on_screen_box);

        let x_entry_clone = x_entry.clone();
        let y_entry_clone = y_entry.clone();
        let x_sub_spin_button_clone = x_sub_spin_button.clone();
        let y_sub_spin_button_clone = y_sub_spin_button.clone();
        let under_cursor_switch_clone = under_cursor_switch.clone();
        let on_screen_switch_clone = on_screen_switch.clone();
        let update_ui = move |hypr_coord: HyprCoord| {
            x_entry_clone.set_text(&hypr_coord.x.to_string());
            y_entry_clone.set_text(&hypr_coord.y.to_string());
            x_sub_spin_button_clone.set_value(hypr_coord.x_sub as f64);
            y_sub_spin_button_clone.set_value(hypr_coord.y_sub as f64);
            under_cursor_switch_clone.set_active(hypr_coord.under_cursor);
            on_screen_switch_clone.set_active(hypr_coord.on_screen);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        x_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.x = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        y_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.y = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        x_sub_spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.x_sub = spin_button.value() as u32;
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        y_sub_spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.y_sub = spin_button.value() as u32;
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        under_cursor_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.under_cursor = switch.state();
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        on_screen_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.on_screen = switch.state();
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let hypr_coord = entry.text().parse().unwrap_or_default();
            update_ui(hypr_coord);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(HyprCoord);
