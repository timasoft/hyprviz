use crate::{
    gtk_converters::{
        EnumConfigForGtk, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
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
#[strum_discriminants(name(LayerRulePropDiscriminant))]
pub enum LayerRuleProp {
    Namespace(String),
}

impl HasDiscriminant for LayerRuleProp {
    type Discriminant = LayerRulePropDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Namespace => LayerRuleProp::Namespace("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Namespace => LayerRuleProp::Namespace(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            LayerRuleProp::Namespace(namespace) => Some(namespace.to_string()),
        }
    }
}

impl Default for LayerRuleProp {
    fn default() -> Self {
        LayerRuleProp::Namespace("".to_string())
    }
}

impl FromStr for LayerRuleProp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        Ok(LayerRuleProp::Namespace(s.to_string()))
    }
}

impl Display for LayerRuleProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerRuleProp::Namespace(namespace) => write!(f, "{}", namespace),
        }
    }
}

impl EnumConfigForGtk for LayerRuleProp {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("hyprland.layer_rule_prop.namespace")])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::Namespace(_namespace) => Some(<(String,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(LayerRuleProp);
