use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, cow_to_static_str},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(HyprColorDiscriminant))]
pub enum HyprColor {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
}

impl Default for HyprColor {
    fn default() -> Self {
        HyprColor::Rgb(0, 0, 0)
    }
}

impl HasDiscriminant for HyprColor {
    type Discriminant = HyprColorDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Rgb => Self::Rgb(0, 0, 0),
            Self::Discriminant::Rgba => Self::Rgba(0, 0, 0, 255),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Rgb => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() >= 3 {
                    let r = parts[0].parse().unwrap_or(0);
                    let g = parts[1].parse().unwrap_or(0);
                    let b = parts[2].parse().unwrap_or(0);
                    Self::Rgb(r, g, b)
                } else {
                    Self::Rgb(0, 0, 0)
                }
            }
            Self::Discriminant::Rgba => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() >= 4 {
                    let r = parts[0].parse().unwrap_or(0);
                    let g = parts[1].parse().unwrap_or(0);
                    let b = parts[2].parse().unwrap_or(0);
                    let a = parts[3].parse().unwrap_or(255);
                    Self::Rgba(r, g, b, a)
                } else if parts.len() == 3 {
                    let r = parts[0].parse().unwrap_or(0);
                    let g = parts[1].parse().unwrap_or(0);
                    let b = parts[2].parse().unwrap_or(0);
                    Self::Rgba(r, g, b, 255)
                } else {
                    Self::Rgba(0, 0, 0, 255)
                }
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::Rgb(r, g, b) => Some(format!("{},{},{}", r, g, b)),
            Self::Rgba(r, g, b, a) => Some(format!("{},{},{},{}", r, g, b, a)),
        }
    }
}

impl FromStr for HyprColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() || !s.is_ascii() {
            return Err(());
        }

        if s.starts_with("rgb(") && s.ends_with(')') {
            // rgb(255,0,0) and rgb(ff0000)
            let rgb_vec: Vec<&str> = s[4..s.len() - 1].split(',').collect();
            if rgb_vec.len() == 1 && rgb_vec[0].len() == 6 {
                let r = u8::from_str_radix(&rgb_vec[0][0..2], 16).unwrap_or_default();
                let g = u8::from_str_radix(&rgb_vec[0][2..4], 16).unwrap_or_default();
                let b = u8::from_str_radix(&rgb_vec[0][4..6], 16).unwrap_or_default();
                Ok(HyprColor::Rgb(r, g, b))
            } else if rgb_vec.len() == 3 {
                let r = u8::from_str(rgb_vec[0]).unwrap_or_default();
                let g = u8::from_str(rgb_vec[1]).unwrap_or_default();
                let b = u8::from_str(rgb_vec[2]).unwrap_or_default();
                Ok(HyprColor::Rgb(r, g, b))
            } else {
                Err(())
            }
        } else if s.starts_with("rgba(") && s.ends_with(')') {
            // rgba(255,0,0,1) and rgba(ff0000ff)
            let rgba_vec: Vec<&str> = s[5..s.len() - 1].split(',').collect();
            if rgba_vec.len() == 1 && rgba_vec[0].len() == 8 {
                let r = u8::from_str_radix(&rgba_vec[0][0..2], 16).unwrap_or_default();
                let g = u8::from_str_radix(&rgba_vec[0][2..4], 16).unwrap_or_default();
                let b = u8::from_str_radix(&rgba_vec[0][4..6], 16).unwrap_or_default();
                let a = u8::from_str_radix(&rgba_vec[0][6..8], 16).unwrap_or_default();
                Ok(HyprColor::Rgba(r, g, b, a))
            } else if rgba_vec.len() == 4 {
                let r = u8::from_str(rgba_vec[0]).unwrap_or_default();
                let g = u8::from_str(rgba_vec[1]).unwrap_or_default();
                let b = u8::from_str(rgba_vec[2]).unwrap_or_default();
                let a = (f64::from_str(rgba_vec[3]).unwrap_or_default() * 255.0) as u8;
                Ok(HyprColor::Rgba(r, g, b, a))
            } else {
                Err(())
            }
        } else if s.starts_with("0x") && s.len() == 10 {
            // 0xffff0000
            let a = u8::from_str_radix(&s[2..4], 16).unwrap_or_default();
            let r = u8::from_str_radix(&s[4..6], 16).unwrap_or_default();
            let g = u8::from_str_radix(&s[6..8], 16).unwrap_or_default();
            let b = u8::from_str_radix(&s[8..10], 16).unwrap_or_default();
            Ok(HyprColor::Rgba(r, g, b, a))
        } else {
            Err(())
        }
    }
}

impl Display for HyprColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprColor::Rgb(r, g, b) => write!(f, "rgb({},{},{})", r, g, b),
            HyprColor::Rgba(r, g, b, a) => {
                write!(f, "rgba({},{},{},{})", r, g, b, *a as f64 / 255.0)
            }
        }
    }
}

impl EnumConfigForGtk for HyprColor {
    fn dropdown_items() -> StringList {
        StringList::new(&["RGB", "RGBA"])
    }

    const SEPARATOR: Option<char> = Some(',');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            HyprColor::Rgb(_r, _g, _b) => Some(<(u8, u8, u8)>::to_gtk_box),
            HyprColor::Rgba(_r, _g, _b, _a) => Some(<(u8, u8, u8, u8)>::to_gtk_box),
        }
    }

    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        Some(vec![
            vec![
                FieldLabel::Named(cow_to_static_str(t!("hyprland.hypr_color.red"))),
                FieldLabel::Named(cow_to_static_str(t!("hyprland.hypr_color.green"))),
                FieldLabel::Named(cow_to_static_str(t!("hyprland.hypr_color.blue"))),
            ],
            vec![
                FieldLabel::Named(cow_to_static_str(t!("hyprland.hypr_color.red"))),
                FieldLabel::Named(cow_to_static_str(t!("hyprland.hypr_color.green"))),
                FieldLabel::Named(cow_to_static_str(t!("hyprland.hypr_color.blue"))),
                FieldLabel::Named(cow_to_static_str(t!("hyprland.hypr_color.alpha"))),
            ],
        ])
    }
}

register_togtkbox!(HyprColor);
register_togtkbox_with_separator_names!((u8, u8, u8), (u8, u8, u8, u8));
