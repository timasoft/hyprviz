use crate::{advanced_editors::create_switch, gtk_converters::ToGtkBox, register_togtkbox};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorkspaceSelectorWindowCountFlags {
    pub tiled: bool,
    pub floating: bool,
    pub groups: bool,
    pub visible: bool,
    pub pinned: bool,
}

impl FromStr for WorkspaceSelectorWindowCountFlags {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = WorkspaceSelectorWindowCountFlags {
            tiled: false,
            floating: false,
            groups: false,
            visible: false,
            pinned: false,
        };

        for c in s.trim().chars() {
            match c {
                't' => flags.tiled = true,
                'f' => flags.floating = true,
                'g' => flags.groups = true,
                'v' => flags.visible = true,
                'p' => flags.pinned = true,
                _ => return Err(format!("Invalid flag: {}", c)),
            }
        }

        Ok(flags)
    }
}

impl Display for WorkspaceSelectorWindowCountFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = String::new();

        if self.tiled {
            flags.push('t');
        }
        if self.floating {
            flags.push('f');
        }
        if self.groups {
            flags.push('g');
        }
        if self.visible {
            flags.push('v');
        }
        if self.pinned {
            flags.push('p');
        }
        write!(f, "{}", flags)
    }
}

impl ToGtkBox for WorkspaceSelectorWindowCountFlags {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);

        let tiled_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        tiled_box.append(&Label::new(Some(&t!(
            "hyprland.workspace_selector_window_count_flags.is_tiled"
        ))));
        let tiled_switch = create_switch();
        tiled_box.append(&tiled_switch);
        mother_box.append(&tiled_box);

        let floating_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        floating_box.append(&Label::new(Some(&t!(
            "hyprland.workspace_selector_window_count_flags.is_floating"
        ))));
        let floating_switch = create_switch();
        floating_box.append(&floating_switch);
        mother_box.append(&floating_box);

        let groups_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        groups_box.append(&Label::new(Some(&t!(
            "hyprland.workspace_selector_window_count_flags.is_in_group"
        ))));
        let groups_switch = create_switch();
        groups_box.append(&groups_switch);
        mother_box.append(&groups_box);

        let visible_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        visible_box.append(&Label::new(Some(&t!(
            "hyprland.workspace_selector_window_count_flags.is_visible"
        ))));
        let visible_switch = create_switch();
        visible_box.append(&visible_switch);
        mother_box.append(&visible_box);

        let pinned_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        pinned_box.append(&Label::new(Some(&t!(
            "hyprland.workspace_selector_window_count_flags.is_pinned"
        ))));
        let pinned_switch = create_switch();
        pinned_box.append(&pinned_switch);
        mother_box.append(&pinned_box);

        let tiled_switch_clone = tiled_switch.clone();
        let floating_switch_clone = floating_switch.clone();
        let groups_switch_clone = groups_switch.clone();
        let visible_switch_clone = visible_switch.clone();
        let pinned_switch_clone = pinned_switch.clone();
        let update_ui = move |flags: WorkspaceSelectorWindowCountFlags| {
            tiled_switch_clone.set_active(flags.tiled);
            floating_switch_clone.set_active(flags.floating);
            groups_switch_clone.set_active(flags.groups);
            visible_switch_clone.set_active(flags.visible);
            pinned_switch_clone.set_active(flags.pinned);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        tiled_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut flags: WorkspaceSelectorWindowCountFlags =
                entry_clone.text().parse().unwrap_or_default();
            flags.tiled = switch.state();
            entry_clone.set_text(&flags.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        floating_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut flags: WorkspaceSelectorWindowCountFlags =
                entry_clone.text().parse().unwrap_or_default();
            flags.floating = switch.state();
            entry_clone.set_text(&flags.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        groups_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut flags: WorkspaceSelectorWindowCountFlags =
                entry_clone.text().parse().unwrap_or_default();
            flags.groups = switch.state();
            entry_clone.set_text(&flags.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        visible_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut flags: WorkspaceSelectorWindowCountFlags =
                entry_clone.text().parse().unwrap_or_default();
            flags.visible = switch.state();
            entry_clone.set_text(&flags.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        pinned_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut flags: WorkspaceSelectorWindowCountFlags =
                entry_clone.text().parse().unwrap_or_default();
            flags.pinned = switch.state();
            entry_clone.set_text(&flags.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let flags: WorkspaceSelectorWindowCountFlags = entry.text().parse().unwrap_or_default();
            update_ui(flags);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(WorkspaceSelectorWindowCountFlags);
