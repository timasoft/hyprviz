use super::{PixelOrPercent, SizeBound};
use crate::{
    advanced_editors::{create_dropdown, create_entry},
    gtk_converters::ToGtkBox,
    register_togtkbox,
};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, StringList, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HyprSize {
    pub width: PixelOrPercent,
    pub height: PixelOrPercent,
    pub width_bound: SizeBound,
    pub height_bound: SizeBound,
}

impl FromStr for HyprSize {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let mut result = HyprSize {
            width: PixelOrPercent::Percent(50.0),
            height: PixelOrPercent::Percent(50.0),
            width_bound: SizeBound::Exact,
            height_bound: SizeBound::Exact,
        };

        let parts: Vec<&str> = s.split(' ').collect();

        let width = parts.first().unwrap_or(&"");
        let height = parts.get(1).unwrap_or(&"");

        if let Some(stripped) = width.strip_prefix("<") {
            result.width_bound = SizeBound::Max;
            result.width = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else if let Some(stripped) = width.strip_prefix(">") {
            result.width_bound = SizeBound::Min;
            result.width = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else {
            result.width = PixelOrPercent::from_str(width).unwrap_or_default();
        }

        if let Some(stripped) = height.strip_prefix("<") {
            result.height_bound = SizeBound::Max;
            result.height = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else if let Some(stripped) = height.strip_prefix(">") {
            result.height_bound = SizeBound::Min;
            result.height = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else {
            result.height = PixelOrPercent::from_str(height).unwrap_or_default();
        }

        Ok(result)
    }
}

impl Display for HyprSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        match self.width_bound {
            SizeBound::Exact => {}
            SizeBound::Max => result.push('<'),
            SizeBound::Min => result.push('>'),
        }

        match self.width {
            PixelOrPercent::Percent(p) => {
                result.push_str(&p.to_string());
                result.push('%');
            }
            PixelOrPercent::Pixel(p) => {
                result.push_str(&p.to_string());
            }
        }

        match self.height_bound {
            SizeBound::Exact => {}
            SizeBound::Max => result.push('<'),
            SizeBound::Min => result.push('>'),
        }

        match self.height {
            PixelOrPercent::Percent(p) => {
                result.push_str(&p.to_string());
                result.push('%');
            }
            PixelOrPercent::Pixel(p) => {
                result.push_str(&p.to_string());
            }
        }

        write!(f, "{}", result)
    }
}

impl ToGtkBox for HyprSize {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);

        let width_box_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        width_box_box.append(&Label::new(Some(&t!("hyprland.hypr_size.width"))));
        let width_entry = create_entry();
        let width_box = PixelOrPercent::to_gtk_box(&width_entry);
        width_box_box.append(&width_box);
        mother_box.append(&width_box_box);

        let height_box_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        height_box_box.append(&Label::new(Some(&t!("hyprland.hypr_size.height"))));
        let height_entry = create_entry();
        let height_box = PixelOrPercent::to_gtk_box(&height_entry);
        height_box_box.append(&height_box);
        mother_box.append(&height_box_box);

        let size_bound_string_list = StringList::new(&[
            &t!("hyprland.hypr_size.exact"),
            &t!("hyprland.hypr_size.max"),
            &t!("hyprland.hypr_size.min"),
        ]);

        let width_bound_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        width_bound_box.append(&Label::new(Some(&t!("hyprland.hypr_size.width_bound"))));
        let width_bound_dropdown = create_dropdown(&size_bound_string_list);
        width_bound_box.append(&width_bound_dropdown);
        mother_box.append(&width_bound_box);

        let height_bound_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        height_bound_box.append(&Label::new(Some(&t!("hyprland.hypr_size.height_bound"))));
        let height_bound_dropdown = create_dropdown(&size_bound_string_list);
        height_bound_box.append(&height_bound_dropdown);
        mother_box.append(&height_bound_box);

        let width_entry_clone = width_entry.clone();
        let height_entry_clone = height_entry.clone();
        let width_bound_dropdown_clone = width_bound_dropdown.clone();
        let height_bound_dropdown_clone = height_bound_dropdown.clone();
        let update_ui = move |hypr_size: HyprSize| {
            width_entry_clone.set_text(&hypr_size.width.to_string());
            height_entry_clone.set_text(&hypr_size.height.to_string());
            width_bound_dropdown_clone.set_selected(match hypr_size.width_bound {
                SizeBound::Exact => 0,
                SizeBound::Max => 1,
                SizeBound::Min => 2,
            });
            height_bound_dropdown_clone.set_selected(match hypr_size.height_bound {
                SizeBound::Exact => 0,
                SizeBound::Max => 1,
                SizeBound::Min => 2,
            });
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        width_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_size: HyprSize = entry_clone.text().parse().unwrap_or_default();
            hypr_size.width = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&hypr_size.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        height_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_size: HyprSize = entry_clone.text().parse().unwrap_or_default();
            hypr_size.height = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&hypr_size.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        width_bound_dropdown.connect_selected_notify(move |dropdown| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_size: HyprSize = entry_clone.text().parse().unwrap_or_default();
            hypr_size.width_bound = match dropdown.selected() {
                0 => SizeBound::Exact,
                1 => SizeBound::Max,
                2 => SizeBound::Min,
                _ => unreachable!(),
            };
            entry_clone.set_text(&hypr_size.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        height_bound_dropdown.connect_selected_notify(move |dropdown| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_size: HyprSize = entry_clone.text().parse().unwrap_or_default();
            hypr_size.height_bound = match dropdown.selected() {
                0 => SizeBound::Exact,
                1 => SizeBound::Max,
                2 => SizeBound::Min,
                _ => unreachable!(),
            };
            entry_clone.set_text(&hypr_size.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let hypr_size = entry.text().parse().unwrap_or_default();
            update_ui(hypr_size);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(HyprSize);
