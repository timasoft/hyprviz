use crate::{
    gtk_converters::{
        EnumConfigForGtk, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(RelativeIdDiscriminant))]
pub enum RelativeId {
    Absolute(u32),
    Previous(u32),
    Next(u32),
}

impl HasDiscriminant for RelativeId {
    type Discriminant = RelativeIdDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Absolute => Self::Absolute(1),
            Self::Discriminant::Previous => Self::Previous(1),
            Self::Discriminant::Next => Self::Next(1),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Absolute => Self::Absolute(str.parse().unwrap_or_default()),
            Self::Discriminant::Previous => Self::Previous(str.parse().unwrap_or_default()),
            Self::Discriminant::Next => Self::Next(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            RelativeId::Absolute(id) => Some(id.to_string()),
            RelativeId::Previous(id) => Some(id.to_string()),
            RelativeId::Next(id) => Some(id.to_string()),
        }
    }
}

impl Default for RelativeId {
    fn default() -> Self {
        RelativeId::Absolute(1)
    }
}

impl FromStr for RelativeId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        match s.chars().next().unwrap() {
            '~' => Ok(RelativeId::Absolute(s[1..].parse::<u32>().unwrap_or(1))),
            '-' => Ok(RelativeId::Previous(s[1..].parse::<u32>().unwrap_or(1))),
            '+' => Ok(RelativeId::Next(s[1..].parse::<u32>().unwrap_or(1))),
            _ => Err(()),
        }
    }
}

impl Display for RelativeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelativeId::Absolute(id) => write!(f, "~{}", id),
            RelativeId::Previous(id) => write!(f, "-{}", id),
            RelativeId::Next(id) => write!(f, "+{}", id),
        }
    }
}

impl EnumConfigForGtk for RelativeId {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.absolute"),
            &t!("gtk_converters.previous"),
            &t!("gtk_converters.next"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            RelativeId::Absolute(_i) => Some(<(u32,)>::to_gtk_box),
            RelativeId::Previous(_i) => Some(<(u32,)>::to_gtk_box),
            RelativeId::Next(_i) => Some(<(u32,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(RelativeId);
register_togtkbox_with_separator_names!((u32,));
