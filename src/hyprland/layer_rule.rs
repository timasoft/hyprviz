use super::{AnimationStyle, OnOrOffOrUnset};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder, create_spin_button_builder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, parse_bool},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, EnumDiscriminants, Default)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(LayerRuleDiscriminant))]
pub enum LayerRule {
    #[default]
    Unset,
    NoAnim,
    Blur,
    BlurPopups,
    IgnoreAlpha(f64),
    IgnoreZero,
    DimAround,
    Xray(OnOrOffOrUnset),
    Animation(AnimationStyle),
    Order(i32),
    AboveLock,
    AboveLockInteractable,
}

impl HasDiscriminant for LayerRule {
    type Discriminant = LayerRuleDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Unset => LayerRule::Unset,
            Self::Discriminant::NoAnim => LayerRule::NoAnim,
            Self::Discriminant::Blur => LayerRule::Blur,
            Self::Discriminant::BlurPopups => LayerRule::BlurPopups,
            Self::Discriminant::IgnoreAlpha => LayerRule::IgnoreAlpha(0.0),
            Self::Discriminant::IgnoreZero => LayerRule::IgnoreZero,
            Self::Discriminant::DimAround => LayerRule::DimAround,
            Self::Discriminant::Xray => LayerRule::Xray(OnOrOffOrUnset::default()),
            Self::Discriminant::Animation => LayerRule::Animation(AnimationStyle::default()),
            Self::Discriminant::Order => LayerRule::Order(0),
            Self::Discriminant::AboveLock => LayerRule::AboveLock,
            Self::Discriminant::AboveLockInteractable => LayerRule::AboveLockInteractable,
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Unset => LayerRule::Unset,
            Self::Discriminant::NoAnim => LayerRule::NoAnim,
            Self::Discriminant::Blur => LayerRule::Blur,
            Self::Discriminant::BlurPopups => LayerRule::BlurPopups,
            Self::Discriminant::IgnoreAlpha => LayerRule::IgnoreAlpha(str.parse().unwrap_or(0.0)),
            Self::Discriminant::IgnoreZero => LayerRule::IgnoreZero,
            Self::Discriminant::DimAround => LayerRule::DimAround,
            Self::Discriminant::Xray => {
                LayerRule::Xray(OnOrOffOrUnset::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Animation => {
                LayerRule::Animation(AnimationStyle::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Order => LayerRule::Order(str.parse().unwrap_or(0)),
            Self::Discriminant::AboveLock => LayerRule::AboveLock,
            Self::Discriminant::AboveLockInteractable => LayerRule::AboveLockInteractable,
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            LayerRule::Unset => None,
            LayerRule::NoAnim => None,
            LayerRule::Blur => None,
            LayerRule::BlurPopups => None,
            LayerRule::IgnoreAlpha(value) => Some(format!("{}", value)),
            LayerRule::IgnoreZero => None,
            LayerRule::DimAround => None,
            LayerRule::Xray(value) => Some(value.to_string()),
            LayerRule::Animation(value) => Some(value.to_string()),
            LayerRule::Order(value) => Some(value.to_string()),
            LayerRule::AboveLock => None,
            LayerRule::AboveLockInteractable => None,
        }
    }
}

impl FromStr for LayerRule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let (discriminant, str) = s.split_once(' ').unwrap_or((s, ""));
        let discriminant = discriminant.trim();
        let str = str.trim();

        match discriminant.to_lowercase().as_str() {
            "unset" => Ok(LayerRule::Unset),
            "noanim" => Ok(LayerRule::NoAnim),
            "blur" => Ok(LayerRule::Blur),
            "blurpopups" => Ok(LayerRule::BlurPopups),
            "ignorealpha" => Ok(LayerRule::IgnoreAlpha(
                str.parse().unwrap_or(0.0f64).clamp(0.0, 1.0),
            )),
            "ignorezero" => Ok(LayerRule::IgnoreZero),
            "dimaround" => Ok(LayerRule::DimAround),
            "xray" => Ok(LayerRule::Xray(
                OnOrOffOrUnset::from_str(str).unwrap_or_default(),
            )),
            "animation" => Ok(LayerRule::Animation(str.parse().unwrap_or_default())),
            "order" => Ok(LayerRule::Order(str.parse().unwrap_or(0))),
            "abovelock" => {
                if let Some(true) = parse_bool(str) {
                    Ok(LayerRule::AboveLockInteractable)
                } else {
                    Ok(LayerRule::AboveLock)
                }
            }
            _ => Err(()),
        }
    }
}

impl Display for LayerRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerRule::Unset => write!(f, "unset"),
            LayerRule::NoAnim => write!(f, "noanim"),
            LayerRule::Blur => write!(f, "blur"),
            LayerRule::BlurPopups => write!(f, "blurpopups"),
            LayerRule::IgnoreAlpha(value) => write!(f, "ignorealpha {}", value),
            LayerRule::IgnoreZero => write!(f, "ignorezero"),
            LayerRule::DimAround => write!(f, "dimaround"),
            LayerRule::Xray(value) => write!(f, "xray {}", value),
            LayerRule::Animation(value) => write!(f, "animation {}", value),
            LayerRule::Order(value) => write!(f, "order {}", value),
            LayerRule::AboveLock => write!(f, "abovelock"),
            LayerRule::AboveLockInteractable => write!(f, "abovelock true"),
        }
    }
}

impl EnumConfigForGtk for LayerRule {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.unset"),
            &t!("gtk_converters.no_anim"),
            &t!("gtk_converters.blur"),
            &t!("gtk_converters.blur_popups"),
            &t!("gtk_converters.ignore_alpha"),
            &t!("gtk_converters.ignore_zero"),
            &t!("gtk_converters.dim_around"),
            &t!("gtk_converters.xray"),
            &t!("gtk_converters.animation"),
            &t!("gtk_converters.order"),
            &t!("gtk_converters.above_lock"),
            &t!("gtk_converters.above_lock_interactable"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::Unset => None,
            Self::NoAnim => None,
            Self::Blur => None,
            Self::BlurPopups => None,
            Self::IgnoreAlpha(_float) => Some(|entry, _separator, _names, _custom_split| {
                create_spin_button_builder(0.0, 1.0, 0.01)(entry, &FieldLabel::Unnamed)
            }),
            Self::IgnoreZero => None,
            Self::DimAround => None,
            Self::Xray(_on_or_off_or_unset) => Some(<(OnOrOffOrUnset,)>::to_gtk_box),
            Self::Animation(_animation_style) => Some(<(AnimationStyle,)>::to_gtk_box),
            Self::Order(_i32) => Some(<(i32,)>::to_gtk_box),
            Self::AboveLock => None,
            Self::AboveLockInteractable => None,
        }
    }
}

register_togtkbox!(LayerRule);
register_togtkbox_with_separator_names!((OnOrOffOrUnset,), (AnimationStyle,), (i32,));
