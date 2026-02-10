use super::{Angle, HyprColor};
use crate::{
    advanced_editors::{create_dropdown, create_entry},
    gtk_converters::{ToGtkBox, ToGtkBoxWithSeparator},
    register_togtkbox, register_togtkbox_with_separator,
    utils::join_with_separator,
};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, StringList, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub enum BorderColor {
    Color(HyprColor),
    Gradient(Vec<HyprColor>, Angle),
    DoubleColor(HyprColor, HyprColor),
    DoubleGradient(Vec<HyprColor>, Angle, Vec<HyprColor>, Option<Angle>),
}

impl BorderColor {
    pub const SEPARATOR: char = ' ';
}

impl Default for BorderColor {
    fn default() -> Self {
        BorderColor::Color(HyprColor::default())
    }
}

impl FromStr for BorderColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let parts: Vec<&str> = s.split(Self::SEPARATOR).collect();
        if parts.len() == 1 {
            // Color
            let color = HyprColor::from_str(parts[0]).unwrap_or_default();
            Ok(BorderColor::Color(color))
        } else if parts.len() == 2 {
            // Double Color and Simple Gradient
            let color1 = HyprColor::from_str(parts[0]).unwrap_or_default();
            match parts[1].parse::<Angle>() {
                Ok(angle) => Ok(BorderColor::Gradient(vec![color1, color1], angle)),
                Err(_) => Ok(BorderColor::DoubleColor(
                    color1,
                    HyprColor::from_str(parts[1]).unwrap_or_default(),
                )),
            }
        } else {
            // Gradient or Double Gradient
            let mut first_gradient: Vec<HyprColor> = Vec::new();
            let mut first_angle: Angle = Angle::default();
            let mut first_angle_idx = 0;

            for (i, part) in parts.iter().enumerate() {
                if let Ok(angle) = Angle::from_str(part) {
                    first_angle = angle;
                    first_angle_idx = i;
                    break;
                } else if let Ok(color) = HyprColor::from_str(part) {
                    first_gradient.push(color);
                }
            }

            if first_gradient.len() == 1 {
                first_gradient.push(first_gradient[0]);
            }

            if first_angle_idx == parts.len() - 1 {
                // Gradient
                Ok(BorderColor::Gradient(first_gradient, first_angle))
            } else {
                // Double Gradient
                let mut second_gradient: Vec<HyprColor> = Vec::new();
                let mut second_angle: Option<Angle> = None;

                for part in parts[first_angle_idx + 1..].iter() {
                    if let Ok(angle) = Angle::from_str(part) {
                        second_angle = Some(angle);
                        break;
                    } else if let Ok(color) = HyprColor::from_str(part) {
                        second_gradient.push(color);
                    }
                }

                if second_gradient.len() == 1 {
                    second_gradient.push(second_gradient[0]);
                }

                Ok(BorderColor::DoubleGradient(
                    first_gradient,
                    first_angle,
                    second_gradient,
                    second_angle,
                ))
            }
        }
    }
}

impl Display for BorderColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BorderColor::Color(color) => write!(f, "{}", color),
            BorderColor::Gradient(colors, angle) => {
                let colors: String = join_with_separator(colors, &Self::SEPARATOR.to_string());
                write!(f, "{}{}{}", colors, Self::SEPARATOR, angle)
            }
            BorderColor::DoubleColor(color1, color2) => {
                write!(f, "{}{}{}", color1, Self::SEPARATOR, color2)
            }
            BorderColor::DoubleGradient(colors1, angle1, colors2, angle2) => {
                let colors1: String = join_with_separator(colors1, &Self::SEPARATOR.to_string());
                let colors2: String = join_with_separator(colors2, &Self::SEPARATOR.to_string());
                match angle2 {
                    None => write!(
                        f,
                        "{}{}{}{}{}",
                        colors1,
                        Self::SEPARATOR,
                        angle1,
                        Self::SEPARATOR,
                        colors2
                    ),
                    Some(angle2) => write!(
                        f,
                        "{}{}{}{}{}{}{}",
                        colors1,
                        Self::SEPARATOR,
                        angle1,
                        Self::SEPARATOR,
                        colors2,
                        Self::SEPARATOR,
                        angle2
                    ),
                }
            }
        }
    }
}

impl ToGtkBox for BorderColor {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let border_color_string_list = StringList::new(&[
            &t!("gtk_converters.active_border_color"),
            &t!("gtk_converters.active_border_gradient"),
            &t!("gtk_converters.active_and_inactive_border_color"),
            &t!("gtk_converters.active_and_inactive_border_gradient"),
        ]);
        let border_color_dropdown = create_dropdown(&border_color_string_list);
        border_color_dropdown.set_selected(0);
        mother_box.append(&border_color_dropdown);

        let hypr_color_entry = create_entry();
        let hypr_color_box = HyprColor::to_gtk_box(&hypr_color_entry);
        hypr_color_box.set_visible(false);
        mother_box.append(&hypr_color_box);

        let second_hypr_color_entry = create_entry();
        let second_hypr_color_box = HyprColor::to_gtk_box(&second_hypr_color_entry);
        second_hypr_color_box.set_visible(false);
        mother_box.append(&second_hypr_color_box);

        let gradient_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let vec_hypr_color_entry = create_entry();
        let vec_hypr_color_box =
            Vec::<HyprColor>::to_gtk_box(&vec_hypr_color_entry, Self::SEPARATOR);
        gradient_box.append(&vec_hypr_color_box);
        let angle_entry = create_entry();
        let angle_box = Angle::to_gtk_box(&angle_entry);
        angle_box.prepend(&Label::new(Some(&t!("gtk_converters.angle"))));
        gradient_box.append(&angle_box);
        mother_box.append(&gradient_box);

        let second_gradient_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let second_vec_hypr_color_entry = create_entry();
        let second_vec_hypr_color_box =
            Vec::<HyprColor>::to_gtk_box(&second_vec_hypr_color_entry, Self::SEPARATOR);
        second_gradient_box.append(&second_vec_hypr_color_box);
        let opt_angle_entry = create_entry();
        let opt_angle_box = Option::<Angle>::to_gtk_box(&opt_angle_entry);
        opt_angle_box.prepend(&Label::new(Some(&t!("gtk_converters.angle"))));
        second_gradient_box.append(&opt_angle_box);
        mother_box.append(&second_gradient_box);

        let border_color_dropdown_clone = border_color_dropdown.clone();
        let hypr_color_entry_clone = hypr_color_entry.clone();
        let hypr_color_box_clone = hypr_color_box.clone();
        let second_hypr_color_entry_clone = second_hypr_color_entry.clone();
        let second_hypr_color_box_clone = second_hypr_color_box.clone();
        let vec_hypr_color_entry_clone = vec_hypr_color_entry.clone();
        let vec_hypr_color_box_clone = vec_hypr_color_box.clone();
        let angle_entry_clone = angle_entry.clone();
        let angle_box_clone = angle_box.clone();
        let second_vec_hypr_color_entry_clone = second_vec_hypr_color_entry.clone();
        let second_vec_hypr_color_box_clone = second_vec_hypr_color_box.clone();
        let opt_angle_entry_clone = opt_angle_entry.clone();
        let opt_angle_box_clone = opt_angle_box.clone();
        let update_ui = move |border_color: BorderColor| match border_color {
            BorderColor::Color(color) => {
                border_color_dropdown_clone.set_selected(0);
                hypr_color_entry_clone.set_text(&color.to_string());

                hypr_color_box_clone.set_visible(true);
                second_hypr_color_box_clone.set_visible(false);
                vec_hypr_color_box_clone.set_visible(false);
                second_vec_hypr_color_box_clone.set_visible(false);
                angle_box_clone.set_visible(false);
                opt_angle_box_clone.set_visible(false);
            }
            BorderColor::Gradient(colors, angle) => {
                border_color_dropdown_clone.set_selected(1);
                vec_hypr_color_entry_clone
                    .set_text(&join_with_separator(colors, &Self::SEPARATOR.to_string()));
                angle_entry_clone.set_text(&angle.to_string());

                hypr_color_box_clone.set_visible(false);
                second_hypr_color_box_clone.set_visible(false);
                vec_hypr_color_box_clone.set_visible(true);
                second_vec_hypr_color_box_clone.set_visible(false);
                angle_box_clone.set_visible(true);
                opt_angle_box_clone.set_visible(false);
            }
            BorderColor::DoubleColor(color1, color2) => {
                border_color_dropdown_clone.set_selected(2);
                hypr_color_entry_clone.set_text(&color1.to_string());
                second_hypr_color_entry_clone.set_text(&color2.to_string());

                hypr_color_box_clone.set_visible(true);
                second_hypr_color_box_clone.set_visible(true);
                vec_hypr_color_box_clone.set_visible(false);
                second_vec_hypr_color_box_clone.set_visible(false);
                angle_box_clone.set_visible(false);
                opt_angle_box_clone.set_visible(false);
            }
            BorderColor::DoubleGradient(colors1, angle1, colors2, angle2) => {
                border_color_dropdown_clone.set_selected(3);
                vec_hypr_color_entry_clone
                    .set_text(&join_with_separator(colors1, &Self::SEPARATOR.to_string()));
                angle_entry_clone.set_text(&angle1.to_string());
                second_vec_hypr_color_entry_clone
                    .set_text(&join_with_separator(colors2, &Self::SEPARATOR.to_string()));
                opt_angle_entry_clone.set_text(&angle2.unwrap_or_default().to_string());

                hypr_color_box_clone.set_visible(false);
                second_hypr_color_box_clone.set_visible(false);
                vec_hypr_color_box_clone.set_visible(true);
                second_vec_hypr_color_box_clone.set_visible(true);
                angle_box_clone.set_visible(true);
                opt_angle_box_clone.set_visible(true);
            }
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let update_ui_clone = update_ui.clone();
        let entry_clone = entry.clone();
        let hypr_color_entry_clone = hypr_color_entry.clone();
        let second_hypr_color_entry_clone = second_hypr_color_entry.clone();
        let vec_hypr_color_entry_clone = vec_hypr_color_entry.clone();
        let second_vec_hypr_color_entry_clone = second_vec_hypr_color_entry.clone();
        let angle_entry_clone = angle_entry.clone();
        let opt_angle_entry_clone = opt_angle_entry.clone();
        let is_updating_clone = is_updating.clone();
        border_color_dropdown.connect_selected_notify(move |dropdown| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let border_color = match dropdown.selected() {
                0 => BorderColor::Color(hypr_color_entry_clone.text().parse().unwrap_or_default()),
                1 => BorderColor::Gradient(
                    vec_hypr_color_entry_clone
                        .text()
                        .split(Self::SEPARATOR)
                        .map(|s| s.parse().unwrap_or_default())
                        .collect(),
                    angle_entry_clone.text().parse().unwrap_or_default(),
                ),
                2 => BorderColor::DoubleColor(
                    hypr_color_entry_clone.text().parse().unwrap_or_default(),
                    second_hypr_color_entry_clone
                        .text()
                        .parse()
                        .unwrap_or_default(),
                ),
                3 => BorderColor::DoubleGradient(
                    vec_hypr_color_entry_clone
                        .text()
                        .split(Self::SEPARATOR)
                        .map(|s| s.parse().unwrap_or_default())
                        .collect(),
                    angle_entry_clone.text().parse().unwrap_or_default(),
                    second_vec_hypr_color_entry_clone
                        .text()
                        .split(Self::SEPARATOR)
                        .map(|s| s.parse().unwrap_or_default())
                        .collect(),
                    match opt_angle_entry_clone.text().as_str() {
                        "" => None,
                        s => Some(s.parse().unwrap_or_default()),
                    },
                ),
                _ => unreachable!(),
            };

            entry_clone.set_text(&border_color.to_string());
            update_ui_clone(border_color);
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        hypr_color_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let color = entry.text().parse().unwrap_or_default();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            match border_color {
                BorderColor::Color(_) => {
                    entry_clone.set_text(&BorderColor::Color(color).to_string());
                }
                BorderColor::DoubleColor(_, color2) => {
                    entry_clone.set_text(&BorderColor::DoubleColor(color, color2).to_string());
                }
                _ => {}
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        second_hypr_color_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let color = entry.text().parse().unwrap_or_default();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            if let BorderColor::DoubleColor(color1, _) = border_color {
                entry_clone.set_text(&BorderColor::DoubleColor(color1, color).to_string());
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        vec_hypr_color_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let colors: Vec<HyprColor> = entry
                .text()
                .split(Self::SEPARATOR)
                .map(|s| s.parse().unwrap_or_default())
                .collect();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            match border_color {
                BorderColor::Gradient(_, angle) => {
                    entry_clone.set_text(&BorderColor::Gradient(colors, angle).to_string());
                }
                BorderColor::DoubleGradient(_, angle1, colors2, angle2) => {
                    entry_clone.set_text(
                        &BorderColor::DoubleGradient(colors, angle1, colors2, angle2).to_string(),
                    );
                }
                _ => {}
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        second_vec_hypr_color_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let colors: Vec<HyprColor> = entry
                .text()
                .split(Self::SEPARATOR)
                .map(|s| s.parse().unwrap_or_default())
                .collect();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            if let BorderColor::DoubleGradient(colors1, angle1, _, angle2) = border_color {
                entry_clone.set_text(
                    &BorderColor::DoubleGradient(colors1, angle1, colors, angle2).to_string(),
                );
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        angle_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let angle = entry.text().parse().unwrap_or_default();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            match border_color {
                BorderColor::Gradient(colors, _) => {
                    entry_clone.set_text(&BorderColor::Gradient(colors, angle).to_string());
                }
                BorderColor::DoubleGradient(colors1, _, colors2, angle2) => {
                    entry_clone.set_text(
                        &BorderColor::DoubleGradient(colors1, angle, colors2, angle2).to_string(),
                    );
                }
                _ => {}
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        opt_angle_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let angle = match entry.text().as_str() {
                "" => None,
                s => Some(s.parse().unwrap_or_default()),
            };
            let border_color = entry_clone.text().parse().unwrap_or_default();

            if let BorderColor::DoubleGradient(colors1, angle1, colors2, _) = border_color {
                entry_clone.set_text(
                    &BorderColor::DoubleGradient(colors1, angle1, colors2, angle).to_string(),
                );
            };
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            update_ui(entry.text().parse().unwrap_or_default());
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(BorderColor);
register_togtkbox_with_separator!(Vec<HyprColor>);
