use crate::{advanced_editors::create_spin_button, gtk_converters::ToGtkBox, register_togtkbox};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

impl Default for Range {
    fn default() -> Self {
        Range { start: 1, end: 1 }
    }
}

impl FromStr for Range {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = s.split_once('-').unwrap_or((s, "1"));
        Ok(Range {
            start: start.parse().unwrap_or(1),
            end: end.parse().unwrap_or(1),
        })
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl ToGtkBox for Range {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let start_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        start_box_box.append(&Label::new(Some(&t!("gtk_converters.start"))));
        let start_spin_button = create_spin_button(1.0, i32::MAX as f64, 1.0);
        start_box_box.append(&start_spin_button);
        mother_box.append(&start_box_box);

        let end_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        end_box_box.append(&Label::new(Some(&t!("gtk_converters.end"))));
        let end_spin_button = create_spin_button(1.0, i32::MAX as f64, 1.0);
        end_box_box.append(&end_spin_button);
        mother_box.append(&end_box_box);

        let start_spin_button_clone = start_spin_button.clone();
        let end_spin_button_clone = end_spin_button.clone();
        let update_ui = move |range: Range| {
            start_spin_button_clone.set_value(range.start as f64);
            end_spin_button_clone.set_value(range.end as f64);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        start_spin_button.connect_value_changed(move |spin| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut range: Range = entry_clone.text().parse().unwrap_or_default();
            range.start = spin.value() as u32;
            entry_clone.set_text(&range.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        end_spin_button.connect_value_changed(move |spin| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut range: Range = entry_clone.text().parse().unwrap_or_default();
            range.end = spin.value() as u32;
            entry_clone.set_text(&range.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let range: Range = entry.text().parse().unwrap_or_default();
            update_ui(range);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(Range);
