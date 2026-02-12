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

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(IdOrNameDiscriminant))]
pub enum IdOrName {
    Id(u32),
    Name(String),
}

impl HasDiscriminant for IdOrName {
    type Discriminant = IdOrNameDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(0),
            Self::Discriminant::Name => Self::Name("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(str.parse().unwrap_or_default()),
            Self::Discriminant::Name => Self::Name(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            IdOrName::Id(id) => Some(id.to_string()),
            IdOrName::Name(name) => Some(name.clone()),
        }
    }
}

impl Default for IdOrName {
    fn default() -> Self {
        IdOrName::Id(0)
    }
}

impl FromStr for IdOrName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Ok(id) = s.parse::<u32>() {
            Ok(IdOrName::Id(id))
        } else {
            Ok(IdOrName::Name(s.to_string()))
        }
    }
}

impl Display for IdOrName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdOrName::Id(id) => write!(f, "{}", id),
            IdOrName::Name(name) => write!(f, "{}", name),
        }
    }
}

impl EnumConfigForGtk for IdOrName {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.id_or_name.id"),
            &t!("hyprland.id_or_name.name"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            IdOrName::Id(_id) => Some(<(u32,)>::to_gtk_box),
            IdOrName::Name(_name) => Some(<(String,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(IdOrName);
register_togtkbox_with_separator_names!((u32,), (String,));
