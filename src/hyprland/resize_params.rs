use super::PixelOrPercent;
use crate::{
    gtk_converters::{
        EnumConfigForGtk, ToGtkBoxWithSeparatorAndNames, ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(ResizeParamsDiscriminant))]
pub enum ResizeParams {
    Relative(PixelOrPercent, PixelOrPercent),
    Exact(PixelOrPercent, PixelOrPercent),
}

impl HasDiscriminant for ResizeParams {
    type Discriminant = ResizeParamsDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Relative => {
                Self::Relative(PixelOrPercent::default(), PixelOrPercent::default())
            }
            Self::Discriminant::Exact => {
                Self::Exact(PixelOrPercent::default(), PixelOrPercent::default())
            }
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Relative => match Self::from_str(str) {
                Ok(ResizeParams::Relative(p1, p2)) => ResizeParams::Relative(p1, p2),
                Ok(ResizeParams::Exact(p1, p2)) => ResizeParams::Relative(p1, p2),
                Err(_) => {
                    ResizeParams::Relative(PixelOrPercent::default(), PixelOrPercent::default())
                }
            },
            Self::Discriminant::Exact => match Self::from_str(str) {
                Ok(ResizeParams::Relative(p1, p2)) => ResizeParams::Exact(p1, p2),
                Ok(ResizeParams::Exact(p1, p2)) => ResizeParams::Exact(p1, p2),
                Err(_) => ResizeParams::Exact(PixelOrPercent::default(), PixelOrPercent::default()),
            },
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            ResizeParams::Relative(width, height) => Some(format!("{} {}", width, height)),
            ResizeParams::Exact(width, height) => Some(format!("{} {}", width, height)),
        }
    }
}

impl Default for ResizeParams {
    fn default() -> Self {
        ResizeParams::Relative(PixelOrPercent::default(), PixelOrPercent::default())
    }
}

impl FromStr for ResizeParams {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Some(s) = s.strip_prefix("exact ") {
            let (width, height) = s.split_once(' ').unwrap_or(("", ""));
            let width = width
                .parse::<PixelOrPercent>()
                .unwrap_or(PixelOrPercent::Pixel(0));
            let height = height
                .parse::<PixelOrPercent>()
                .unwrap_or(PixelOrPercent::Pixel(0));
            Ok(ResizeParams::Exact(width, height))
        } else {
            let (width, height) = s.split_once(' ').unwrap_or(("", ""));
            let width = width
                .parse::<PixelOrPercent>()
                .unwrap_or(PixelOrPercent::Pixel(0));
            let height = height
                .parse::<PixelOrPercent>()
                .unwrap_or(PixelOrPercent::Pixel(0));
            Ok(ResizeParams::Relative(width, height))
        }
    }
}

impl Display for ResizeParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResizeParams::Relative(width, height) => write!(f, "{} {}", width, height),
            ResizeParams::Exact(width, height) => write!(f, "exact {} {}", width, height),
        }
    }
}

impl EnumConfigForGtk for ResizeParams {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("gtk_converters.relative"), &t!("gtk_converters.exact")])
    }

    const SEPARATOR: Option<char> = Some(' ');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            ResizeParams::Relative(_x, _y) => Some(<(PixelOrPercent, PixelOrPercent)>::to_gtk_box),
            ResizeParams::Exact(_x, _y) => Some(<(PixelOrPercent, PixelOrPercent)>::to_gtk_box),
        }
    }
}

register_togtkbox!(ResizeParams);
register_togtkbox_with_separator_names!((PixelOrPercent, PixelOrPercent));
