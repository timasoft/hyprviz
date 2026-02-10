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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(ChangeGroupActiveDiscriminant))]
pub enum ChangeGroupActive {
    Back,
    #[default]
    Forward,
    Index(u32),
}

impl HasDiscriminant for ChangeGroupActive {
    type Discriminant = ChangeGroupActiveDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Back => Self::Back,
            Self::Discriminant::Forward => Self::Forward,
            Self::Discriminant::Index => Self::Index(0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Back => Self::Back,
            Self::Discriminant::Forward => Self::Forward,
            Self::Discriminant::Index => Self::Index(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            ChangeGroupActive::Back => None,
            ChangeGroupActive::Forward => None,
            ChangeGroupActive::Index(index) => Some(index.to_string()),
        }
    }
}

impl FromStr for ChangeGroupActive {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        match s {
            "b" => Ok(ChangeGroupActive::Back),
            "f" => Ok(ChangeGroupActive::Forward),
            index => Ok(ChangeGroupActive::Index(
                index.parse::<u32>().unwrap_or_default().saturating_sub(1),
            )),
        }
    }
}

impl Display for ChangeGroupActive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeGroupActive::Back => write!(f, "b"),
            ChangeGroupActive::Forward => write!(f, "f"),
            ChangeGroupActive::Index(index) => write!(f, "{}", index.saturating_add(1)),
        }
    }
}

impl EnumConfigForGtk for ChangeGroupActive {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.back"),
            &t!("gtk_converters.forward"),
            &t!("gtk_converters.index"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            ChangeGroupActive::Back => None,
            ChangeGroupActive::Forward => None,
            ChangeGroupActive::Index(_index) => Some(<(u32,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(ChangeGroupActive);
register_togtkbox_with_separator_names!((u32,));
