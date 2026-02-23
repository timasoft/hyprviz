use super::{AnimationStyle, BorderColor, HyprOpacity, IdleIngibitMode, TagToggleState};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder, create_spin_button_builder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, MAX_SAFE_STEP_0_01_F64, parse_bool},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(SetPropDiscriminant))]
pub enum WindowRuleDynamicEffect {
    PersistentSizeOn,
    PersistentSizeOff,
    NoMaxSizeOn,
    NoMaxSizeOff,
    StayFocusedOn,
    StayFocusedOff,
    Animation(AnimationStyle),
    BorderColor(BorderColor),
    IdleIngibit(IdleIngibitMode),
    Opacity(HyprOpacity),
    Tag(TagToggleState, String),
    MaxSize(u32, u32),
    MinSize(u32, u32),
    BorderSize(u32),
    Rounding(u32),
    RoundingPower(f64),
    AllowsInputOn,
    AllowsInputOff,
    DimAroundOn,
    DimAroundOff,
    DecorateOn,
    DecorateOff,
    FocusOnActivateOn,
    FocusOnActivateOff,
    KeepAspectRatioOn,
    KeepAspectRatioOff,
    NearestNeighborOn,
    NearestNeighborOff,
    NoAnimOn,
    NoAnimOff,
    NoBlurOn,
    NoBlurOff,
    NoDimOn,
    NoDimOff,
    NoFocusOn,
    NoFocusOff,
    NoFollowMouseOn,
    NoFollowMouseOff,
    NoShadowOn,
    NoShadowOff,
    NoShortcutsInhibitOn,
    NoShortcutsInhibitOff,
    NoScreenShareOn,
    NoScreenShareOff,
    NoVRROn,
    NoVRROff,
    OpaqueOn,
    OpaqueOff,
    ForceRGBXOn,
    ForceRGBXOff,
    SyncFullscreenOn,
    SyncFullscreenOff,
    ImmediateOn,
    ImmediateOff,
    XrayOn,
    XrayOff,
    RenderUnfocusedOn,
    RenderUnfocusedOff,
    ScrollMouse(f64),
    ScrollTouchpad(f64),
}

impl HasDiscriminant for WindowRuleDynamicEffect {
    type Discriminant = SetPropDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::PersistentSizeOn => Self::PersistentSizeOn,
            Self::Discriminant::PersistentSizeOff => Self::PersistentSizeOff,
            Self::Discriminant::NoMaxSizeOn => Self::NoMaxSizeOn,
            Self::Discriminant::NoMaxSizeOff => Self::NoMaxSizeOff,
            Self::Discriminant::StayFocusedOn => Self::StayFocusedOn,
            Self::Discriminant::StayFocusedOff => Self::StayFocusedOff,
            Self::Discriminant::Animation => Self::Animation(AnimationStyle::default()),
            Self::Discriminant::BorderColor => Self::BorderColor(BorderColor::default()),
            Self::Discriminant::IdleIngibit => Self::IdleIngibit(IdleIngibitMode::default()),
            Self::Discriminant::Opacity => Self::Opacity(HyprOpacity::default()),
            Self::Discriminant::Tag => Self::Tag(TagToggleState::Toggle, "".to_string()),
            Self::Discriminant::MaxSize => Self::MaxSize(0, 0),
            Self::Discriminant::MinSize => Self::MinSize(0, 0),
            Self::Discriminant::BorderSize => Self::BorderSize(0),
            Self::Discriminant::Rounding => Self::Rounding(0),
            Self::Discriminant::RoundingPower => Self::RoundingPower(0.0),
            Self::Discriminant::AllowsInputOn => Self::AllowsInputOn,
            Self::Discriminant::AllowsInputOff => Self::AllowsInputOff,
            Self::Discriminant::DimAroundOn => Self::DimAroundOn,
            Self::Discriminant::DimAroundOff => Self::DimAroundOff,
            Self::Discriminant::DecorateOn => Self::DecorateOn,
            Self::Discriminant::DecorateOff => Self::DecorateOff,
            Self::Discriminant::FocusOnActivateOn => Self::FocusOnActivateOn,
            Self::Discriminant::FocusOnActivateOff => Self::FocusOnActivateOff,
            Self::Discriminant::KeepAspectRatioOn => Self::KeepAspectRatioOn,
            Self::Discriminant::KeepAspectRatioOff => Self::KeepAspectRatioOff,
            Self::Discriminant::NearestNeighborOn => Self::NearestNeighborOn,
            Self::Discriminant::NearestNeighborOff => Self::NearestNeighborOff,
            Self::Discriminant::NoAnimOn => Self::NoAnimOn,
            Self::Discriminant::NoAnimOff => Self::NoAnimOff,
            Self::Discriminant::NoBlurOn => Self::NoBlurOn,
            Self::Discriminant::NoBlurOff => Self::NoBlurOff,
            Self::Discriminant::NoDimOn => Self::NoDimOn,
            Self::Discriminant::NoDimOff => Self::NoDimOff,
            Self::Discriminant::NoFocusOn => Self::NoFocusOn,
            Self::Discriminant::NoFocusOff => Self::NoFocusOff,
            Self::Discriminant::NoFollowMouseOn => Self::NoFollowMouseOn,
            Self::Discriminant::NoFollowMouseOff => Self::NoFollowMouseOff,
            Self::Discriminant::NoShadowOn => Self::NoShadowOn,
            Self::Discriminant::NoShadowOff => Self::NoShadowOff,
            Self::Discriminant::NoShortcutsInhibitOn => Self::NoShortcutsInhibitOn,
            Self::Discriminant::NoShortcutsInhibitOff => Self::NoShortcutsInhibitOff,
            Self::Discriminant::NoScreenShareOn => Self::NoScreenShareOn,
            Self::Discriminant::NoScreenShareOff => Self::NoScreenShareOff,
            Self::Discriminant::NoVRROn => Self::NoVRROn,
            Self::Discriminant::NoVRROff => Self::NoVRROff,
            Self::Discriminant::OpaqueOn => Self::OpaqueOn,
            Self::Discriminant::OpaqueOff => Self::OpaqueOff,
            Self::Discriminant::ForceRGBXOn => Self::ForceRGBXOn,
            Self::Discriminant::ForceRGBXOff => Self::ForceRGBXOff,
            Self::Discriminant::SyncFullscreenOn => Self::SyncFullscreenOn,
            Self::Discriminant::SyncFullscreenOff => Self::SyncFullscreenOff,
            Self::Discriminant::ImmediateOn => Self::ImmediateOn,
            Self::Discriminant::ImmediateOff => Self::ImmediateOff,
            Self::Discriminant::XrayOn => Self::XrayOn,
            Self::Discriminant::XrayOff => Self::XrayOff,
            Self::Discriminant::RenderUnfocusedOn => Self::RenderUnfocusedOn,
            Self::Discriminant::RenderUnfocusedOff => Self::RenderUnfocusedOff,
            Self::Discriminant::ScrollMouse => Self::ScrollMouse(0.0),
            Self::Discriminant::ScrollTouchpad => Self::ScrollTouchpad(0.0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::PersistentSizeOn => Self::PersistentSizeOn,
            Self::Discriminant::PersistentSizeOff => Self::PersistentSizeOff,
            Self::Discriminant::NoMaxSizeOn => Self::NoMaxSizeOn,
            Self::Discriminant::NoMaxSizeOff => Self::NoMaxSizeOff,
            Self::Discriminant::StayFocusedOn => Self::StayFocusedOn,
            Self::Discriminant::StayFocusedOff => Self::StayFocusedOff,
            Self::Discriminant::Animation => Self::Animation(str.parse().unwrap_or_default()),
            Self::Discriminant::BorderColor => Self::BorderColor(str.parse().unwrap_or_default()),
            Self::Discriminant::IdleIngibit => Self::IdleIngibit(str.parse().unwrap_or_default()),
            Self::Discriminant::Opacity => Self::Opacity(str.parse().unwrap_or_default()),
            Self::Discriminant::Tag => {
                if let Some(stripped) = str.strip_prefix('+') {
                    Self::Tag(TagToggleState::Set, stripped.trim().to_string())
                } else if let Some(stripped) = str.strip_prefix('-') {
                    Self::Tag(TagToggleState::Unset, stripped.trim().to_string())
                } else {
                    Self::Tag(TagToggleState::Toggle, str.trim().to_string())
                }
            }
            Self::Discriminant::MaxSize => {
                let (width, height) = str.split_once(' ').unwrap_or((str, "0"));
                Self::MaxSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::MinSize => {
                let (width, height) = str.split_once(' ').unwrap_or((str, "0"));
                Self::MinSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::BorderSize => Self::BorderSize(str.parse().unwrap_or_default()),
            Self::Discriminant::Rounding => Self::Rounding(str.parse().unwrap_or_default()),
            Self::Discriminant::RoundingPower => {
                Self::RoundingPower(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AllowsInputOn => Self::AllowsInputOn,
            Self::Discriminant::AllowsInputOff => Self::AllowsInputOff,
            Self::Discriminant::DimAroundOn => Self::DimAroundOn,
            Self::Discriminant::DimAroundOff => Self::DimAroundOff,
            Self::Discriminant::DecorateOn => Self::DecorateOn,
            Self::Discriminant::DecorateOff => Self::DecorateOff,
            Self::Discriminant::FocusOnActivateOn => Self::FocusOnActivateOn,
            Self::Discriminant::FocusOnActivateOff => Self::FocusOnActivateOff,
            Self::Discriminant::KeepAspectRatioOn => Self::KeepAspectRatioOn,
            Self::Discriminant::KeepAspectRatioOff => Self::KeepAspectRatioOff,
            Self::Discriminant::NearestNeighborOn => Self::NearestNeighborOn,
            Self::Discriminant::NearestNeighborOff => Self::NearestNeighborOff,
            Self::Discriminant::NoAnimOn => Self::NoAnimOn,
            Self::Discriminant::NoAnimOff => Self::NoAnimOff,
            Self::Discriminant::NoBlurOn => Self::NoBlurOn,
            Self::Discriminant::NoBlurOff => Self::NoBlurOff,
            Self::Discriminant::NoDimOn => Self::NoDimOn,
            Self::Discriminant::NoDimOff => Self::NoDimOff,
            Self::Discriminant::NoFocusOn => Self::NoFocusOn,
            Self::Discriminant::NoFocusOff => Self::NoFocusOff,
            Self::Discriminant::NoFollowMouseOn => Self::NoFollowMouseOn,
            Self::Discriminant::NoFollowMouseOff => Self::NoFollowMouseOff,
            Self::Discriminant::NoShadowOn => Self::NoShadowOn,
            Self::Discriminant::NoShadowOff => Self::NoShadowOff,
            Self::Discriminant::NoShortcutsInhibitOn => Self::NoShortcutsInhibitOn,
            Self::Discriminant::NoShortcutsInhibitOff => Self::NoShortcutsInhibitOff,
            Self::Discriminant::NoScreenShareOn => Self::NoScreenShareOn,
            Self::Discriminant::NoScreenShareOff => Self::NoScreenShareOff,
            Self::Discriminant::NoVRROn => Self::NoVRROn,
            Self::Discriminant::NoVRROff => Self::NoVRROff,
            Self::Discriminant::OpaqueOn => Self::OpaqueOn,
            Self::Discriminant::OpaqueOff => Self::OpaqueOff,
            Self::Discriminant::ForceRGBXOn => Self::ForceRGBXOn,
            Self::Discriminant::ForceRGBXOff => Self::ForceRGBXOff,
            Self::Discriminant::SyncFullscreenOn => Self::SyncFullscreenOn,
            Self::Discriminant::SyncFullscreenOff => Self::SyncFullscreenOff,
            Self::Discriminant::ImmediateOn => Self::ImmediateOn,
            Self::Discriminant::ImmediateOff => Self::ImmediateOff,
            Self::Discriminant::XrayOn => Self::XrayOn,
            Self::Discriminant::XrayOff => Self::XrayOff,
            Self::Discriminant::RenderUnfocusedOn => Self::RenderUnfocusedOn,
            Self::Discriminant::RenderUnfocusedOff => Self::RenderUnfocusedOff,
            Self::Discriminant::ScrollMouse => Self::ScrollMouse(str.parse().unwrap_or_default()),
            Self::Discriminant::ScrollTouchpad => {
                Self::ScrollTouchpad(str.parse().unwrap_or_default())
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WindowRuleDynamicEffect::PersistentSizeOn => None,
            WindowRuleDynamicEffect::PersistentSizeOff => None,
            WindowRuleDynamicEffect::NoMaxSizeOn => None,
            WindowRuleDynamicEffect::NoMaxSizeOff => None,
            WindowRuleDynamicEffect::StayFocusedOn => None,
            WindowRuleDynamicEffect::StayFocusedOff => None,
            WindowRuleDynamicEffect::Animation(animation) => Some(animation.to_string()),
            WindowRuleDynamicEffect::BorderColor(border_color) => Some(border_color.to_string()),
            WindowRuleDynamicEffect::IdleIngibit(mode) => Some(mode.to_string()),
            WindowRuleDynamicEffect::Opacity(opacity) => Some(opacity.to_string()),
            WindowRuleDynamicEffect::Tag(TagToggleState::Set, tag) => Some(format!("+{}", tag)),
            WindowRuleDynamicEffect::Tag(TagToggleState::Unset, tag) => Some(format!("-{}", tag)),
            WindowRuleDynamicEffect::Tag(TagToggleState::Toggle, tag) => Some(tag.clone()),
            WindowRuleDynamicEffect::MaxSize(width, height) => {
                Some(format!("{} {}", width, height))
            }
            WindowRuleDynamicEffect::MinSize(width, height) => {
                Some(format!("{} {}", width, height))
            }
            WindowRuleDynamicEffect::BorderSize(size) => Some(size.to_string()),
            WindowRuleDynamicEffect::Rounding(size) => Some(size.to_string()),
            WindowRuleDynamicEffect::RoundingPower(power) => Some(power.to_string()),
            WindowRuleDynamicEffect::AllowsInputOn => None,
            WindowRuleDynamicEffect::AllowsInputOff => None,
            WindowRuleDynamicEffect::DimAroundOn => None,
            WindowRuleDynamicEffect::DimAroundOff => None,
            WindowRuleDynamicEffect::DecorateOn => None,
            WindowRuleDynamicEffect::DecorateOff => None,
            WindowRuleDynamicEffect::FocusOnActivateOn => None,
            WindowRuleDynamicEffect::FocusOnActivateOff => None,
            WindowRuleDynamicEffect::KeepAspectRatioOn => None,
            WindowRuleDynamicEffect::KeepAspectRatioOff => None,
            WindowRuleDynamicEffect::NearestNeighborOn => None,
            WindowRuleDynamicEffect::NearestNeighborOff => None,
            WindowRuleDynamicEffect::NoAnimOn => None,
            WindowRuleDynamicEffect::NoAnimOff => None,
            WindowRuleDynamicEffect::NoBlurOn => None,
            WindowRuleDynamicEffect::NoBlurOff => None,
            WindowRuleDynamicEffect::NoDimOn => None,
            WindowRuleDynamicEffect::NoDimOff => None,
            WindowRuleDynamicEffect::NoFocusOn => None,
            WindowRuleDynamicEffect::NoFocusOff => None,
            WindowRuleDynamicEffect::NoFollowMouseOn => None,
            WindowRuleDynamicEffect::NoFollowMouseOff => None,
            WindowRuleDynamicEffect::NoShadowOn => None,
            WindowRuleDynamicEffect::NoShadowOff => None,
            WindowRuleDynamicEffect::NoShortcutsInhibitOn => None,
            WindowRuleDynamicEffect::NoShortcutsInhibitOff => None,
            WindowRuleDynamicEffect::NoScreenShareOn => None,
            WindowRuleDynamicEffect::NoScreenShareOff => None,
            WindowRuleDynamicEffect::NoVRROn => None,
            WindowRuleDynamicEffect::NoVRROff => None,
            WindowRuleDynamicEffect::OpaqueOn => None,
            WindowRuleDynamicEffect::OpaqueOff => None,
            WindowRuleDynamicEffect::ForceRGBXOn => None,
            WindowRuleDynamicEffect::ForceRGBXOff => None,
            WindowRuleDynamicEffect::SyncFullscreenOn => None,
            WindowRuleDynamicEffect::SyncFullscreenOff => None,
            WindowRuleDynamicEffect::ImmediateOn => None,
            WindowRuleDynamicEffect::ImmediateOff => None,
            WindowRuleDynamicEffect::XrayOn => None,
            WindowRuleDynamicEffect::XrayOff => None,
            WindowRuleDynamicEffect::RenderUnfocusedOn => None,
            WindowRuleDynamicEffect::RenderUnfocusedOff => None,
            WindowRuleDynamicEffect::ScrollMouse(speed) => Some(speed.to_string()),
            WindowRuleDynamicEffect::ScrollTouchpad(speed) => Some(speed.to_string()),
        }
    }

    fn custom_split(discriminant: Self::Discriminant) -> Option<fn(&str) -> Vec<&str>> {
        match discriminant {
            Self::Discriminant::Tag => Some(|s| {
                if let Some(stripped) = s.strip_prefix("+") {
                    vec!["+", stripped]
                } else if let Some(stripped) = s.strip_prefix("-") {
                    vec!["-", stripped]
                } else {
                    vec!["", s]
                }
            }),
            _ => None,
        }
    }
}

impl Default for WindowRuleDynamicEffect {
    fn default() -> Self {
        WindowRuleDynamicEffect::Animation(AnimationStyle::default())
    }
}

impl FromStr for WindowRuleDynamicEffect {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        let parts = s.split_whitespace().collect::<Vec<_>>();

        match parts[0].to_lowercase().as_str() {
            "persistentsize" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::PersistentSizeOn),
                Some(false) => Ok(WindowRuleDynamicEffect::PersistentSizeOff),
                None => Ok(WindowRuleDynamicEffect::PersistentSizeOff),
            },
            "nomaxsize" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoMaxSizeOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoMaxSizeOff),
                None => Ok(WindowRuleDynamicEffect::NoMaxSizeOff),
            },
            "stayfocused" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::StayFocusedOn),
                Some(false) => Ok(WindowRuleDynamicEffect::StayFocusedOff),
                None => Ok(WindowRuleDynamicEffect::StayFocusedOff),
            },
            "animation" => Ok(WindowRuleDynamicEffect::Animation(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "bordercolor" => Ok(WindowRuleDynamicEffect::BorderColor(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "idleingibit" => Ok(WindowRuleDynamicEffect::IdleIngibit(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "opacity" => Ok(WindowRuleDynamicEffect::Opacity(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "tag" => {
                let part2 = parts.get(1).unwrap_or(&"");
                if let Some(stripped) = part2.strip_prefix("+") {
                    Ok(WindowRuleDynamicEffect::Tag(
                        TagToggleState::Set,
                        stripped.trim().to_string(),
                    ))
                } else if let Some(stripped) = part2.strip_prefix("-") {
                    Ok(WindowRuleDynamicEffect::Tag(
                        TagToggleState::Unset,
                        stripped.trim().to_string(),
                    ))
                } else {
                    Ok(WindowRuleDynamicEffect::Tag(
                        TagToggleState::Toggle,
                        part2.trim().to_string(),
                    ))
                }
            }
            "maxsize" => {
                let width = parts.get(1).unwrap_or(&"");
                let height = parts.get(2).unwrap_or(&"");
                Ok(WindowRuleDynamicEffect::MaxSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            "minsize" => {
                let width = parts.get(1).unwrap_or(&"");
                let height = parts.get(2).unwrap_or(&"");
                Ok(WindowRuleDynamicEffect::MinSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            "bordersize" => Ok(WindowRuleDynamicEffect::BorderSize(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "rounding" => Ok(WindowRuleDynamicEffect::Rounding(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "roundingpower" => Ok(WindowRuleDynamicEffect::RoundingPower(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "allowsinput" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::AllowsInputOn),
                Some(false) => Ok(WindowRuleDynamicEffect::AllowsInputOff),
                None => Ok(WindowRuleDynamicEffect::AllowsInputOff),
            },
            "dimaround" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::DimAroundOn),
                Some(false) => Ok(WindowRuleDynamicEffect::DimAroundOff),
                None => Ok(WindowRuleDynamicEffect::DimAroundOff),
            },
            "decorate" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::DecorateOn),
                Some(false) => Ok(WindowRuleDynamicEffect::DecorateOff),
                None => Ok(WindowRuleDynamicEffect::DecorateOff),
            },
            "focusonactivate" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::FocusOnActivateOn),
                Some(false) => Ok(WindowRuleDynamicEffect::FocusOnActivateOff),
                None => Ok(WindowRuleDynamicEffect::FocusOnActivateOff),
            },
            "keepaspectratio" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::KeepAspectRatioOn),
                Some(false) => Ok(WindowRuleDynamicEffect::KeepAspectRatioOff),
                None => Ok(WindowRuleDynamicEffect::KeepAspectRatioOff),
            },
            "nearestneighbor" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NearestNeighborOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NearestNeighborOff),
                None => Ok(WindowRuleDynamicEffect::NearestNeighborOff),
            },
            "noanim" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoAnimOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoAnimOff),
                None => Ok(WindowRuleDynamicEffect::NoAnimOff),
            },
            "noblur" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoBlurOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoBlurOff),
                None => Ok(WindowRuleDynamicEffect::NoBlurOff),
            },
            "nodim" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoDimOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoDimOff),
                None => Ok(WindowRuleDynamicEffect::NoDimOff),
            },
            "nofocus" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoFocusOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoFocusOff),
                None => Ok(WindowRuleDynamicEffect::NoFocusOff),
            },
            "nofollowmouse" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoFollowMouseOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoFollowMouseOff),
                None => Ok(WindowRuleDynamicEffect::NoFollowMouseOff),
            },
            "noshadow" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoShadowOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoShadowOff),
                None => Ok(WindowRuleDynamicEffect::NoShadowOff),
            },
            "noshortcutsinhibit" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoShortcutsInhibitOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoShortcutsInhibitOff),
                None => Ok(WindowRuleDynamicEffect::NoShortcutsInhibitOff),
            },
            "noscreenshare" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoScreenShareOn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoScreenShareOff),
                None => Ok(WindowRuleDynamicEffect::NoScreenShareOff),
            },
            "novrr" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::NoVRROn),
                Some(false) => Ok(WindowRuleDynamicEffect::NoVRROff),
                None => Ok(WindowRuleDynamicEffect::NoVRROff),
            },
            "opaque" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::OpaqueOn),
                Some(false) => Ok(WindowRuleDynamicEffect::OpaqueOff),
                None => Ok(WindowRuleDynamicEffect::OpaqueOff),
            },
            "forcergbx" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::ForceRGBXOn),
                Some(false) => Ok(WindowRuleDynamicEffect::ForceRGBXOff),
                None => Ok(WindowRuleDynamicEffect::ForceRGBXOff),
            },
            "syncfullscreen" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::SyncFullscreenOn),
                Some(false) => Ok(WindowRuleDynamicEffect::SyncFullscreenOff),
                None => Ok(WindowRuleDynamicEffect::SyncFullscreenOff),
            },
            "immediate" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::ImmediateOn),
                Some(false) => Ok(WindowRuleDynamicEffect::ImmediateOff),
                None => Ok(WindowRuleDynamicEffect::ImmediateOff),
            },
            "xray" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::XrayOn),
                Some(false) => Ok(WindowRuleDynamicEffect::XrayOff),
                None => Ok(WindowRuleDynamicEffect::XrayOff),
            },
            "renderunfocused" => match parse_bool(parts.get(1).unwrap_or(&"")) {
                Some(true) => Ok(WindowRuleDynamicEffect::RenderUnfocusedOn),
                Some(false) => Ok(WindowRuleDynamicEffect::RenderUnfocusedOff),
                None => Ok(WindowRuleDynamicEffect::RenderUnfocusedOff),
            },
            "scrollmouse" => Ok(WindowRuleDynamicEffect::ScrollMouse(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "scrolltouchpad" => Ok(WindowRuleDynamicEffect::ScrollTouchpad(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            _ => Err(()),
        }
    }
}

impl Display for WindowRuleDynamicEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRuleDynamicEffect::PersistentSizeOn => write!(f, "persistentsize on"),
            WindowRuleDynamicEffect::PersistentSizeOff => write!(f, "persistentsize off"),
            WindowRuleDynamicEffect::NoMaxSizeOn => write!(f, "nomaxsize on"),
            WindowRuleDynamicEffect::NoMaxSizeOff => write!(f, "nomaxsize off"),
            WindowRuleDynamicEffect::StayFocusedOn => write!(f, "stayfocused on"),
            WindowRuleDynamicEffect::StayFocusedOff => write!(f, "stayfocused off"),
            WindowRuleDynamicEffect::Animation(animation) => write!(f, "animation {}", animation),
            WindowRuleDynamicEffect::BorderColor(border_color) => {
                write!(f, "bordercolor {}", border_color)
            }
            WindowRuleDynamicEffect::IdleIngibit(mode) => write!(f, "idleingibit {}", mode),
            WindowRuleDynamicEffect::Opacity(opacity) => write!(f, "opacity {}", opacity),
            WindowRuleDynamicEffect::Tag(TagToggleState::Set, tag) => write!(f, "tag +{}", tag),
            WindowRuleDynamicEffect::Tag(TagToggleState::Unset, tag) => write!(f, "tag -{}", tag),
            WindowRuleDynamicEffect::Tag(TagToggleState::Toggle, tag) => write!(f, "tag {}", tag),
            WindowRuleDynamicEffect::MaxSize(width, height) => {
                write!(f, "maxsize {} {}", width, height)
            }
            WindowRuleDynamicEffect::MinSize(width, height) => {
                write!(f, "minsize {} {}", width, height)
            }
            WindowRuleDynamicEffect::BorderSize(size) => write!(f, "bordersize {}", size),
            WindowRuleDynamicEffect::Rounding(size) => write!(f, "rounding {}", size),
            WindowRuleDynamicEffect::RoundingPower(power) => write!(f, "roundingpower {}", power),
            WindowRuleDynamicEffect::AllowsInputOn => write!(f, "allowsinput on"),
            WindowRuleDynamicEffect::AllowsInputOff => write!(f, "allowsinput off"),
            WindowRuleDynamicEffect::DimAroundOn => write!(f, "dimaround on"),
            WindowRuleDynamicEffect::DimAroundOff => write!(f, "dimaround off"),
            WindowRuleDynamicEffect::DecorateOn => write!(f, "decorate on"),
            WindowRuleDynamicEffect::DecorateOff => write!(f, "decorate off"),
            WindowRuleDynamicEffect::FocusOnActivateOn => write!(f, "focusonactivate on"),
            WindowRuleDynamicEffect::FocusOnActivateOff => write!(f, "focusonactivate off"),
            WindowRuleDynamicEffect::KeepAspectRatioOn => write!(f, "keepaspectratio on"),
            WindowRuleDynamicEffect::KeepAspectRatioOff => write!(f, "keepaspectratio off"),
            WindowRuleDynamicEffect::NearestNeighborOn => write!(f, "nearestneighbor on"),
            WindowRuleDynamicEffect::NearestNeighborOff => write!(f, "nearestneighbor off"),
            WindowRuleDynamicEffect::NoAnimOn => write!(f, "noanim on"),
            WindowRuleDynamicEffect::NoAnimOff => write!(f, "noanim off"),
            WindowRuleDynamicEffect::NoBlurOn => write!(f, "noblur on"),
            WindowRuleDynamicEffect::NoBlurOff => write!(f, "noblur off"),
            WindowRuleDynamicEffect::NoDimOn => write!(f, "nodim on"),
            WindowRuleDynamicEffect::NoDimOff => write!(f, "nodim off"),
            WindowRuleDynamicEffect::NoFocusOn => write!(f, "nofocus on"),
            WindowRuleDynamicEffect::NoFocusOff => write!(f, "nofocus off"),
            WindowRuleDynamicEffect::NoFollowMouseOn => write!(f, "nofollowmouse on"),
            WindowRuleDynamicEffect::NoFollowMouseOff => write!(f, "nofollowmouse off"),
            WindowRuleDynamicEffect::NoShadowOn => write!(f, "noshadow on"),
            WindowRuleDynamicEffect::NoShadowOff => write!(f, "noshadow off"),
            WindowRuleDynamicEffect::NoShortcutsInhibitOn => write!(f, "noshortcutsinhibit on"),
            WindowRuleDynamicEffect::NoShortcutsInhibitOff => write!(f, "noshortcutsinhibit off"),
            WindowRuleDynamicEffect::NoScreenShareOn => write!(f, "noscreenshare on"),
            WindowRuleDynamicEffect::NoScreenShareOff => write!(f, "noscreenshare off"),
            WindowRuleDynamicEffect::NoVRROn => write!(f, "novrr on"),
            WindowRuleDynamicEffect::NoVRROff => write!(f, "novrr off"),
            WindowRuleDynamicEffect::OpaqueOn => write!(f, "opaque on"),
            WindowRuleDynamicEffect::OpaqueOff => write!(f, "opaque off"),
            WindowRuleDynamicEffect::ForceRGBXOn => write!(f, "forcergbx on"),
            WindowRuleDynamicEffect::ForceRGBXOff => write!(f, "forcergbx off"),
            WindowRuleDynamicEffect::SyncFullscreenOn => write!(f, "syncfullscreen on"),
            WindowRuleDynamicEffect::SyncFullscreenOff => write!(f, "syncfullscreen off"),
            WindowRuleDynamicEffect::ImmediateOn => write!(f, "immediate on"),
            WindowRuleDynamicEffect::ImmediateOff => write!(f, "immediate off"),
            WindowRuleDynamicEffect::XrayOn => write!(f, "xray on"),
            WindowRuleDynamicEffect::XrayOff => write!(f, "xray off"),
            WindowRuleDynamicEffect::RenderUnfocusedOn => write!(f, "renderunfocused on"),
            WindowRuleDynamicEffect::RenderUnfocusedOff => write!(f, "renderunfocused off"),
            WindowRuleDynamicEffect::ScrollMouse(speed) => write!(f, "scrollmouse {}", speed),
            WindowRuleDynamicEffect::ScrollTouchpad(speed) => write!(f, "scrolltouchpad {}", speed),
        }
    }
}

impl EnumConfigForGtk for WindowRuleDynamicEffect {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.set_prop.persistent_size"),
            &t!("hyprland.set_prop.no_max_size"),
            &t!("hyprland.set_prop.stay_focused"),
            &t!("hyprland.set_prop.animation"),
            &t!("hyprland.set_prop.border_color"),
            &t!("hyprland.set_prop.idle_ingibit"),
            &t!("hyprland.set_prop.opacity"),
            &t!("hyprland.set_prop.tag"),
            &t!("hyprland.set_prop.max_size"),
            &t!("hyprland.set_prop.min_size"),
            &t!("hyprland.set_prop.border_size"),
            &t!("hyprland.set_prop.rounding"),
            &t!("hyprland.set_prop.rounding_power"),
            &t!("hyprland.set_prop.allows_input"),
            &t!("hyprland.set_prop.dim_around"),
            &t!("hyprland.set_prop.decorate"),
            &t!("hyprland.set_prop.focus_on_activate"),
            &t!("hyprland.set_prop.keep_aspect_ratio"),
            &t!("hyprland.set_prop.nearest_neighbor"),
            &t!("hyprland.set_prop.no_anim"),
            &t!("hyprland.set_prop.no_blur"),
            &t!("hyprland.set_prop.no_dim"),
            &t!("hyprland.set_prop.no_focus"),
            &t!("hyprland.set_prop.no_follow_mouse"),
            &t!("hyprland.set_prop.no_shadow"),
            &t!("hyprland.set_prop.no_shortcuts_inhibit"),
            &t!("hyprland.set_prop.no_screen_share"),
            &t!("hyprland.set_prop.no_vrr"),
            &t!("hyprland.set_prop.opaque"),
            &t!("hyprland.set_prop.force_rgbx"),
            &t!("hyprland.set_prop.sync_fullscreen"),
            &t!("hyprland.set_prop.immediate"),
            &t!("hyprland.set_prop.xray"),
            &t!("hyprland.set_prop.render_unfocused"),
            &t!("hyprland.set_prop.scroll_mouse"),
            &t!("hyprland.set_prop.scroll_touchpad"),
        ])
    }

    const SEPARATOR: Option<char> = Some(' ');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            WindowRuleDynamicEffect::PersistentSizeOn
            | WindowRuleDynamicEffect::PersistentSizeOff => None,
            WindowRuleDynamicEffect::NoMaxSizeOn | WindowRuleDynamicEffect::NoMaxSizeOff => None,
            WindowRuleDynamicEffect::StayFocusedOn | WindowRuleDynamicEffect::StayFocusedOff => {
                None
            }
            WindowRuleDynamicEffect::Animation(_style) => Some(<(AnimationStyle,)>::to_gtk_box),
            WindowRuleDynamicEffect::BorderColor(_color) => Some(<(BorderColor,)>::to_gtk_box),
            WindowRuleDynamicEffect::IdleIngibit(_mode) => Some(<(IdleIngibitMode,)>::to_gtk_box),
            WindowRuleDynamicEffect::Opacity(_opacity) => Some(<(HyprOpacity,)>::to_gtk_box),
            WindowRuleDynamicEffect::Tag(_toggle_state, _tag) => {
                Some(|entry, _, names, custom_split| {
                    <(TagToggleState, String)>::to_gtk_box(
                        entry,
                        PLUG_SEPARATOR,
                        names,
                        custom_split,
                    )
                })
            }
            WindowRuleDynamicEffect::MaxSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            WindowRuleDynamicEffect::MinSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            WindowRuleDynamicEffect::BorderSize(_size) => Some(<(u32,)>::to_gtk_box),
            WindowRuleDynamicEffect::Rounding(_size) => Some(<(u32,)>::to_gtk_box),
            WindowRuleDynamicEffect::RoundingPower(_power) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            WindowRuleDynamicEffect::AllowsInputOn | WindowRuleDynamicEffect::AllowsInputOff => {
                None
            }
            WindowRuleDynamicEffect::DimAroundOn | WindowRuleDynamicEffect::DimAroundOff => None,
            WindowRuleDynamicEffect::DecorateOn | WindowRuleDynamicEffect::DecorateOff => None,
            WindowRuleDynamicEffect::FocusOnActivateOn
            | WindowRuleDynamicEffect::FocusOnActivateOff => None,
            WindowRuleDynamicEffect::KeepAspectRatioOn
            | WindowRuleDynamicEffect::KeepAspectRatioOff => None,
            WindowRuleDynamicEffect::NearestNeighborOn
            | WindowRuleDynamicEffect::NearestNeighborOff => None,
            WindowRuleDynamicEffect::NoAnimOn | WindowRuleDynamicEffect::NoAnimOff => None,
            WindowRuleDynamicEffect::NoBlurOn | WindowRuleDynamicEffect::NoBlurOff => None,
            WindowRuleDynamicEffect::NoDimOn | WindowRuleDynamicEffect::NoDimOff => None,
            WindowRuleDynamicEffect::NoFocusOn | WindowRuleDynamicEffect::NoFocusOff => None,
            WindowRuleDynamicEffect::NoFollowMouseOn
            | WindowRuleDynamicEffect::NoFollowMouseOff => None,
            WindowRuleDynamicEffect::NoShadowOn | WindowRuleDynamicEffect::NoShadowOff => None,
            WindowRuleDynamicEffect::NoShortcutsInhibitOn
            | WindowRuleDynamicEffect::NoShortcutsInhibitOff => None,
            WindowRuleDynamicEffect::NoScreenShareOn
            | WindowRuleDynamicEffect::NoScreenShareOff => None,
            WindowRuleDynamicEffect::NoVRROn | WindowRuleDynamicEffect::NoVRROff => None,
            WindowRuleDynamicEffect::OpaqueOn | WindowRuleDynamicEffect::OpaqueOff => None,
            WindowRuleDynamicEffect::ForceRGBXOn | WindowRuleDynamicEffect::ForceRGBXOff => None,
            WindowRuleDynamicEffect::SyncFullscreenOn
            | WindowRuleDynamicEffect::SyncFullscreenOff => None,
            WindowRuleDynamicEffect::ImmediateOn | WindowRuleDynamicEffect::ImmediateOff => None,
            WindowRuleDynamicEffect::XrayOn | WindowRuleDynamicEffect::XrayOff => None,
            WindowRuleDynamicEffect::RenderUnfocusedOn
            | WindowRuleDynamicEffect::RenderUnfocusedOff => None,
            WindowRuleDynamicEffect::ScrollMouse(_scroll_factor) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            WindowRuleDynamicEffect::ScrollTouchpad(_scroll_factor) => {
                Some(|entry, _, names, _| {
                    create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                        entry,
                        names.first().unwrap_or(&FieldLabel::Unnamed),
                    )
                })
            }
        }
    }
}

register_togtkbox!(WindowRuleDynamicEffect);
register_togtkbox_with_separator_names!(
    (BorderColor,),
    (IdleIngibitMode,),
    (HyprOpacity,),
    (TagToggleState, String),
);
