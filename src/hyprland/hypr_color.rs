use crate::{
    gtk_converters::{EnumConfigForGtk, ToGtkBoxWithSeparatorAndNamesBuilder},
    register_togtkbox,
    utils::{HasDiscriminant, ONE_OVER_255},
};
use gtk::{
    Box as GtkBox, ColorDialog, ColorDialogButton, Entry, StringList, gdk::RGBA, prelude::*,
};
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(HyprColorDiscriminant))]
pub enum HyprColor {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
}

impl HyprColor {
    pub fn to_gtk_rgba(self) -> RGBA {
        match self {
            Self::Rgb(r, g, b) => RGBA::new(
                (r as f64 * ONE_OVER_255) as f32,
                (g as f64 * ONE_OVER_255) as f32,
                (b as f64 * ONE_OVER_255) as f32,
                1.0,
            ),
            Self::Rgba(r, g, b, a) => RGBA::new(
                (r as f64 * ONE_OVER_255) as f32,
                (g as f64 * ONE_OVER_255) as f32,
                (b as f64 * ONE_OVER_255) as f32,
                (a as f64 * ONE_OVER_255) as f32,
            ),
        }
    }

    pub fn to_rgb_hex(self) -> String {
        match self {
            Self::Rgb(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
            Self::Rgba(r, g, b, _a) => format!("#{:02X}{:02X}{:02X}", r, g, b),
        }
    }

    pub fn to_rgba_hex(self) -> String {
        match self {
            Self::Rgb(r, g, b) => format!("#{:02X}{:02X}{:02X}FF", r, g, b),
            Self::Rgba(r, g, b, a) => format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a),
        }
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

impl Default for HyprColor {
    fn default() -> Self {
        HyprColor::Rgb(0, 0, 0)
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
                let a = (f64::from_str(rgba_vec[3]).unwrap_or_default() * 255.0).round() as u8;
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
        } else if s.starts_with("#") && s.len() == 7 {
            // #ff0000
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or_default();
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or_default();
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or_default();
            Ok(HyprColor::Rgb(r, g, b))
        } else if s.starts_with("#") && s.len() == 9 {
            // #ff0000ff
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or_default();
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or_default();
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or_default();
            let a = u8::from_str_radix(&s[7..9], 16).unwrap_or_default();
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
                write!(f, "rgba({},{},{},{})", r, g, b, *a as f64 * ONE_OVER_255)
            }
        }
    }
}

fn color_builder(entry: &Entry, has_alpha: bool) -> GtkBox {
    let is_updating = Rc::new(Cell::new(false));

    let mother_box = GtkBox::new(gtk::Orientation::Horizontal, 5);

    let color_dialog = ColorDialog::new();
    color_dialog.set_with_alpha(has_alpha);
    let color_button = ColorDialogButton::new(Some(color_dialog));
    mother_box.append(&color_button);

    let hex_entry = Entry::new();
    hex_entry.set_width_chars(8);
    hex_entry.set_max_width_chars(9);
    match has_alpha {
        true => hex_entry.set_placeholder_text(Some("#RRGGBBAA")),
        false => hex_entry.set_placeholder_text(Some("#RRGGBB")),
    }
    mother_box.append(&hex_entry);

    if let Ok(hypr_color) = entry.text().parse::<HyprColor>() {
        color_button.set_rgba(&hypr_color.to_gtk_rgba());
        match has_alpha {
            true => hex_entry.set_text(&hypr_color.to_rgba_hex()),
            false => hex_entry.set_text(&hypr_color.to_rgb_hex()),
        }
        hex_entry.set_css_classes(&[]);
    }

    let hex_entry_clone = hex_entry.clone();
    let entry_clone = entry.clone();
    let is_updating_clone = is_updating.clone();
    color_button.connect_rgba_notify(move |btn| {
        if is_updating_clone.get() {
            return;
        }
        is_updating_clone.set(true);

        let rgba = btn.rgba();
        let r = (rgba.red() * 255.0).round() as u8;
        let g = (rgba.green() * 255.0).round() as u8;
        let b = (rgba.blue() * 255.0).round() as u8;
        let a = (rgba.alpha() * 255.0).round() as u8;
        let hex = match has_alpha {
            true => format!("#{r:02X}{g:02X}{b:02X}{a:02X}"),
            false => format!("#{r:02X}{g:02X}{b:02X}"),
        };
        let hypr_color = match has_alpha {
            true => HyprColor::Rgba(r, g, b, a),
            false => HyprColor::Rgb(r, g, b),
        };
        hex_entry_clone.set_text(&hex);
        entry_clone.set_text(
            &hypr_color
                .to_str_without_discriminant()
                .expect("HyprColor should be always valid"),
        );

        is_updating_clone.set(false);
    });

    let color_button_clone = color_button.clone();
    let entry_clone = entry.clone();
    let is_updating_clone = is_updating.clone();
    hex_entry.connect_changed(move |entry| {
        if is_updating_clone.get() {
            return;
        }
        is_updating_clone.set(true);

        let text = entry.text().trim().to_string();

        match HyprColor::from_str(&text) {
            Ok(hypr_color) => {
                color_button_clone.set_rgba(&hypr_color.to_gtk_rgba());
                entry.set_css_classes(&[]);
                entry_clone.set_text(
                    &hypr_color
                        .to_str_without_discriminant()
                        .expect("HyprColor should be always valid"),
                );
            }
            Err(_) => entry.set_css_classes(&["error"]),
        }

        is_updating_clone.set(false);
    });

    let is_updating_clone = is_updating.clone();
    entry.connect_changed(move |entry| {
        if is_updating_clone.get() {
            return;
        }
        is_updating_clone.set(true);

        let discriminant = if has_alpha {
            HyprColorDiscriminant::Rgba
        } else {
            HyprColorDiscriminant::Rgb
        };
        let hypr_color = HyprColor::from_discriminant_and_str(discriminant, &entry.text());
        color_button.set_rgba(&hypr_color.to_gtk_rgba());
        match has_alpha {
            true => hex_entry.set_text(&hypr_color.to_rgba_hex()),
            false => hex_entry.set_text(&hypr_color.to_rgb_hex()),
        }
        hex_entry.set_css_classes(&[]);

        is_updating_clone.set(false);
    });

    mother_box
}

impl EnumConfigForGtk for HyprColor {
    fn dropdown_items() -> StringList {
        StringList::new(&["RGB", "RGBA"])
    }

    const SEPARATOR: Option<char> = Some(',');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            HyprColor::Rgb(_r, _g, _b) => Some(|entry, _, _, _| color_builder(entry, false)),
            HyprColor::Rgba(_r, _g, _b, _a) => Some(|entry, _, _, _| color_builder(entry, true)),
        }
    }
}

register_togtkbox!(HyprColor);
