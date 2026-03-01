use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNamesBuilder,
        create_spin_button_builder,
    },
    register_togtkbox,
    utils::{HasDiscriminant, MAX_SAFE_STEP_0_01_F64},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(CursorZoomDiscriminant))]
pub enum CursorZoom {
    Toggle(f64),
    Mult(f64),
}

impl HasDiscriminant for CursorZoom {
    type Discriminant = CursorZoomDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Toggle => Self::Toggle(1.0),
            Self::Discriminant::Mult => Self::Mult(1.0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Toggle => Self::Toggle(str.parse().unwrap_or_default()),
            Self::Discriminant::Mult => Self::Mult(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            CursorZoom::Toggle(value) => Some(value.to_string()),
            CursorZoom::Mult(value) => Some(value.to_string()),
        }
    }
}

impl Default for CursorZoom {
    fn default() -> Self {
        Self::Toggle(1.0)
    }
}

impl FromStr for CursorZoom {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let mut is_mult = false;
        let mut zoom_factor = None;
        let parts: Vec<&str> = s.split(' ').collect();

        for part in &parts {
            let part = part.trim();

            if part == "mult" {
                is_mult = true;
            }

            if let Ok(value) = part.parse::<f64>() {
                zoom_factor = Some(value);
            }
        }

        match (is_mult, zoom_factor) {
            (true, Some(value)) => Ok(Self::Mult(value)),
            (false, Some(value)) => Ok(Self::Toggle(value)),
            _ => Err(()),
        }
    }
}

impl Display for CursorZoom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CursorZoom::Toggle(value) => write!(f, "{}", value),
            CursorZoom::Mult(value) => write!(f, "{} mult", value),
        }
    }
}

impl EnumConfigForGtk for CursorZoom {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.cursor_zoom.toggle"),
            &t!("hyprland.cursor_zoom.mult"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            CursorZoom::Toggle(_float) => Some(|entry, _, _, _| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    &FieldLabel::Unnamed,
                )
            }),
            CursorZoom::Mult(_float) => Some(|entry, _, _, _| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    &FieldLabel::Unnamed,
                )
            }),
        }
    }
}

register_togtkbox!(CursorZoom);
