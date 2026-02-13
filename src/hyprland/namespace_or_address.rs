use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(NamespaceOrAddressDiscriminant))]
pub enum NamespaceOrAddress {
    Namespace(String),
    Address(String),
}

impl HasDiscriminant for NamespaceOrAddress {
    type Discriminant = NamespaceOrAddressDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Namespace => NamespaceOrAddress::Namespace("".to_string()),
            Self::Discriminant::Address => NamespaceOrAddress::Address("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Namespace => NamespaceOrAddress::Namespace(str.to_string()),
            Self::Discriminant::Address => NamespaceOrAddress::Address(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            NamespaceOrAddress::Namespace(namespace) => Some(namespace.to_string()),
            NamespaceOrAddress::Address(address) => Some(address.to_string()),
        }
    }
}

impl Default for NamespaceOrAddress {
    fn default() -> Self {
        NamespaceOrAddress::Namespace("".to_string())
    }
}

impl FromStr for NamespaceOrAddress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(address) = s.strip_prefix("address:0x") {
            Ok(NamespaceOrAddress::Address(address.to_string()))
        } else {
            Ok(NamespaceOrAddress::Namespace(s.to_string()))
        }
    }
}

impl Display for NamespaceOrAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceOrAddress::Namespace(namespace) => write!(f, "{}", namespace),
            NamespaceOrAddress::Address(address) => write!(f, "address:0x{}", address),
        }
    }
}

impl EnumConfigForGtk for NamespaceOrAddress {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.namespace_or_address.namespace"),
            &t!("hyprland.namespace_or_address.address"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::Namespace(_namespace) => Some(<(String,)>::to_gtk_box),
            Self::Address(_address) => Some(<(String,)>::to_gtk_box),
        }
    }

    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        Some(vec![vec![], vec![FieldLabel::Named("0x")]])
    }
}

register_togtkbox!(NamespaceOrAddress);
