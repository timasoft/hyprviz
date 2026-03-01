use super::{HyprPermission, PermissionMode};
use crate::{advanced_editors::create_entry, gtk_converters::ToGtkBox, register_togtkbox};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Permission {
    regex: String,
    permission: HyprPermission,
    mode: PermissionMode,
}

impl FromStr for Permission {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.is_empty() {
            return Err(());
        }

        let regex = parts.first().unwrap_or(&"").to_string();
        let permission = parts.get(1).unwrap_or(&"").parse().unwrap_or_default();
        let mode = parts
            .get(2)
            .unwrap_or(&"")
            .parse()
            .unwrap_or(match permission {
                HyprPermission::ScreenCopy => PermissionMode::Ask,
                HyprPermission::Plugin => PermissionMode::Ask,
                HyprPermission::Keyboard => PermissionMode::Allow,
            });

        Ok(Permission {
            regex,
            permission,
            mode,
        })
    }
}

impl Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.regex, self.permission, self.mode)
    }
}

impl ToGtkBox for Permission {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let regex_box = GtkBox::new(GtkOrientation::Vertical, 5);
        regex_box.append(&Label::new(Some(&t!("hyprland.permission.regex"))));
        let regex_entry = create_entry();
        regex_box.append(&regex_entry);
        mother_box.append(&regex_box);

        let permission_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        permission_box_box.append(&Label::new(Some(&t!("hyprland.permission.permission"))));
        let permission_entry = create_entry();
        let permission_box = HyprPermission::to_gtk_box(&permission_entry);
        permission_box_box.append(&permission_box);
        mother_box.append(&permission_box_box);

        let mode_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        mode_box_box.append(&Label::new(Some(&t!("hyprland.permission.mode"))));
        let mode_entry = create_entry();
        let mode_box = PermissionMode::to_gtk_box(&mode_entry);
        mode_box_box.append(&mode_box);
        mother_box.append(&mode_box_box);

        let regex_entry_clone = regex_entry.clone();
        let permission_entry_clone = permission_entry.clone();
        let mode_entry_clone = mode_entry.clone();
        let update_ui = move |permission: Permission| {
            regex_entry_clone.set_text(&permission.regex);
            permission_entry_clone.set_text(&permission.permission.to_string());
            mode_entry_clone.set_text(&permission.mode.to_string());
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        regex_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let mut permission: Permission = entry_clone.text().parse().unwrap_or_default();
            permission.regex = entry.text().to_string();
            entry_clone.set_text(&permission.to_string());

            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        permission_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let mut permission: Permission = entry_clone.text().parse().unwrap_or_default();
            permission.permission = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&permission.to_string());

            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        mode_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let mut permission: Permission = entry_clone.text().parse().unwrap_or_default();
            permission.mode = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&permission.to_string());

            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let permission: Permission = entry.text().parse().unwrap_or_default();
            update_ui(permission);

            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(Permission);
