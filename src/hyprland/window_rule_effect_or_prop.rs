use super::{
    WindowRuleDynamicEffect, WindowRuleFullscreenState, WindowRuleProp, WindowRuleStaticEffect,
};
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
#[strum_discriminants(name(WindowRuleEffectOrPropDiscriminant))]
pub enum WindowRuleEffectOrProp {
    StaticEffect(WindowRuleStaticEffect),
    DynamicEffect(WindowRuleDynamicEffect),
    Prop(WindowRuleProp),
}

impl HasDiscriminant for WindowRuleEffectOrProp {
    type Discriminant = WindowRuleEffectOrPropDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::StaticEffect => {
                Self::StaticEffect(WindowRuleStaticEffect::default())
            }
            Self::Discriminant::DynamicEffect => {
                Self::DynamicEffect(WindowRuleDynamicEffect::default())
            }
            Self::Discriminant::Prop => Self::Prop(WindowRuleProp::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::StaticEffect => {
                Self::StaticEffect(WindowRuleStaticEffect::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::DynamicEffect => {
                Self::DynamicEffect(WindowRuleDynamicEffect::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Prop => {
                Self::Prop(WindowRuleProp::from_str(str).unwrap_or_default())
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::StaticEffect(effect) => Some(effect.to_string()),
            Self::DynamicEffect(effect) => Some(effect.to_string()),
            Self::Prop(prop) => Some(prop.to_string()),
        }
    }
}

impl Default for WindowRuleEffectOrProp {
    fn default() -> Self {
        Self::Prop(WindowRuleProp::default())
    }
}

impl FromStr for WindowRuleEffectOrProp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        // windowrulev2
        if let Some((parameter, value)) = s.split_once(':')
            && parameter.trim() != "match"
        {
            let value = value.trim();
            let parameter_new = match parameter.trim() {
                "inicialClass" => "inicial_class",
                "inicialTitle" => "inicial_title",
                "floating" => "float",
                "pinned" => "pin",
                "fullscreenstate" => {
                    let mut parts = value.split_whitespace();

                    let internal_state = parts
                        .next()
                        .and_then(|s| WindowRuleFullscreenState::from_str(s).ok());

                    let client_state = parts
                        .next()
                        .and_then(|s| WindowRuleFullscreenState::from_str(s).ok());

                    if let Some(state) = client_state {
                        return Ok(Self::Prop(WindowRuleProp::FullscreenStateClient(state)));
                    }

                    if let Some(state) = internal_state {
                        return Ok(Self::Prop(WindowRuleProp::FullscreenStateInternal(state)));
                    }

                    return Ok(Self::Prop(WindowRuleProp::FullscreenStateClient(
                        WindowRuleFullscreenState::default(),
                    )));
                }
                "onworkspace" => "workspace",
                "xdgTag" => "xdg_tag",
                parameter => parameter,
            };

            let prop = format!("{} {}", parameter_new, value);
            if let Ok(prop) = WindowRuleProp::from_str(&prop) {
                return Ok(Self::Prop(prop));
            }
        }

        if let Some(prop) = s.strip_prefix("match:") {
            Ok(Self::Prop(
                WindowRuleProp::from_str(prop).unwrap_or_default(),
            ))
        } else if let Ok(effect) = WindowRuleStaticEffect::from_str(s) {
            Ok(Self::StaticEffect(effect))
        } else if let Ok(effect) = WindowRuleDynamicEffect::from_str(s) {
            Ok(Self::DynamicEffect(effect))
        } else {
            Err(())
        }
    }
}

impl Display for WindowRuleEffectOrProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRuleEffectOrProp::StaticEffect(effect) => write!(f, "{}", effect),
            WindowRuleEffectOrProp::DynamicEffect(effect) => write!(f, "{}", effect),
            WindowRuleEffectOrProp::Prop(prop) => write!(f, "match:{}", prop),
        }
    }
}

impl EnumConfigForGtk for WindowRuleEffectOrProp {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.window_rule_effect_or_prop.static_effect"),
            &t!("hyprland.window_rule_effect_or_prop.dynamic_effect"),
            &t!("hyprland.window_rule_effect_or_prop.prop"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::StaticEffect(_effect) => Some(<(WindowRuleStaticEffect,)>::to_gtk_box),
            Self::DynamicEffect(_effect) => Some(<(WindowRuleDynamicEffect,)>::to_gtk_box),
            Self::Prop(_prop) => Some(<(WindowRuleProp,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(WindowRuleEffectOrProp);
register_togtkbox_with_separator_names!((WindowRuleProp,));
