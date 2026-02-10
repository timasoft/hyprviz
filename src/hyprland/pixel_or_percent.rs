use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder, create_spin_button_builder,
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
#[strum_discriminants(name(PixelOrPercentDiscriminant))]
pub enum PixelOrPercent {
    Pixel(i32),
    Percent(f64),
}

impl HasDiscriminant for PixelOrPercent {
    type Discriminant = PixelOrPercentDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Pixel => Self::Pixel(0),
            Self::Discriminant::Percent => Self::Percent(0.0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Pixel => Self::Pixel(str.parse().unwrap_or_default()),
            Self::Discriminant::Percent => Self::Percent(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            PixelOrPercent::Pixel(p) => Some(p.to_string()),
            PixelOrPercent::Percent(p) => Some(format!("{:.2}", p)),
        }
    }
}

impl Default for PixelOrPercent {
    fn default() -> Self {
        PixelOrPercent::Pixel(0)
    }
}

impl FromStr for PixelOrPercent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Ok(p) = s.parse::<i32>() {
            Ok(PixelOrPercent::Pixel(p))
        } else if let Some(stripped) = s.strip_suffix("%") {
            if let Ok(p) = stripped.parse::<f64>() {
                Ok(PixelOrPercent::Percent(p))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl Display for PixelOrPercent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PixelOrPercent::Pixel(p) => write!(f, "{}", p),
            PixelOrPercent::Percent(p) => write!(f, "{}%", p),
        }
    }
}

impl EnumConfigForGtk for PixelOrPercent {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("gtk_converters.pixel"), &t!("gtk_converters.percent")])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            PixelOrPercent::Pixel(_foo) => Some(<(i32,)>::to_gtk_box),
            PixelOrPercent::Percent(_foo) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
        }
    }

    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        Some(vec![
            vec![FieldLabel::Unnamed],
            vec![FieldLabel::Named("%")],
        ])
    }
}

register_togtkbox!(PixelOrPercent);
register_togtkbox_with_separator_names!((i32,));
