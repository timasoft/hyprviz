use super::{AboveLock, AnimationStyle};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, ToGtkBoxWithSeparatorAndNames,
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
#[strum_discriminants(name(LayerRuleEffectDiscriminant))]
pub enum LayerRuleEffect {
    #[default]
    NoAnimOn,
    NoAnimOff,
    BlurOn,
    BlurOff,
    BlurPopupsOn,
    BlurPopupsOff,
    IgnoreAlpha(f64),
    DimAroundOn,
    DimAroundOff,
    XrayOn,
    XrayOff,
    Animation(AnimationStyle),
    Order(i32),
    AboveLock(AboveLock),
    NoScreenShareOn,
    NoScreenShareOff,
}

impl HasDiscriminant for LayerRuleEffect {
    type Discriminant = LayerRuleEffectDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::NoAnimOn => Self::NoAnimOn,
            Self::Discriminant::NoAnimOff => Self::NoAnimOff,
            Self::Discriminant::BlurOn => Self::BlurOn,
            Self::Discriminant::BlurOff => Self::BlurOff,
            Self::Discriminant::BlurPopupsOn => Self::BlurPopupsOn,
            Self::Discriminant::BlurPopupsOff => Self::BlurPopupsOff,
            Self::Discriminant::IgnoreAlpha => Self::IgnoreAlpha(0.0),
            Self::Discriminant::DimAroundOn => Self::DimAroundOn,
            Self::Discriminant::DimAroundOff => Self::DimAroundOff,
            Self::Discriminant::XrayOn => Self::XrayOn,
            Self::Discriminant::XrayOff => Self::XrayOff,
            Self::Discriminant::Animation => Self::Animation(AnimationStyle::default()),
            Self::Discriminant::Order => Self::Order(0),
            Self::Discriminant::AboveLock => Self::AboveLock(AboveLock::default()),
            Self::Discriminant::NoScreenShareOn => Self::NoScreenShareOn,
            Self::Discriminant::NoScreenShareOff => Self::NoScreenShareOff,
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::NoAnimOn => Self::NoAnimOn,
            Self::Discriminant::NoAnimOff => Self::NoAnimOff,
            Self::Discriminant::BlurOn => Self::BlurOn,
            Self::Discriminant::BlurOff => Self::BlurOff,
            Self::Discriminant::BlurPopupsOn => Self::BlurPopupsOn,
            Self::Discriminant::BlurPopupsOff => Self::BlurPopupsOff,
            Self::Discriminant::IgnoreAlpha => Self::IgnoreAlpha(str.parse().unwrap_or(0.0)),
            Self::Discriminant::DimAroundOn => Self::DimAroundOn,
            Self::Discriminant::DimAroundOff => Self::DimAroundOff,
            Self::Discriminant::XrayOn => Self::XrayOn,
            Self::Discriminant::XrayOff => Self::XrayOff,
            Self::Discriminant::Animation => Self::Animation(str.parse().unwrap_or_default()),
            Self::Discriminant::Order => Self::Order(str.parse().unwrap_or(0)),
            Self::Discriminant::AboveLock => Self::AboveLock(str.parse().unwrap_or_default()),
            Self::Discriminant::NoScreenShareOn => Self::NoScreenShareOn,
            Self::Discriminant::NoScreenShareOff => Self::NoScreenShareOff,
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            LayerRuleEffect::NoAnimOn => None,
            LayerRuleEffect::NoAnimOff => None,
            LayerRuleEffect::BlurOn => None,
            LayerRuleEffect::BlurOff => None,
            LayerRuleEffect::BlurPopupsOn => None,
            LayerRuleEffect::BlurPopupsOff => None,
            LayerRuleEffect::IgnoreAlpha(value) => Some(format!("{}", value)),
            LayerRuleEffect::DimAroundOn => None,
            LayerRuleEffect::DimAroundOff => None,
            LayerRuleEffect::XrayOn => None,
            LayerRuleEffect::XrayOff => None,
            LayerRuleEffect::Animation(value) => Some(value.to_string()),
            LayerRuleEffect::Order(value) => Some(value.to_string()),
            LayerRuleEffect::AboveLock(value) => Some(value.to_string()),
            LayerRuleEffect::NoScreenShareOn => None,
            LayerRuleEffect::NoScreenShareOff => None,
        }
    }

    fn custom_split(_discriminant: Self::Discriminant) -> Option<fn(&str) -> Vec<&str>> {
        None
    }
}

impl FromStr for LayerRuleEffect {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }
        let (part1, part2) = s.split_once(' ').unwrap_or((s, ""));
        match part1.trim().to_lowercase().as_str() {
            "no_anim" => match parse_bool(part2) {
                Some(true) => Ok(LayerRuleEffect::NoAnimOn),
                Some(false) => Ok(LayerRuleEffect::NoAnimOff),
                None => Ok(LayerRuleEffect::NoAnimOff),
            },
            "noanim" => Ok(LayerRuleEffect::NoAnimOn),
            "blur" => match parse_bool(part2) {
                Some(true) => Ok(LayerRuleEffect::BlurOn),
                Some(false) => Ok(LayerRuleEffect::BlurOff),
                None if part2.is_empty() => Ok(LayerRuleEffect::BlurOn),
                None => Ok(LayerRuleEffect::BlurOff),
            },
            "blur_popups" => match parse_bool(part2) {
                Some(true) => Ok(LayerRuleEffect::BlurPopupsOn),
                Some(false) => Ok(LayerRuleEffect::BlurPopupsOff),
                None => Ok(LayerRuleEffect::BlurPopupsOff),
            },
            "blurpopups" => Ok(LayerRuleEffect::BlurPopupsOn),
            "ignore_alpha" | "ignorealpha" => Ok(LayerRuleEffect::IgnoreAlpha(
                part2.parse().unwrap_or(0.0f64).clamp(0.0, 1.0),
            )),
            "ignorezero" => Ok(LayerRuleEffect::IgnoreAlpha(0.0)),
            "dim_around" => match parse_bool(part2) {
                Some(true) => Ok(LayerRuleEffect::DimAroundOn),
                Some(false) => Ok(LayerRuleEffect::DimAroundOff),
                None => Ok(LayerRuleEffect::DimAroundOff),
            },
            "dimaround" => Ok(LayerRuleEffect::DimAroundOn),
            "xray" => match parse_bool(part2) {
                Some(true) => Ok(LayerRuleEffect::XrayOn),
                Some(false) => Ok(LayerRuleEffect::XrayOff),
                None => Ok(LayerRuleEffect::XrayOff),
            },
            "animation" => Ok(LayerRuleEffect::Animation(
                part2.parse().unwrap_or_default(),
            )),
            "order" => Ok(LayerRuleEffect::Order(part2.parse().unwrap_or(0))),
            "above_lock" => Ok(LayerRuleEffect::AboveLock(
                part2.parse().unwrap_or_default(),
            )),
            "abovelock" => match parse_bool(part2) {
                Some(true) => Ok(LayerRuleEffect::AboveLock(AboveLock::AboveLockInteractable)),
                Some(false) => Ok(LayerRuleEffect::AboveLock(AboveLock::AboveLock)),
                None => Ok(LayerRuleEffect::AboveLock(AboveLock::AboveLock)),
            },
            "no_screen_share" | "noscreenshare" => match parse_bool(part2) {
                Some(true) => Ok(LayerRuleEffect::NoScreenShareOn),
                Some(false) => Ok(LayerRuleEffect::NoScreenShareOff),
                None => Ok(LayerRuleEffect::NoScreenShareOff),
            },
            _ => Err(()),
        }
    }
}

impl Display for LayerRuleEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerRuleEffect::NoAnimOn => write!(f, "no_anim on"),
            LayerRuleEffect::NoAnimOff => write!(f, "no_anim off"),
            LayerRuleEffect::BlurOn => write!(f, "blur on"),
            LayerRuleEffect::BlurOff => write!(f, "blur off"),
            LayerRuleEffect::BlurPopupsOn => write!(f, "blur_popups on"),
            LayerRuleEffect::BlurPopupsOff => write!(f, "blur_popups off"),
            LayerRuleEffect::IgnoreAlpha(value) => write!(f, "ignore_alpha {}", value),
            LayerRuleEffect::DimAroundOn => write!(f, "dim_around on"),
            LayerRuleEffect::DimAroundOff => write!(f, "dim_around off"),
            LayerRuleEffect::XrayOn => write!(f, "xray on"),
            LayerRuleEffect::XrayOff => write!(f, "xray off"),
            LayerRuleEffect::Animation(value) => write!(f, "animation {}", value),
            LayerRuleEffect::Order(value) => write!(f, "order {}", value),
            LayerRuleEffect::AboveLock(value) => write!(f, "above_lock {}", value),
            LayerRuleEffect::NoScreenShareOn => write!(f, "no_screen_share on"),
            LayerRuleEffect::NoScreenShareOff => write!(f, "no_screen_share off"),
        }
    }
}

impl EnumConfigForGtk for LayerRuleEffect {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.layer_rule_effect.no_anim_on"),
            &t!("hyprland.layer_rule_effect.no_anim_off"),
            &t!("hyprland.layer_rule_effect.blur_on"),
            &t!("hyprland.layer_rule_effect.blur_off"),
            &t!("hyprland.layer_rule_effect.blur_popups_on"),
            &t!("hyprland.layer_rule_effect.blur_popups_off"),
            &t!("hyprland.layer_rule_effect.ignore_alpha"),
            &t!("hyprland.layer_rule_effect.dim_around_on"),
            &t!("hyprland.layer_rule_effect.dim_around_off"),
            &t!("hyprland.layer_rule_effect.xray_on"),
            &t!("hyprland.layer_rule_effect.xray_off"),
            &t!("hyprland.layer_rule_effect.animation"),
            &t!("hyprland.layer_rule_effect.order"),
            &t!("hyprland.layer_rule_effect.above_lock"),
            &t!("hyprland.layer_rule_effect.no_screen_share_on"),
            &t!("hyprland.layer_rule_effect.no_screen_share_off"),
        ])
    }

    const SEPARATOR: Option<char> = Some(' ');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            LayerRuleEffect::NoAnimOn => None,
            LayerRuleEffect::NoAnimOff => None,
            LayerRuleEffect::BlurOn => None,
            LayerRuleEffect::BlurOff => None,
            LayerRuleEffect::BlurPopupsOn => None,
            LayerRuleEffect::BlurPopupsOff => None,
            Self::IgnoreAlpha(_float) => Some(|entry, _separator, _names, _custom_split| {
                create_spin_button_builder(0.0, 1.0, 0.01)(entry, &FieldLabel::Unnamed)
            }),
            LayerRuleEffect::DimAroundOn => None,
            LayerRuleEffect::DimAroundOff => None,
            LayerRuleEffect::XrayOn => None,
            LayerRuleEffect::XrayOff => None,
            LayerRuleEffect::Animation(_) => Some(<(AnimationStyle,)>::to_gtk_box),
            LayerRuleEffect::Order(_) => Some(<(i32,)>::to_gtk_box),
            LayerRuleEffect::AboveLock(_) => Some(<(AboveLock,)>::to_gtk_box),
            LayerRuleEffect::NoScreenShareOn => None,
            LayerRuleEffect::NoScreenShareOff => None,
        }
    }
}

register_togtkbox!(LayerRuleEffect);
register_togtkbox_with_separator_names!((AnimationStyle,), (i32,), (AboveLock,));
