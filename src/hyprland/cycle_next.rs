use crate::{advanced_editors::create_switch, gtk_converters::ToGtkBox, register_togtkbox};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CycleNext {
    pub is_prev: bool,
    pub is_tiled: bool,
    pub is_floating: bool,
    pub is_visible: bool,
    pub is_hist: bool,
}

impl FromStr for CycleNext {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        let parts = s.split_whitespace().collect::<Vec<_>>();

        let mut is_next = false;
        let mut is_prev = false;
        let mut is_tiled = false;
        let mut is_floating = false;
        let mut is_visible = false;
        let mut is_hist = false;

        for part in parts {
            match part {
                "next" => {
                    if !(is_next || is_prev) {
                        is_next = true;
                    }
                }
                "prev" => {
                    if !(is_next || is_prev) {
                        is_prev = true;
                    }
                }
                "tiled" => {
                    if !(is_tiled || is_floating) {
                        is_tiled = true;
                    }
                }
                "floating" => {
                    if !(is_tiled || is_floating) {
                        is_floating = true;
                    }
                }
                "visible" => {
                    if !(is_visible) {
                        is_visible = true;
                    }
                }
                "hist" => {
                    if !(is_hist) {
                        is_hist = true;
                    }
                }
                _ => {}
            }
        }

        Ok(CycleNext {
            is_prev,
            is_tiled,
            is_floating,
            is_visible,
            is_hist,
        })
    }
}

impl Display for CycleNext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        if self.is_visible {
            result.push_str("visible ");
        }

        if self.is_prev {
            result.push_str("prev ");
        } else {
            result.push_str("next ");
        }

        if self.is_tiled {
            result.push_str("tiled ");
        } else if self.is_floating {
            result.push_str("floating ");
        }

        if self.is_hist {
            result.push_str("hist ");
        }

        write!(f, "{}", result.trim())
    }
}

impl ToGtkBox for CycleNext {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let is_prev_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_prev_box.append(&Label::new(Some(&t!("hyprland.cycle_next.is_previous"))));
        let is_prev_switch = create_switch();
        is_prev_box.append(&is_prev_switch);
        mother_box.append(&is_prev_box);

        let is_tiled_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_tiled_box.append(&Label::new(Some(&t!("hyprland.cycle_next.is_tiled"))));
        let is_tiled_switch = create_switch();
        is_tiled_box.append(&is_tiled_switch);
        mother_box.append(&is_tiled_box);

        let is_floating_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_floating_box.append(&Label::new(Some(&t!("hyprland.cycle_next.is_floating"))));
        let is_floating_switch = create_switch();
        is_floating_box.append(&is_floating_switch);
        mother_box.append(&is_floating_box);

        let is_visible_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_visible_box.append(&Label::new(Some(&t!("hyprland.cycle_next.is_visible"))));
        let is_visible_switch = create_switch();
        is_visible_box.append(&is_visible_switch);
        mother_box.append(&is_visible_box);

        let is_hist_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_hist_box.append(&Label::new(Some(&t!("hyprland.cycle_next.is_hist"))));
        let is_hist_switch = create_switch();
        is_hist_box.append(&is_hist_switch);
        mother_box.append(&is_hist_box);

        let is_prev_switch_clone = is_prev_switch.clone();
        let is_tiled_switch_clone = is_tiled_switch.clone();
        let is_floating_switch_clone = is_floating_switch.clone();
        let is_visible_switch_clone = is_visible_switch.clone();
        let is_hist_switch_clone = is_hist_switch.clone();
        let update_ui = move |cycle_next: CycleNext| {
            is_prev_switch_clone.set_state(cycle_next.is_prev);
            is_tiled_switch_clone.set_state(cycle_next.is_tiled);
            is_floating_switch_clone.set_state(cycle_next.is_floating);
            is_visible_switch_clone.set_state(cycle_next.is_visible);
            is_hist_switch_clone.set_state(cycle_next.is_hist);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_prev_switch.connect_state_notify(move |is_prev_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_prev = is_prev_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_tiled_switch.connect_state_notify(move |is_tiled_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_tiled = is_tiled_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_floating_switch.connect_state_notify(move |is_floating_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_floating = is_floating_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_visible_switch.connect_state_notify(move |is_visible_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_visible = is_visible_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_hist_switch.connect_state_notify(move |is_hist_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_hist = is_hist_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let cycle_next: CycleNext = entry.text().parse().unwrap_or_default();
            update_ui(cycle_next);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(CycleNext);
