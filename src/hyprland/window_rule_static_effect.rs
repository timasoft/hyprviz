use super::{
    ContentType, FullscreenState, HyprExpression, IdOrName, WindowEvent, WindowGroupOption,
    WorkspaceTarget,
};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, ToGtkBoxWithSeparator, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, join_with_separator, parse_bool},
};
use gtk::StringList;
use rust_i18n::t;
use std::{collections::HashSet, fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WindowRuleStaticEffectDiscriminant))]
#[derive(Default)]
pub enum WindowRuleStaticEffect {
    #[default]
    FloatOn,
    FloatOff,
    TileOn,
    TileOff,
    FullscreenOn,
    FullscreenOff,
    MaximizeOn,
    MaximizeOff,
    FullscreenState(FullscreenState, FullscreenState),
    Move(HyprExpression, HyprExpression),
    Size(HyprExpression, HyprExpression),
    CenterOn,
    CenterOff,
    PseudoOn,
    PseudoOff,
    Monitor(IdOrName),
    Workspace(WorkspaceTarget),
    NoInitialFocusOn,
    NoInitialFocusOff,
    PinOn,
    PinOff,
    Group(Vec<WindowGroupOption>),
    SuppressEvent(HashSet<WindowEvent>),
    Content(ContentType),
    NoCloseFor(u32),
}

impl HasDiscriminant for WindowRuleStaticEffect {
    type Discriminant = WindowRuleStaticEffectDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::FloatOn => Self::FloatOn,
            Self::Discriminant::FloatOff => Self::FloatOff,
            Self::Discriminant::TileOn => Self::TileOn,
            Self::Discriminant::TileOff => Self::TileOff,
            Self::Discriminant::FullscreenOn => Self::FullscreenOn,
            Self::Discriminant::FullscreenOff => Self::FullscreenOff,
            Self::Discriminant::MaximizeOn => Self::MaximizeOn,
            Self::Discriminant::MaximizeOff => Self::MaximizeOff,
            Self::Discriminant::FullscreenState => {
                Self::FullscreenState(FullscreenState::default(), FullscreenState::default())
            }
            Self::Discriminant::Move => {
                Self::Move(HyprExpression::default(), HyprExpression::default())
            }
            Self::Discriminant::Size => {
                Self::Size(HyprExpression::default(), HyprExpression::default())
            }
            Self::Discriminant::CenterOn => Self::CenterOn,
            Self::Discriminant::CenterOff => Self::CenterOff,
            Self::Discriminant::PseudoOn => Self::PseudoOn,
            Self::Discriminant::PseudoOff => Self::PseudoOff,
            Self::Discriminant::Monitor => Self::Monitor(IdOrName::default()),
            Self::Discriminant::Workspace => Self::Workspace(WorkspaceTarget::default()),
            Self::Discriminant::NoInitialFocusOn => Self::NoInitialFocusOn,
            Self::Discriminant::NoInitialFocusOff => Self::NoInitialFocusOff,
            Self::Discriminant::PinOn => Self::PinOn,
            Self::Discriminant::PinOff => Self::PinOff,
            Self::Discriminant::Group => Self::Group(vec![WindowGroupOption::default()]),
            Self::Discriminant::SuppressEvent => {
                Self::SuppressEvent([WindowEvent::default()].into_iter().collect())
            }
            Self::Discriminant::Content => Self::Content(ContentType::default()),
            Self::Discriminant::NoCloseFor => Self::NoCloseFor(0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::FloatOn => Self::FloatOn,
            Self::Discriminant::FloatOff => Self::FloatOff,
            Self::Discriminant::TileOn => Self::TileOn,
            Self::Discriminant::TileOff => Self::TileOff,
            Self::Discriminant::FullscreenOn => Self::FullscreenOn,
            Self::Discriminant::FullscreenOff => Self::FullscreenOff,
            Self::Discriminant::MaximizeOn => Self::MaximizeOn,
            Self::Discriminant::MaximizeOff => Self::MaximizeOff,
            Self::Discriminant::FullscreenState => {
                let (internal, client) = str.split_once(' ').unwrap_or((str, ""));
                Self::FullscreenState(
                    internal.parse().unwrap_or_default(),
                    client.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::Move => {
                let (x, y) = str.split_once(' ').unwrap_or((str, ""));
                Self::Move(x.parse().unwrap_or_default(), y.parse().unwrap_or_default())
            }
            Self::Discriminant::Size => {
                let (w, h) = str.split_once(' ').unwrap_or((str, ""));
                Self::Size(w.parse().unwrap_or_default(), h.parse().unwrap_or_default())
            }
            Self::Discriminant::CenterOn => Self::CenterOn,
            Self::Discriminant::CenterOff => Self::CenterOff,
            Self::Discriminant::PseudoOn => Self::PseudoOn,
            Self::Discriminant::PseudoOff => Self::PseudoOff,
            Self::Discriminant::Monitor => Self::Monitor(str.parse().unwrap_or_default()),
            Self::Discriminant::Workspace => Self::Workspace(str.parse().unwrap_or_default()),
            Self::Discriminant::NoInitialFocusOn => Self::NoInitialFocusOn,
            Self::Discriminant::NoInitialFocusOff => Self::NoInitialFocusOff,
            Self::Discriminant::PinOn => Self::PinOn,
            Self::Discriminant::PinOff => Self::PinOff,
            Self::Discriminant::Group => Self::Group(
                str.split(' ')
                    .map(|s| s.parse().unwrap_or_default())
                    .collect(),
            ),
            Self::Discriminant::SuppressEvent => Self::SuppressEvent(
                str.split(' ')
                    .map(|s| s.parse().unwrap_or_default())
                    .collect(),
            ),
            Self::Discriminant::Content => Self::Content(str.parse().unwrap_or_default()),
            Self::Discriminant::NoCloseFor => Self::NoCloseFor(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WindowRuleStaticEffect::FloatOn => None,
            WindowRuleStaticEffect::FloatOff => None,
            WindowRuleStaticEffect::TileOn => None,
            WindowRuleStaticEffect::TileOff => None,
            WindowRuleStaticEffect::FullscreenOn => None,
            WindowRuleStaticEffect::FullscreenOff => None,
            WindowRuleStaticEffect::MaximizeOn => None,
            WindowRuleStaticEffect::MaximizeOff => None,
            WindowRuleStaticEffect::FullscreenState(internal, client) => {
                Some(format!("{} {}", internal.to_num(), client.to_num()))
            }
            WindowRuleStaticEffect::Move(x, y) => Some(format!("{} {}", x, y)),
            WindowRuleStaticEffect::Size(w, h) => Some(format!("{} {}", w, h)),
            WindowRuleStaticEffect::CenterOn => None,
            WindowRuleStaticEffect::CenterOff => None,
            WindowRuleStaticEffect::PseudoOn => None,
            WindowRuleStaticEffect::PseudoOff => None,
            WindowRuleStaticEffect::Monitor(target) => Some(target.to_string()),
            WindowRuleStaticEffect::Workspace(target) => Some(target.to_string()),
            WindowRuleStaticEffect::NoInitialFocusOn => None,
            WindowRuleStaticEffect::NoInitialFocusOff => None,
            WindowRuleStaticEffect::PinOn => None,
            WindowRuleStaticEffect::PinOff => None,
            WindowRuleStaticEffect::Group(group) => Some(join_with_separator(group, " ")),
            WindowRuleStaticEffect::SuppressEvent(events) => Some(join_with_separator(events, " ")),
            WindowRuleStaticEffect::Content(content) => Some(content.to_string()),
            WindowRuleStaticEffect::NoCloseFor(duration) => Some(duration.to_string()),
        }
    }

    fn custom_split(_discriminant: Self::Discriminant) -> Option<fn(&str) -> Vec<&str>> {
        None
    }
}

impl FromStr for WindowRuleStaticEffect {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }
        let (part1, part2) = s.split_once(' ').unwrap_or((s, ""));
        match part1.trim().to_lowercase().as_str() {
            "float" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleStaticEffect::FloatOn),
                Some(false) => Ok(WindowRuleStaticEffect::FloatOff),
                None => Ok(WindowRuleStaticEffect::FloatOff),
            },
            "tile" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleStaticEffect::TileOn),
                Some(false) => Ok(WindowRuleStaticEffect::TileOff),
                None => Ok(WindowRuleStaticEffect::TileOff),
            },
            "fullscreen" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleStaticEffect::FullscreenOn),
                Some(false) => Ok(WindowRuleStaticEffect::FullscreenOff),
                None => Ok(WindowRuleStaticEffect::FullscreenOff),
            },
            "maximize" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleStaticEffect::MaximizeOn),
                Some(false) => Ok(WindowRuleStaticEffect::MaximizeOff),
                None => Ok(WindowRuleStaticEffect::MaximizeOff),
            },
            "fullscreenstate" => {
                let (internal, client) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRuleStaticEffect::FullscreenState(
                    FullscreenState::from_num(internal.parse().unwrap_or_default()),
                    FullscreenState::from_num(client.parse().unwrap_or_default()),
                ))
            }
            "move" => {
                let (x, y) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRuleStaticEffect::Move(
                    x.parse().unwrap_or_default(),
                    y.parse().unwrap_or_default(),
                ))
            }
            "size" => {
                let (w, h) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRuleStaticEffect::Size(
                    w.parse().unwrap_or_default(),
                    h.parse().unwrap_or_default(),
                ))
            }
            "center" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleStaticEffect::CenterOn),
                Some(false) => Ok(WindowRuleStaticEffect::CenterOff),
                None => Ok(WindowRuleStaticEffect::CenterOff),
            },
            "pseudo" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleStaticEffect::PseudoOn),
                Some(false) => Ok(WindowRuleStaticEffect::PseudoOff),
                None => Ok(WindowRuleStaticEffect::PseudoOff),
            },
            "monitor" => Ok(WindowRuleStaticEffect::Monitor(
                part2.parse().unwrap_or_default(),
            )),
            "workspace" => Ok(WindowRuleStaticEffect::Workspace(
                part2.parse().unwrap_or_default(),
            )),
            "noinitialfocus" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleStaticEffect::NoInitialFocusOn),
                Some(false) => Ok(WindowRuleStaticEffect::NoInitialFocusOff),
                None => Ok(WindowRuleStaticEffect::NoInitialFocusOff),
            },
            "pin" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleStaticEffect::PinOn),
                Some(false) => Ok(WindowRuleStaticEffect::PinOff),
                None => Ok(WindowRuleStaticEffect::PinOff),
            },
            "group" => Ok(WindowRuleStaticEffect::Group(
                part2
                    .split(' ')
                    .map(|s| WindowGroupOption::from_str(s).unwrap_or_default())
                    .collect(),
            )),
            "suppress" => Ok(WindowRuleStaticEffect::SuppressEvent(
                part2
                    .split(' ')
                    .map(|s| WindowEvent::from_str(s).unwrap_or_default())
                    .collect(),
            )),
            "content" => Ok(WindowRuleStaticEffect::Content(
                part2.parse().unwrap_or_default(),
            )),
            "noclosefor" => Ok(WindowRuleStaticEffect::NoCloseFor(
                part2.parse().unwrap_or_default(),
            )),
            _ => Err(()),
        }
    }
}

impl Display for WindowRuleStaticEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRuleStaticEffect::FloatOn => write!(f, "float on"),
            WindowRuleStaticEffect::FloatOff => write!(f, "float off"),
            WindowRuleStaticEffect::TileOn => write!(f, "tile on"),
            WindowRuleStaticEffect::TileOff => write!(f, "tile off"),
            WindowRuleStaticEffect::FullscreenOn => write!(f, "fullscreen on"),
            WindowRuleStaticEffect::FullscreenOff => write!(f, "fullscreen off"),
            WindowRuleStaticEffect::MaximizeOn => write!(f, "maximize on"),
            WindowRuleStaticEffect::MaximizeOff => write!(f, "maximize off"),
            WindowRuleStaticEffect::FullscreenState(internal, client) => write!(
                f,
                "fullscreenstate {} {}",
                internal.to_num(),
                client.to_num()
            ),
            WindowRuleStaticEffect::Move(x, y) => write!(f, "move {} {}", x, y),
            WindowRuleStaticEffect::Size(w, h) => write!(f, "size {} {}", w, h),
            WindowRuleStaticEffect::CenterOn => write!(f, "center on"),
            WindowRuleStaticEffect::CenterOff => write!(f, "center off"),
            WindowRuleStaticEffect::PseudoOn => write!(f, "pseudo on"),
            WindowRuleStaticEffect::PseudoOff => write!(f, "pseudo off"),
            WindowRuleStaticEffect::Monitor(monitor) => write!(f, "monitor {}", monitor),
            WindowRuleStaticEffect::Workspace(workspace) => write!(f, "workspace {}", workspace),
            WindowRuleStaticEffect::NoInitialFocusOn => write!(f, "noinitialfocus on"),
            WindowRuleStaticEffect::NoInitialFocusOff => write!(f, "noinitialfocus off"),
            WindowRuleStaticEffect::PinOn => write!(f, "pin on"),
            WindowRuleStaticEffect::PinOff => write!(f, "pin off"),
            WindowRuleStaticEffect::Group(group) => {
                write!(f, "group {}", join_with_separator(group, " "))
            }
            WindowRuleStaticEffect::SuppressEvent(suppress) => {
                write!(f, "suppress {}", join_with_separator(suppress, " "))
            }
            WindowRuleStaticEffect::Content(content) => write!(f, "content {}", content),
            WindowRuleStaticEffect::NoCloseFor(no_close_for) => {
                write!(f, "noclosefor {}", no_close_for)
            }
        }
    }
}

impl EnumConfigForGtk for WindowRuleStaticEffect {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.window_rule_static_effect.float_on"),
            &t!("hyprland.window_rule_static_effect.float_off"),
            &t!("hyprland.window_rule_static_effect.tile_on"),
            &t!("hyprland.window_rule_static_effect.tile_off"),
            &t!("hyprland.window_rule_static_effect.fullscreen_on"),
            &t!("hyprland.window_rule_static_effect.fullscreen_off"),
            &t!("hyprland.window_rule_static_effect.maximize_on"),
            &t!("hyprland.window_rule_static_effect.maximize_off"),
            &t!("hyprland.window_rule_static_effect.fullscreen_state"),
            &t!("hyprland.window_rule_static_effect.move"),
            &t!("hyprland.window_rule_static_effect.size"),
            &t!("hyprland.window_rule_static_effect.center_on"),
            &t!("hyprland.window_rule_static_effect.center_off"),
            &t!("hyprland.window_rule_static_effect.pseudo_on"),
            &t!("hyprland.window_rule_static_effect.pseudo_off"),
            &t!("hyprland.window_rule_static_effect.monitor"),
            &t!("hyprland.window_rule_static_effect.workspace"),
            &t!("hyprland.window_rule_static_effect.no_initial_focus_on"),
            &t!("hyprland.window_rule_static_effect.no_initial_focus_off"),
            &t!("hyprland.window_rule_static_effect.pin_on"),
            &t!("hyprland.window_rule_static_effect.pin_off"),
            &t!("hyprland.window_rule_static_effect.group"),
            &t!("hyprland.window_rule_static_effect.suppress_event"),
            &t!("hyprland.window_rule_static_effect.content"),
            &t!("hyprland.window_rule_static_effect.no_close_for"),
        ])
    }

    const SEPARATOR: Option<char> = Some(' ');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            WindowRuleStaticEffect::FloatOn => None,
            WindowRuleStaticEffect::FloatOff => None,
            WindowRuleStaticEffect::TileOn => None,
            WindowRuleStaticEffect::TileOff => None,
            WindowRuleStaticEffect::FullscreenOn => None,
            WindowRuleStaticEffect::FullscreenOff => None,
            WindowRuleStaticEffect::MaximizeOn => None,
            WindowRuleStaticEffect::MaximizeOff => None,
            WindowRuleStaticEffect::FullscreenState(_, _) => {
                Some(<(FullscreenState, FullscreenState)>::to_gtk_box)
            }
            WindowRuleStaticEffect::Move(_, _) => {
                Some(<(HyprExpression, HyprExpression)>::to_gtk_box)
            }
            WindowRuleStaticEffect::Size(_, _) => {
                Some(<(HyprExpression, HyprExpression)>::to_gtk_box)
            }
            WindowRuleStaticEffect::CenterOn => None,
            WindowRuleStaticEffect::CenterOff => None,
            WindowRuleStaticEffect::PseudoOn => None,
            WindowRuleStaticEffect::PseudoOff => None,
            WindowRuleStaticEffect::Monitor(_) => Some(<(IdOrName,)>::to_gtk_box),
            WindowRuleStaticEffect::Workspace(_) => Some(<(WorkspaceTarget,)>::to_gtk_box),
            WindowRuleStaticEffect::NoInitialFocusOn => None,
            WindowRuleStaticEffect::NoInitialFocusOff => None,
            WindowRuleStaticEffect::PinOn => None,
            WindowRuleStaticEffect::PinOff => None,
            WindowRuleStaticEffect::Group(_) => Some(|entry, separator, _names, _| {
                Vec::<WindowGroupOption>::to_gtk_box(entry, separator)
            }),
            WindowRuleStaticEffect::SuppressEvent(_) => Some(|entry, separator, _names, _| {
                HashSet::<WindowEvent>::to_gtk_box(entry, separator)
            }),
            WindowRuleStaticEffect::Content(_) => Some(<(ContentType,)>::to_gtk_box),
            WindowRuleStaticEffect::NoCloseFor(_) => Some(<(u32,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(WindowRuleStaticEffect);
register_togtkbox_with_separator_names!((HyprExpression, HyprExpression),);
