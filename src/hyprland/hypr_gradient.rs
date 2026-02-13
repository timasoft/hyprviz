use super::{Angle, HyprColor};
use crate::{
    advanced_editors::create_entry,
    gtk_converters::{ToGtkBox, ToGtkBoxWithSeparator},
    register_togtkbox,
    utils::join_with_separator,
};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub struct HyprGradient {
    pub colors: Vec<HyprColor>,
    pub angle: Option<Angle>,
}

impl Default for HyprGradient {
    fn default() -> Self {
        HyprGradient {
            colors: vec![HyprColor::default(), HyprColor::default()],
            angle: None,
        }
    }
}

impl FromStr for HyprGradient {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        let mut colors = Vec::new();

        let parts = s.split_whitespace().collect::<Vec<_>>();

        for part in parts {
            if let Ok(angle) = Angle::from_str(part) {
                return Ok(HyprGradient {
                    colors,
                    angle: Some(angle),
                });
            } else if let Ok(color) = HyprColor::from_str(part) {
                colors.push(color);
            }
        }

        if colors.is_empty() {
            colors.push(HyprColor::default());
            colors.push(HyprColor::default());
        } else if colors.len() == 1 {
            colors.push(colors[0]);
        }

        Ok(HyprGradient {
            colors,
            angle: None,
        })
    }
}

impl Display for HyprGradient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.angle {
            Some(angle) => write!(f, "{} {}", join_with_separator(&self.colors, " "), angle),
            None => write!(f, "{}", join_with_separator(&self.colors, " ")),
        }
    }
}

impl ToGtkBox for HyprGradient {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);

        let colors_box = GtkBox::new(GtkOrientation::Vertical, 5);
        colors_box.append(&Label::new(Some(&t!(
            "hyprland.hypr_gradient.gradient_colors"
        ))));

        let colors_entry = create_entry();
        let colors_separator = ' ';
        let colors_ui_box = Vec::<HyprColor>::to_gtk_box(&colors_entry, colors_separator);
        colors_box.append(&colors_ui_box);
        mother_box.append(&colors_box);

        let angle_box = GtkBox::new(GtkOrientation::Vertical, 5);
        angle_box.append(&Label::new(Some(&t!(
            "hyprland.hypr_gradient.gradient_angle"
        ))));

        let angle_entry = create_entry();
        let angle_ui_box = Option::<Angle>::to_gtk_box(&angle_entry);
        angle_box.append(&angle_ui_box);
        mother_box.append(&angle_box);

        let colors_entry_clone = colors_entry.clone();
        let angle_entry_clone = angle_entry.clone();
        let update_ui = move |gradient: HyprGradient| {
            let colors_text = join_with_separator(&gradient.colors, &colors_separator.to_string());
            colors_entry_clone.set_text(&colors_text);

            let angle_text = gradient.angle.map(|a| a.to_string()).unwrap_or_default();
            angle_entry_clone.set_text(&angle_text);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let angle_entry_clone = angle_entry.clone();
        let is_updating_clone = is_updating.clone();
        colors_entry.connect_changed(move |colors_entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let colors: Vec<HyprColor> = colors_entry
                .text()
                .split(colors_separator)
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().unwrap_or_default())
                .collect();

            let angle_str = angle_entry_clone.text().to_string();
            let angle = match angle_str.as_str() {
                "" => None,
                _ => Some(angle_str.parse().unwrap_or_default()),
            };

            let gradient = HyprGradient { colors, angle };

            entry_clone.set_text(&gradient.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let colors_entry_clone = colors_entry.clone();
        let is_updating_clone = is_updating.clone();
        angle_entry.connect_changed(move |angle_entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let colors: Vec<HyprColor> = colors_entry_clone
                .text()
                .split(colors_separator)
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().unwrap_or_default())
                .collect();

            let angle = match angle_entry.text().as_str() {
                "" => None,
                s => Some(s.parse().unwrap_or_default()),
            };

            let gradient = HyprGradient { colors, angle };

            entry_clone.set_text(&gradient.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let gradient = entry.text().parse().unwrap_or_default();
            update_ui(gradient);

            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(HyprGradient);
