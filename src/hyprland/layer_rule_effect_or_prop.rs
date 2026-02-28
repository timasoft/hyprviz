use super::{LayerRuleEffect, LayerRuleProp};
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

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(LayerRuleEffectOrPropDiscriminant))]
pub enum LayerRuleEffectOrProp {
    Effect(LayerRuleEffect),
    Prop(LayerRuleProp),
}

impl HasDiscriminant for LayerRuleEffectOrProp {
    type Discriminant = LayerRuleEffectOrPropDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Effect => Self::Effect(LayerRuleEffect::default()),
            Self::Discriminant::Prop => Self::Prop(LayerRuleProp::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Effect => {
                Self::Effect(LayerRuleEffect::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Prop => {
                Self::Prop(LayerRuleProp::from_str(str).unwrap_or_default())
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::Effect(effect) => Some(effect.to_string()),
            Self::Prop(prop) => Some(prop.to_string()),
        }
    }
}

impl Default for LayerRuleEffectOrProp {
    fn default() -> Self {
        Self::Prop(LayerRuleProp::default())
    }
}

impl FromStr for LayerRuleEffectOrProp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        if let Some(prop) = s.strip_prefix("match:") {
            Ok(Self::Prop(
                LayerRuleProp::from_str(prop).unwrap_or_default(),
            ))
        } else if let Ok(effect) = LayerRuleEffect::from_str(s) {
            Ok(Self::Effect(effect))
        } else if let Ok(prop) = LayerRuleProp::from_str(s) {
            Ok(Self::Prop(prop))
        } else {
            Err(())
        }
    }
}

impl Display for LayerRuleEffectOrProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerRuleEffectOrProp::Effect(effect) => write!(f, "{}", effect),
            LayerRuleEffectOrProp::Prop(prop) => write!(f, "match:{}", prop),
        }
    }
}

impl EnumConfigForGtk for LayerRuleEffectOrProp {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.layer_rule_effect_or_prop.effect"),
            &t!("hyprland.layer_rule_effect_or_prop.prop"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::Effect(_effect) => Some(<(LayerRuleEffect,)>::to_gtk_box),
            Self::Prop(_prop) => Some(<(LayerRuleProp,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(LayerRuleEffectOrProp);
register_togtkbox_with_separator_names!((LayerRuleEffect,), (LayerRuleProp,));
