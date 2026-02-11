use super::{
    AnimationStyle, BorderColor, HyprGradient, HyprOpacity, IdleIngibitMode, SetPropToggleState,
    TagToggleState,
};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBox, ToGtkBoxWithSeparatorAndNames,
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
pub enum SetProp {
    Alpha(f64),
    AlphaOverride(bool),
    AlphaInactive(f64),
    AlphaInactiveOverride(bool),
    AlphaFullscreen(f64),
    AlphaFullscreenOverride(bool),
    AnimationStyle(String),
    ActiveBorderColor(Option<HyprGradient>),
    InactiveBorderColor(Option<HyprGradient>),
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
    AllowsInput(SetPropToggleState),
    DimAround(SetPropToggleState),
    Decorate(SetPropToggleState),
    FocusOnActivate(SetPropToggleState),
    KeepAspectRatio(SetPropToggleState),
    NearestNeighbor(SetPropToggleState),
    NoAnim(SetPropToggleState),
    NoBlur(SetPropToggleState),
    NoBorder(SetPropToggleState),
    NoDim(SetPropToggleState),
    NoFocus(SetPropToggleState),
    NoFollowMouse(SetPropToggleState),
    NoMaxSize(SetPropToggleState),
    NoRounding(SetPropToggleState),
    NoShadow(SetPropToggleState),
    NoShortcutsInhibit(SetPropToggleState),
    Opaque(SetPropToggleState),
    ForceRGBX(SetPropToggleState),
    SyncFullscreen(SetPropToggleState),
    Immediate(SetPropToggleState),
    Xray(SetPropToggleState),
    RenderUnfocused,
    ScrollMouse(f64),
    ScrollTouchpad(f64),
    NoScreenShare(SetPropToggleState),
    NoVRR(SetPropToggleState),
}

impl HasDiscriminant for SetProp {
    type Discriminant = SetPropDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Alpha => Self::Alpha(1.0),
            Self::Discriminant::AlphaOverride => Self::AlphaOverride(false),
            Self::Discriminant::AlphaInactive => Self::AlphaInactive(1.0),
            Self::Discriminant::AlphaInactiveOverride => Self::AlphaInactiveOverride(false),
            Self::Discriminant::AlphaFullscreen => Self::AlphaFullscreen(1.0),
            Self::Discriminant::AlphaFullscreenOverride => Self::AlphaFullscreenOverride(false),
            Self::Discriminant::AnimationStyle => Self::AnimationStyle("".to_string()),
            Self::Discriminant::ActiveBorderColor => Self::ActiveBorderColor(None),
            Self::Discriminant::InactiveBorderColor => Self::InactiveBorderColor(None),
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
            Self::Discriminant::AllowsInput => Self::AllowsInput(SetPropToggleState::default()),
            Self::Discriminant::DimAround => Self::DimAround(SetPropToggleState::default()),
            Self::Discriminant::Decorate => Self::Decorate(SetPropToggleState::default()),
            Self::Discriminant::FocusOnActivate => {
                Self::FocusOnActivate(SetPropToggleState::default())
            }
            Self::Discriminant::KeepAspectRatio => {
                Self::KeepAspectRatio(SetPropToggleState::default())
            }
            Self::Discriminant::NearestNeighbor => {
                Self::NearestNeighbor(SetPropToggleState::default())
            }
            Self::Discriminant::NoAnim => Self::NoAnim(SetPropToggleState::default()),
            Self::Discriminant::NoBlur => Self::NoBlur(SetPropToggleState::default()),
            Self::Discriminant::NoBorder => Self::NoBorder(SetPropToggleState::default()),
            Self::Discriminant::NoDim => Self::NoDim(SetPropToggleState::default()),
            Self::Discriminant::NoFocus => Self::NoFocus(SetPropToggleState::default()),
            Self::Discriminant::NoFollowMouse => Self::NoFollowMouse(SetPropToggleState::default()),
            Self::Discriminant::NoMaxSize => Self::NoMaxSize(SetPropToggleState::default()),
            Self::Discriminant::NoRounding => Self::NoRounding(SetPropToggleState::default()),
            Self::Discriminant::NoShadow => Self::NoShadow(SetPropToggleState::default()),
            Self::Discriminant::NoShortcutsInhibit => {
                Self::NoShortcutsInhibit(SetPropToggleState::default())
            }
            Self::Discriminant::Opaque => Self::Opaque(SetPropToggleState::default()),
            Self::Discriminant::ForceRGBX => Self::ForceRGBX(SetPropToggleState::default()),
            Self::Discriminant::SyncFullscreen => {
                Self::SyncFullscreen(SetPropToggleState::default())
            }
            Self::Discriminant::Immediate => Self::Immediate(SetPropToggleState::default()),
            Self::Discriminant::Xray => Self::Xray(SetPropToggleState::default()),
            Self::Discriminant::RenderUnfocused => Self::RenderUnfocused,
            Self::Discriminant::ScrollMouse => Self::ScrollMouse(0.0),
            Self::Discriminant::ScrollTouchpad => Self::ScrollTouchpad(0.0),
            Self::Discriminant::NoScreenShare => Self::NoScreenShare(SetPropToggleState::default()),
            Self::Discriminant::NoVRR => Self::NoVRR(SetPropToggleState::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Alpha => Self::Alpha(str.parse().unwrap_or_default()),
            Self::Discriminant::AlphaOverride => {
                Self::AlphaOverride(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AlphaInactive => {
                Self::AlphaInactive(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AlphaInactiveOverride => {
                Self::AlphaInactiveOverride(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AlphaFullscreen => {
                Self::AlphaFullscreen(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AlphaFullscreenOverride => {
                Self::AlphaFullscreenOverride(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AnimationStyle => Self::AnimationStyle(str.to_string()),
            Self::Discriminant::ActiveBorderColor => {
                if str == "-1" {
                    Self::ActiveBorderColor(None)
                } else {
                    Self::ActiveBorderColor(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::InactiveBorderColor => {
                if str == "-1" {
                    Self::InactiveBorderColor(None)
                } else {
                    Self::InactiveBorderColor(Some(str.parse().unwrap_or_default()))
                }
            }
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
            Self::Discriminant::AllowsInput => Self::AllowsInput(str.parse().unwrap_or_default()),
            Self::Discriminant::DimAround => Self::DimAround(str.parse().unwrap_or_default()),
            Self::Discriminant::Decorate => Self::Decorate(str.parse().unwrap_or_default()),
            Self::Discriminant::FocusOnActivate => {
                Self::FocusOnActivate(str.parse().unwrap_or_default())
            }
            Self::Discriminant::KeepAspectRatio => {
                Self::KeepAspectRatio(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NearestNeighbor => {
                Self::NearestNeighbor(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NoAnim => Self::NoAnim(str.parse().unwrap_or_default()),
            Self::Discriminant::NoBlur => Self::NoBlur(str.parse().unwrap_or_default()),
            Self::Discriminant::NoBorder => Self::NoBorder(str.parse().unwrap_or_default()),
            Self::Discriminant::NoDim => Self::NoDim(str.parse().unwrap_or_default()),
            Self::Discriminant::NoFocus => Self::NoFocus(str.parse().unwrap_or_default()),
            Self::Discriminant::NoFollowMouse => {
                Self::NoFollowMouse(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NoMaxSize => Self::NoMaxSize(str.parse().unwrap_or_default()),
            Self::Discriminant::NoRounding => Self::NoRounding(str.parse().unwrap_or_default()),
            Self::Discriminant::NoShadow => Self::NoShadow(str.parse().unwrap_or_default()),
            Self::Discriminant::NoShortcutsInhibit => {
                Self::NoShortcutsInhibit(str.parse().unwrap_or_default())
            }
            Self::Discriminant::Opaque => Self::Opaque(str.parse().unwrap_or_default()),
            Self::Discriminant::ForceRGBX => Self::ForceRGBX(str.parse().unwrap_or_default()),
            Self::Discriminant::SyncFullscreen => {
                Self::SyncFullscreen(str.parse().unwrap_or_default())
            }
            Self::Discriminant::Immediate => Self::Immediate(str.parse().unwrap_or_default()),
            Self::Discriminant::Xray => Self::Xray(str.parse().unwrap_or_default()),
            Self::Discriminant::RenderUnfocused => Self::RenderUnfocused,
            Self::Discriminant::ScrollMouse => Self::ScrollMouse(str.parse().unwrap_or_default()),
            Self::Discriminant::ScrollTouchpad => {
                Self::ScrollTouchpad(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NoScreenShare => {
                Self::NoScreenShare(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NoVRR => Self::NoVRR(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            SetProp::Alpha(alpha) => Some(alpha.to_string()),
            SetProp::AlphaOverride(value) => Some(value.to_string()),
            SetProp::AlphaInactive(alpha) => Some(alpha.to_string()),
            SetProp::AlphaInactiveOverride(value) => Some(value.to_string()),
            SetProp::AlphaFullscreen(alpha) => Some(alpha.to_string()),
            SetProp::AlphaFullscreenOverride(value) => Some(value.to_string()),
            SetProp::AnimationStyle(style) => Some(style.clone()),
            SetProp::ActiveBorderColor(None) => Some("-1".to_string()),
            SetProp::ActiveBorderColor(Some(color)) => Some(color.to_string()),
            SetProp::InactiveBorderColor(None) => Some("-1".to_string()),
            SetProp::InactiveBorderColor(Some(color)) => Some(color.to_string()),
            SetProp::Animation(animation) => Some(animation.to_string()),
            SetProp::BorderColor(border_color) => Some(border_color.to_string()),
            SetProp::IdleIngibit(mode) => Some(mode.to_string()),
            SetProp::Opacity(opacity) => Some(opacity.to_string()),
            SetProp::Tag(TagToggleState::Set, tag) => Some(format!("+{}", tag)),
            SetProp::Tag(TagToggleState::Unset, tag) => Some(format!("-{}", tag)),
            SetProp::Tag(TagToggleState::Toggle, tag) => Some(tag.clone()),
            SetProp::MaxSize(width, height) => Some(format!("{} {}", width, height)),
            SetProp::MinSize(width, height) => Some(format!("{} {}", width, height)),
            SetProp::BorderSize(size) => Some(size.to_string()),
            SetProp::Rounding(size) => Some(size.to_string()),
            SetProp::RoundingPower(power) => Some(power.to_string()),
            SetProp::AllowsInput(mode) => Some(mode.to_string()),
            SetProp::DimAround(mode) => Some(mode.to_string()),
            SetProp::Decorate(mode) => Some(mode.to_string()),
            SetProp::FocusOnActivate(mode) => Some(mode.to_string()),
            SetProp::KeepAspectRatio(mode) => Some(mode.to_string()),
            SetProp::NearestNeighbor(mode) => Some(mode.to_string()),
            SetProp::NoAnim(mode) => Some(mode.to_string()),
            SetProp::NoBlur(mode) => Some(mode.to_string()),
            SetProp::NoBorder(mode) => Some(mode.to_string()),
            SetProp::NoDim(mode) => Some(mode.to_string()),
            SetProp::NoFocus(mode) => Some(mode.to_string()),
            SetProp::NoFollowMouse(mode) => Some(mode.to_string()),
            SetProp::NoMaxSize(mode) => Some(mode.to_string()),
            SetProp::NoRounding(mode) => Some(mode.to_string()),
            SetProp::NoShadow(mode) => Some(mode.to_string()),
            SetProp::NoShortcutsInhibit(mode) => Some(mode.to_string()),
            SetProp::Opaque(mode) => Some(mode.to_string()),
            SetProp::ForceRGBX(mode) => Some(mode.to_string()),
            SetProp::SyncFullscreen(mode) => Some(mode.to_string()),
            SetProp::Immediate(mode) => Some(mode.to_string()),
            SetProp::Xray(mode) => Some(mode.to_string()),
            SetProp::RenderUnfocused => None,
            SetProp::ScrollMouse(speed) => Some(speed.to_string()),
            SetProp::ScrollTouchpad(speed) => Some(speed.to_string()),
            SetProp::NoScreenShare(mode) => Some(mode.to_string()),
            SetProp::NoVRR(mode) => Some(mode.to_string()),
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

impl Default for SetProp {
    fn default() -> Self {
        SetProp::Alpha(1.0)
    }
}

impl FromStr for SetProp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        let parts = s.split_whitespace().collect::<Vec<_>>();

        match parts[0] {
            "alpha" => {
                let alpha = parts.get(1).unwrap_or(&"").parse().unwrap_or(1.0);
                Ok(SetProp::Alpha(alpha))
            }
            "alphaoverride" => Ok(SetProp::AlphaOverride(
                parse_bool(&parts.get(1).unwrap_or(&"").to_lowercase()).unwrap_or(false),
            )),
            "alphainactive" => {
                let alpha = parts.get(1).unwrap_or(&"").parse().unwrap_or(1.0);
                Ok(SetProp::AlphaInactive(alpha))
            }
            "alphainactiveoverride" => Ok(SetProp::AlphaInactiveOverride(
                parse_bool(&parts.get(1).unwrap_or(&"").to_lowercase()).unwrap_or(false),
            )),
            "alphafullscreen" => {
                let alpha = parts.get(1).unwrap_or(&"").parse().unwrap_or(1.0);
                Ok(SetProp::AlphaFullscreen(alpha))
            }
            "alphafullscreenoverride" => Ok(SetProp::AlphaFullscreenOverride(
                parse_bool(&parts.get(1).unwrap_or(&"").to_lowercase()).unwrap_or(false),
            )),
            "animationstyle" => Ok(SetProp::AnimationStyle(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "activebordercolor" => {
                if parts.get(1) == Some(&"-1") {
                    Ok(SetProp::ActiveBorderColor(None))
                } else {
                    Ok(SetProp::ActiveBorderColor(Some(
                        parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
                    )))
                }
            }
            "inactivebordercolor" => {
                if parts.get(1) == Some(&"-1") {
                    Ok(SetProp::InactiveBorderColor(None))
                } else {
                    Ok(SetProp::InactiveBorderColor(Some(
                        parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
                    )))
                }
            }
            "animation" => Ok(SetProp::Animation(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "bordercolor" => Ok(SetProp::BorderColor(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "idleingibit" => Ok(SetProp::IdleIngibit(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "opacity" => Ok(SetProp::Opacity(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "tag" => {
                let part2 = parts.get(1).unwrap_or(&"");
                if let Some(stripped) = part2.strip_prefix("+") {
                    Ok(SetProp::Tag(
                        TagToggleState::Set,
                        stripped.trim().to_string(),
                    ))
                } else if let Some(stripped) = part2.strip_prefix("-") {
                    Ok(SetProp::Tag(
                        TagToggleState::Unset,
                        stripped.trim().to_string(),
                    ))
                } else {
                    Ok(SetProp::Tag(
                        TagToggleState::Toggle,
                        part2.trim().to_string(),
                    ))
                }
            }
            "maxsize" => {
                let width = parts.get(1).unwrap_or(&"");
                let height = parts.get(2).unwrap_or(&"");
                Ok(SetProp::MaxSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            "minsize" => {
                let width = parts.get(1).unwrap_or(&"");
                let height = parts.get(2).unwrap_or(&"");
                Ok(SetProp::MinSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            "bordersize" => Ok(SetProp::BorderSize(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "rounding" => Ok(SetProp::Rounding(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "roundingpower" => Ok(SetProp::RoundingPower(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "allowsinput" => Ok(SetProp::AllowsInput(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "dimaround" => Ok(SetProp::DimAround(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "decorate" => Ok(SetProp::Decorate(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "focusonactivate" => Ok(SetProp::FocusOnActivate(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "keepaspectratio" => Ok(SetProp::KeepAspectRatio(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nearestneighbor" => Ok(SetProp::NearestNeighbor(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noanim" => Ok(SetProp::NoAnim(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noblur" => Ok(SetProp::NoBlur(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noborder" => Ok(SetProp::NoBorder(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nodim" => Ok(SetProp::NoDim(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nofocus" => Ok(SetProp::NoFocus(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nofollowmouse" => Ok(SetProp::NoFollowMouse(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nomaxsize" => Ok(SetProp::NoMaxSize(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "norounding" => Ok(SetProp::NoRounding(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noshadow" => Ok(SetProp::NoShadow(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noshortcutsinhibit" => Ok(SetProp::NoShortcutsInhibit(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "opaque" => Ok(SetProp::Opaque(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "forcergbx" => Ok(SetProp::ForceRGBX(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "syncfullscreen" => Ok(SetProp::SyncFullscreen(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "immediate" => Ok(SetProp::Immediate(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "xray" => Ok(SetProp::Xray(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "renderunfocused" => Ok(SetProp::RenderUnfocused),
            "scrollmouse" => Ok(SetProp::ScrollMouse(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "scrolltouchpad" => Ok(SetProp::ScrollTouchpad(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noscreenshare" => Ok(SetProp::NoScreenShare(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "novrr" => Ok(SetProp::NoVRR(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            _ => Err(()),
        }
    }
}

impl Display for SetProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetProp::Alpha(alpha) => write!(f, "alpha {}", alpha),
            SetProp::AlphaOverride(false) => write!(f, "alphaoverride 0"),
            SetProp::AlphaOverride(true) => write!(f, "alphaoverride 1"),
            SetProp::AlphaInactive(alpha) => write!(f, "alphainactive {}", alpha),
            SetProp::AlphaInactiveOverride(false) => write!(f, "alphainactiveoverride 0"),
            SetProp::AlphaInactiveOverride(true) => write!(f, "alphainactiveoverride 1"),
            SetProp::AlphaFullscreen(alpha) => write!(f, "alphafullscreen {}", alpha),
            SetProp::AlphaFullscreenOverride(false) => write!(f, "alphafullscreenoverride 0"),
            SetProp::AlphaFullscreenOverride(true) => write!(f, "alphafullscreenoverride 1"),
            SetProp::AnimationStyle(style) => write!(f, "animationstyle {}", style),
            SetProp::ActiveBorderColor(None) => write!(f, "activebordercolor -1"),
            SetProp::ActiveBorderColor(Some(color)) => write!(f, "activebordercolor {}", color),
            SetProp::InactiveBorderColor(None) => write!(f, "inactivebordercolor -1"),
            SetProp::InactiveBorderColor(Some(color)) => write!(f, "inactivebordercolor {}", color),
            SetProp::Animation(animation) => write!(f, "animation {}", animation),
            SetProp::BorderColor(border_color) => write!(f, "bordercolor {}", border_color),
            SetProp::IdleIngibit(mode) => write!(f, "idleingibit {}", mode),
            SetProp::Opacity(opacity) => write!(f, "opacity {}", opacity),
            SetProp::Tag(TagToggleState::Set, tag) => write!(f, "tag +{}", tag),
            SetProp::Tag(TagToggleState::Unset, tag) => write!(f, "tag -{}", tag),
            SetProp::Tag(TagToggleState::Toggle, tag) => write!(f, "tag {}", tag),
            SetProp::MaxSize(width, height) => write!(f, "maxsize {} {}", width, height),
            SetProp::MinSize(width, height) => write!(f, "minsize {} {}", width, height),
            SetProp::BorderSize(size) => write!(f, "bordersize {}", size),
            SetProp::Rounding(size) => write!(f, "rounding {}", size),
            SetProp::RoundingPower(power) => write!(f, "roundingpower {}", power),
            SetProp::AllowsInput(mode) => write!(f, "allowsinput {}", mode),
            SetProp::DimAround(mode) => write!(f, "dimaround {}", mode),
            SetProp::Decorate(mode) => write!(f, "decorate {}", mode),
            SetProp::FocusOnActivate(mode) => write!(f, "focusonactivate {}", mode),
            SetProp::KeepAspectRatio(mode) => write!(f, "keepaspectratio {}", mode),
            SetProp::NearestNeighbor(mode) => write!(f, "nearestneighbor {}", mode),
            SetProp::NoAnim(mode) => write!(f, "noanim {}", mode),
            SetProp::NoBlur(mode) => write!(f, "noblur {}", mode),
            SetProp::NoBorder(mode) => write!(f, "noborder {}", mode),
            SetProp::NoDim(mode) => write!(f, "nodim {}", mode),
            SetProp::NoFocus(mode) => write!(f, "nofocus {}", mode),
            SetProp::NoFollowMouse(mode) => write!(f, "nofollowmouse {}", mode),
            SetProp::NoMaxSize(mode) => write!(f, "nomaxsize {}", mode),
            SetProp::NoRounding(mode) => write!(f, "norounding {}", mode),
            SetProp::NoShadow(mode) => write!(f, "noshadow {}", mode),
            SetProp::NoShortcutsInhibit(mode) => write!(f, "noshortcutsinhibit {}", mode),
            SetProp::Opaque(mode) => write!(f, "opaque {}", mode),
            SetProp::ForceRGBX(mode) => write!(f, "forcergbx {}", mode),
            SetProp::SyncFullscreen(mode) => write!(f, "syncfullscreen {}", mode),
            SetProp::Immediate(mode) => write!(f, "immediate {}", mode),
            SetProp::Xray(mode) => write!(f, "xray {}", mode),
            SetProp::RenderUnfocused => write!(f, "renderunfocused"),
            SetProp::ScrollMouse(speed) => write!(f, "scrollmouse {}", speed),
            SetProp::ScrollTouchpad(speed) => write!(f, "scrolltouchpad {}", speed),
            SetProp::NoScreenShare(mode) => write!(f, "noscreenshare {}", mode),
            SetProp::NoVRR(mode) => write!(f, "novrr {}", mode),
        }
    }
}

impl EnumConfigForGtk for SetProp {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.set_prop.alpha"),
            &t!("hyprland.set_prop.alpha_override"),
            &t!("hyprland.set_prop.alpha_inactive"),
            &t!("hyprland.set_prop.alpha_inactive_override"),
            &t!("hyprland.set_prop.alpha_fullscreen"),
            &t!("hyprland.set_prop.alpha_fullscreen_override"),
            &t!("hyprland.set_prop.animation_style"),
            &t!("hyprland.set_prop.active_border_color"),
            &t!("hyprland.set_prop.inactive_border_color"),
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
            &t!("hyprland.set_prop.no_border"),
            &t!("hyprland.set_prop.no_dim"),
            &t!("hyprland.set_prop.no_focus"),
            &t!("hyprland.set_prop.no_follow_mouse"),
            &t!("hyprland.set_prop.no_max_size"),
            &t!("hyprland.set_prop.no_rounding"),
            &t!("hyprland.set_prop.no_shadow"),
            &t!("hyprland.set_prop.no_shortcuts_inhibit"),
            &t!("hyprland.set_prop.opaque"),
            &t!("hyprland.set_prop.force_rgbx"),
            &t!("hyprland.set_prop.sync_fullscreen"),
            &t!("hyprland.set_prop.immediate"),
            &t!("hyprland.set_prop.xray"),
            &t!("hyprland.set_prop.render_unfocused"),
            &t!("hyprland.set_prop.scroll_mouse"),
            &t!("hyprland.set_prop.scroll_touchpad"),
            &t!("hyprland.set_prop.no_screenshare"),
            &t!("hyprland.set_prop.no_vrr"),
        ])
    }

    const SEPARATOR: Option<char> = Some(' ');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            SetProp::Alpha(_alpha) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, 1.0, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::AlphaOverride(_override) => Some(<(bool,)>::to_gtk_box),
            SetProp::AlphaInactive(_alpha) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, 1.0, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::AlphaInactiveOverride(_override) => Some(<(bool,)>::to_gtk_box),
            SetProp::AlphaFullscreen(_alpha) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, 1.0, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::AlphaFullscreenOverride(_override) => Some(<(bool,)>::to_gtk_box),
            SetProp::AnimationStyle(_style) => Some(<(String,)>::to_gtk_box),
            SetProp::ActiveBorderColor(_optional_gradient) => {
                Some(|entry, _, _, _| Option::<HyprGradient>::to_gtk_box(entry))
            }
            SetProp::InactiveBorderColor(_optional_gradient) => {
                Some(|entry, _, _, _| Option::<HyprGradient>::to_gtk_box(entry))
            }
            SetProp::Animation(_style) => Some(<(AnimationStyle,)>::to_gtk_box),
            SetProp::BorderColor(_color) => Some(<(BorderColor,)>::to_gtk_box),
            SetProp::IdleIngibit(_mode) => Some(<(IdleIngibitMode,)>::to_gtk_box),
            SetProp::Opacity(_opacity) => Some(<(HyprOpacity,)>::to_gtk_box),
            SetProp::Tag(_toggle_state, _tag) => Some(|entry, _, names, custom_split| {
                <(TagToggleState, String)>::to_gtk_box(entry, PLUG_SEPARATOR, names, custom_split)
            }),
            SetProp::MaxSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            SetProp::MinSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            SetProp::BorderSize(_size) => Some(<(u32,)>::to_gtk_box),
            SetProp::Rounding(_size) => Some(<(u32,)>::to_gtk_box),
            SetProp::RoundingPower(_power) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::AllowsInput(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::DimAround(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::Decorate(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::FocusOnActivate(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::KeepAspectRatio(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NearestNeighbor(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoAnim(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoBlur(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoBorder(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoDim(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoFocus(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoFollowMouse(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoMaxSize(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoRounding(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoShadow(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoShortcutsInhibit(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::Opaque(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::ForceRGBX(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::SyncFullscreen(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::Immediate(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::Xray(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::RenderUnfocused => None,
            SetProp::ScrollMouse(_scroll_factor) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::ScrollTouchpad(_scroll_factor) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::NoScreenShare(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoVRR(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(SetProp);
register_togtkbox_with_separator_names!(
    (bool,),
    (String,),
    (AnimationStyle,),
    (BorderColor,),
    (IdleIngibitMode,),
    (HyprOpacity,),
    (TagToggleState, String),
    (u32, u32),
    (u32,),
    (SetPropToggleState,),
);
