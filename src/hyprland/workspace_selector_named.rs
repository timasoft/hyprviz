use crate::{
    gtk_converters::{
        EnumConfigForGtk, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox,
    utils::{HasDiscriminant, parse_bool},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorkspaceSelectorNamedDiscriminant))]
pub enum WorkspaceSelectorNamed {
    IsNamed(bool),
    Starts(String),
    Ends(String),
}

impl HasDiscriminant for WorkspaceSelectorNamed {
    type Discriminant = WorkspaceSelectorNamedDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::IsNamed => Self::IsNamed(false),
            Self::Discriminant::Starts => Self::Starts("".to_string()),
            Self::Discriminant::Ends => Self::Ends("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::IsNamed => Self::IsNamed(parse_bool(str).unwrap_or(false)),
            Self::Discriminant::Starts => Self::Starts(str.to_string()),
            Self::Discriminant::Ends => Self::Ends(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::IsNamed(is_named) => Some(format!("{}", is_named)),
            Self::Starts(prefix) => Some(prefix.to_string()),
            Self::Ends(suffix) => Some(suffix.to_string()),
        }
    }
}

impl Default for WorkspaceSelectorNamed {
    fn default() -> Self {
        WorkspaceSelectorNamed::IsNamed(false)
    }
}

impl FromStr for WorkspaceSelectorNamed {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(starts_with) = s.strip_prefix("s:") {
            Ok(WorkspaceSelectorNamed::Starts(starts_with.to_string()))
        } else if let Some(ends_with) = s.strip_prefix("e:") {
            Ok(WorkspaceSelectorNamed::Ends(ends_with.to_string()))
        } else {
            Ok(WorkspaceSelectorNamed::IsNamed(
                parse_bool(s).unwrap_or(false),
            ))
        }
    }
}

impl Display for WorkspaceSelectorNamed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSelectorNamed::IsNamed(is_named) => write!(f, "{}", is_named),
            WorkspaceSelectorNamed::Starts(prefix) => write!(f, "s:{}", prefix),
            WorkspaceSelectorNamed::Ends(suffix) => write!(f, "e:{}", suffix),
        }
    }
}

impl EnumConfigForGtk for WorkspaceSelectorNamed {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.workspace_selector_named.is_named"),
            &t!("hyprland.workspace_selector_named.starts_with"),
            &t!("hyprland.workspace_selector_named.ends_with"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::IsNamed(_is_named) => Some(<(bool,)>::to_gtk_box),
            Self::Starts(_starts_with) => Some(<(String,)>::to_gtk_box),
            Self::Ends(_ends_with) => Some(<(String,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(WorkspaceSelectorNamed);
