use super::{
    AnimationStyle, BorderColor, ContentType, FullscreenState, HyprCoord, HyprOpacity, HyprSize,
    IdOrName, IdleIngibitMode, TagToggleState, WindowEvent, WindowGroupOption, WorkspaceTarget,
};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, PLUG_SEPARATOR, ToGtkBoxWithSeparator, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, join_with_separator},
};
use gtk::StringList;
use rust_i18n::t;
use std::{collections::HashSet, fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WindowRuleDiscriminant))]
#[derive(Default)]
pub enum WindowRule {
    #[default]
    Float,
    Tile,
    Fullscreen,
    Maximize,
    PersistentSize,
    FullscreenState(FullscreenState, FullscreenState),
    Move(HyprCoord),
    Size(HyprSize),
    Center,
    CenterWithRespectToMonitorReservedArea,
    Pseudo,
    Monitor(IdOrName),
    Workspace(WorkspaceTarget),
    NoInitialFocus,
    Pin,
    Unset,
    NoMaxSize,
    StayFocused,
    Group(Vec<WindowGroupOption>),
    SuppressEvent(HashSet<WindowEvent>),
    Content(ContentType),
    NoCloseFor(u32),
    Animation(AnimationStyle),
    BorderColor(BorderColor),
    IdleIngibit(IdleIngibitMode),
    Opacity(HyprOpacity),
    Tag(TagToggleState, String),
    MaxSize(u32, u32),
    MinSize(u32, u32),
}

impl HasDiscriminant for WindowRule {
    type Discriminant = WindowRuleDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Float => Self::Float,
            Self::Discriminant::Tile => Self::Tile,
            Self::Discriminant::Fullscreen => Self::Fullscreen,
            Self::Discriminant::Maximize => Self::Maximize,
            Self::Discriminant::PersistentSize => Self::PersistentSize,
            Self::Discriminant::FullscreenState => {
                Self::FullscreenState(FullscreenState::None, FullscreenState::None)
            }
            Self::Discriminant::Move => Self::Move(HyprCoord::default()),
            Self::Discriminant::Size => Self::Size(HyprSize::default()),
            Self::Discriminant::Center => Self::Center,
            Self::Discriminant::CenterWithRespectToMonitorReservedArea => {
                Self::CenterWithRespectToMonitorReservedArea
            }
            Self::Discriminant::Pseudo => Self::Pseudo,
            Self::Discriminant::Monitor => Self::Monitor(IdOrName::default()),
            Self::Discriminant::Workspace => Self::Workspace(WorkspaceTarget::default()),
            Self::Discriminant::NoInitialFocus => Self::NoInitialFocus,
            Self::Discriminant::Pin => Self::Pin,
            Self::Discriminant::Unset => Self::Unset,
            Self::Discriminant::NoMaxSize => Self::NoMaxSize,
            Self::Discriminant::StayFocused => Self::StayFocused,
            Self::Discriminant::Group => Self::Group(vec![WindowGroupOption::default()]),
            Self::Discriminant::SuppressEvent => {
                Self::SuppressEvent([WindowEvent::default()].into_iter().collect())
            }
            Self::Discriminant::Content => Self::Content(ContentType::default()),
            Self::Discriminant::NoCloseFor => Self::NoCloseFor(0),
            Self::Discriminant::Animation => Self::Animation(AnimationStyle::default()),
            Self::Discriminant::BorderColor => Self::BorderColor(BorderColor::default()),
            Self::Discriminant::IdleIngibit => Self::IdleIngibit(IdleIngibitMode::default()),
            Self::Discriminant::Opacity => Self::Opacity(HyprOpacity::default()),
            Self::Discriminant::Tag => Self::Tag(TagToggleState::Toggle, "".to_string()),
            Self::Discriminant::MaxSize => Self::MaxSize(0, 0),
            Self::Discriminant::MinSize => Self::MinSize(0, 0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Float => Self::Float,
            Self::Discriminant::Tile => Self::Tile,
            Self::Discriminant::Fullscreen => Self::Fullscreen,
            Self::Discriminant::Maximize => Self::Maximize,
            Self::Discriminant::PersistentSize => Self::PersistentSize,
            Self::Discriminant::FullscreenState => {
                let (internal, client) = str.split_once(' ').unwrap_or((str, ""));
                Self::FullscreenState(
                    internal.parse().unwrap_or_default(),
                    client.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::Move => Self::Move(str.parse().unwrap_or_default()),
            Self::Discriminant::Size => Self::Size(str.parse().unwrap_or_default()),
            Self::Discriminant::Center => Self::Center,
            Self::Discriminant::CenterWithRespectToMonitorReservedArea => {
                Self::CenterWithRespectToMonitorReservedArea
            }
            Self::Discriminant::Pseudo => Self::Pseudo,
            Self::Discriminant::Monitor => Self::Monitor(str.parse().unwrap_or_default()),
            Self::Discriminant::Workspace => Self::Workspace(str.parse().unwrap_or_default()),
            Self::Discriminant::NoInitialFocus => Self::NoInitialFocus,
            Self::Discriminant::Pin => Self::Pin,
            Self::Discriminant::Unset => Self::Unset,
            Self::Discriminant::NoMaxSize => Self::NoMaxSize,
            Self::Discriminant::StayFocused => Self::StayFocused,
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
                let (width, height) = str.split_once(' ').unwrap_or((str, ""));
                Self::MaxSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::MinSize => {
                let (width, height) = str.split_once(' ').unwrap_or((str, ""));
                Self::MinSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                )
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WindowRule::Float => None,
            WindowRule::Tile => None,
            WindowRule::Fullscreen => None,
            WindowRule::Maximize => None,
            WindowRule::PersistentSize => None,
            WindowRule::FullscreenState(internal, client) => {
                Some(format!("{} {}", internal.to_num(), client.to_num()))
            }
            WindowRule::Move(coord) => Some(coord.to_string()),
            WindowRule::Size(size) => Some(size.to_string()),
            WindowRule::Center => None,
            WindowRule::CenterWithRespectToMonitorReservedArea => None,
            WindowRule::Pseudo => None,
            WindowRule::Monitor(target) => Some(target.to_string()),
            WindowRule::Workspace(target) => Some(target.to_string()),
            WindowRule::NoInitialFocus => None,
            WindowRule::Pin => None,
            WindowRule::Unset => None,
            WindowRule::NoMaxSize => None,
            WindowRule::StayFocused => None,
            WindowRule::Group(group) => Some(join_with_separator(group, " ")),
            WindowRule::SuppressEvent(events) => Some(join_with_separator(events, " ")),
            WindowRule::Content(content) => Some(content.to_string()),
            WindowRule::NoCloseFor(duration) => Some(duration.to_string()),
            WindowRule::Animation(animation) => Some(animation.to_string()),
            WindowRule::BorderColor(color) => Some(color.to_string()),
            WindowRule::IdleIngibit(mode) => Some(mode.to_string()),
            WindowRule::Opacity(opacity) => Some(opacity.to_string()),
            WindowRule::Tag(toggle_state, tag) => Some(match toggle_state {
                TagToggleState::Set => format!("+{}", tag),
                TagToggleState::Unset => format!("-{}", tag),
                TagToggleState::Toggle => tag.clone(),
            }),
            WindowRule::MaxSize(width, height) => Some(format!("{} {}", width, height)),
            WindowRule::MinSize(width, height) => Some(format!("{} {}", width, height)),
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

impl FromStr for WindowRule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }
        let (part1, part2) = s.split_once(' ').unwrap_or((s, ""));
        match part1.trim().to_lowercase().as_str() {
            "float" => Ok(WindowRule::Float),
            "tile" => Ok(WindowRule::Tile),
            "fullscreen" => Ok(WindowRule::Fullscreen),
            "maximize" => Ok(WindowRule::Maximize),
            "persistent" => Ok(WindowRule::PersistentSize),
            "fullscreenstate" => {
                let (internal, client) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRule::FullscreenState(
                    FullscreenState::from_num(internal.parse().unwrap_or_default()),
                    FullscreenState::from_num(client.parse().unwrap_or_default()),
                ))
            }
            "move" => Ok(WindowRule::Move(part2.parse().unwrap_or_default())),
            "size" => Ok(WindowRule::Size(part2.parse().unwrap_or_default())),
            "center" => match part2.trim().to_lowercase().as_str() {
                "1" => Ok(WindowRule::CenterWithRespectToMonitorReservedArea),
                _ => Ok(WindowRule::Center),
            },
            "pseudo" => Ok(WindowRule::Pseudo),
            "monitor" => Ok(WindowRule::Monitor(part2.parse().unwrap_or_default())),
            "workspace" => Ok(WindowRule::Workspace(part2.parse().unwrap_or_default())),
            "noinitialfocus" => Ok(WindowRule::NoInitialFocus),
            "pin" => Ok(WindowRule::Pin),
            "unset" => Ok(WindowRule::Unset),
            "nomaxsize" => Ok(WindowRule::NoMaxSize),
            "stayfocused" => Ok(WindowRule::StayFocused),
            "group" => Ok(WindowRule::Group(
                part2
                    .split(' ')
                    .map(|s| WindowGroupOption::from_str(s).unwrap_or_default())
                    .collect(),
            )),
            "suppress" => Ok(WindowRule::SuppressEvent(
                part2
                    .split(' ')
                    .map(|s| WindowEvent::from_str(s).unwrap_or_default())
                    .collect(),
            )),
            "content" => Ok(WindowRule::Content(part2.parse().unwrap_or_default())),
            "noclosefor" => Ok(WindowRule::NoCloseFor(part2.parse().unwrap_or_default())),
            "animation" => Ok(WindowRule::Animation(part2.parse().unwrap_or_default())),
            "bordercolor" => Ok(WindowRule::BorderColor(part2.parse().unwrap_or_default())),
            "idleingibit" => Ok(WindowRule::IdleIngibit(part2.parse().unwrap_or_default())),
            "opacity" => Ok(WindowRule::Opacity(part2.parse().unwrap_or_default())),
            "tag" => {
                if let Some(stripped) = part2.strip_prefix("+") {
                    Ok(WindowRule::Tag(
                        TagToggleState::Set,
                        stripped.trim().to_string(),
                    ))
                } else if let Some(stripped) = part2.strip_prefix("-") {
                    Ok(WindowRule::Tag(
                        TagToggleState::Unset,
                        stripped.trim().to_string(),
                    ))
                } else {
                    Ok(WindowRule::Tag(
                        TagToggleState::Toggle,
                        part2.trim().to_string(),
                    ))
                }
            }
            "maxsize" => {
                let (width, height) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRule::MaxSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            "minsize" => {
                let (width, height) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRule::MinSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            _ => Err(()),
        }
    }
}

impl Display for WindowRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRule::Float => write!(f, "float"),
            WindowRule::Tile => write!(f, "tile"),
            WindowRule::Fullscreen => write!(f, "fullscreen"),
            WindowRule::Maximize => write!(f, "maximize"),
            WindowRule::PersistentSize => write!(f, "persistent"),
            WindowRule::FullscreenState(internal, client) => write!(
                f,
                "fullscreenstate {} {}",
                internal.to_num(),
                client.to_num()
            ),
            WindowRule::Move(move_) => write!(f, "move {}", move_),
            WindowRule::Size(size) => write!(f, "size {}", size),
            WindowRule::Center => write!(f, "center"),
            WindowRule::CenterWithRespectToMonitorReservedArea => write!(f, "center 1"),
            WindowRule::Pseudo => write!(f, "pseudo"),
            WindowRule::Monitor(monitor) => write!(f, "monitor {}", monitor),
            WindowRule::Workspace(workspace) => write!(f, "workspace {}", workspace),
            WindowRule::NoInitialFocus => write!(f, "noinitialfocus"),
            WindowRule::Pin => write!(f, "pin"),
            WindowRule::Unset => write!(f, "unset"),
            WindowRule::NoMaxSize => write!(f, "nomaxsize"),
            WindowRule::StayFocused => write!(f, "stayfocused"),
            WindowRule::Group(group) => write!(f, "group {}", join_with_separator(group, " ")),
            WindowRule::SuppressEvent(suppress) => {
                write!(f, "suppress {}", join_with_separator(suppress, " "))
            }
            WindowRule::Content(content) => write!(f, "content {}", content),
            WindowRule::NoCloseFor(no_close_for) => write!(f, "noclosefor {}", no_close_for),
            WindowRule::Animation(animation) => write!(f, "animation {}", animation),
            WindowRule::BorderColor(border_color) => write!(f, "bordercolor {}", border_color),
            WindowRule::IdleIngibit(idle_ingibit) => write!(f, "idleingibit {}", idle_ingibit),
            WindowRule::Opacity(opacity) => write!(f, "opacity {}", opacity),
            WindowRule::Tag(TagToggleState::Set, tag) => write!(f, "tag +{}", tag),
            WindowRule::Tag(TagToggleState::Unset, tag) => write!(f, "tag -{}", tag),
            WindowRule::Tag(TagToggleState::Toggle, tag) => write!(f, "tag {}", tag),
            WindowRule::MaxSize(width, height) => write!(f, "maxsize {} {}", width, height),
            WindowRule::MinSize(width, height) => write!(f, "minsize {} {}", width, height),
        }
    }
}

impl EnumConfigForGtk for WindowRule {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.float"),
            &t!("gtk_converters.tile"),
            &t!("gtk_converters.fullscreen"),
            &t!("gtk_converters.maximize"),
            &t!("gtk_converters.persistent_size"),
            &t!("gtk_converters.fullscreen_state"),
            &t!("gtk_converters.move"),
            &t!("gtk_converters.size"),
            &t!("gtk_converters.center"),
            &t!("gtk_converters.center_with_respect_to_monitor_reserved_area"),
            &t!("gtk_converters.pseudo"),
            &t!("gtk_converters.monitor"),
            &t!("gtk_converters.workspace"),
            &t!("gtk_converters.no_initial_focus"),
            &t!("gtk_converters.pin"),
            &t!("gtk_converters.unset"),
            &t!("gtk_converters.no_max_size"),
            &t!("gtk_converters.stay_focused"),
            &t!("gtk_converters.group"),
            &t!("gtk_converters.suppress_event"),
            &t!("gtk_converters.content"),
            &t!("gtk_converters.no_close_for"),
            &t!("gtk_converters.animation"),
            &t!("gtk_converters.border_color"),
            &t!("gtk_converters.idle_ingibit"),
            &t!("gtk_converters.opacity"),
            &t!("gtk_converters.tag"),
            &t!("gtk_converters.max_size"),
            &t!("gtk_converters.min_size"),
        ])
    }

    const SEPARATOR: Option<char> = Some(' ');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            WindowRule::Float => None,
            WindowRule::Tile => None,
            WindowRule::Fullscreen => None,
            WindowRule::Maximize => None,
            WindowRule::PersistentSize => None,
            WindowRule::FullscreenState(_fullscreen_state1, _fullscreen_state2) => {
                Some(<(FullscreenState, FullscreenState)>::to_gtk_box)
            }
            WindowRule::Move(_hypr_coord) => Some(<(HyprCoord,)>::to_gtk_box),
            WindowRule::Size(_hypr_size) => Some(<(HyprSize,)>::to_gtk_box),
            WindowRule::Center => None,
            WindowRule::CenterWithRespectToMonitorReservedArea => None,
            WindowRule::Pseudo => None,
            WindowRule::Monitor(_id_or_name) => Some(<(IdOrName,)>::to_gtk_box),
            WindowRule::Workspace(_workspace_target) => Some(<(WorkspaceTarget,)>::to_gtk_box),
            WindowRule::NoInitialFocus => None,
            WindowRule::Pin => None,
            WindowRule::Unset => None,
            WindowRule::NoMaxSize => None,
            WindowRule::StayFocused => None,
            WindowRule::Group(_window_group_option) => Some(|entry, separator, _names, _| {
                Vec::<WindowGroupOption>::to_gtk_box(entry, separator)
            }),
            WindowRule::SuppressEvent(_window_event) => Some(|entry, separator, _names, _| {
                HashSet::<WindowEvent>::to_gtk_box(entry, separator)
            }),
            WindowRule::Content(_content_type) => Some(<(ContentType,)>::to_gtk_box),
            WindowRule::NoCloseFor(_duration) => Some(<(u32,)>::to_gtk_box),
            WindowRule::Animation(_animation_style) => Some(<(AnimationStyle,)>::to_gtk_box),
            WindowRule::BorderColor(_border_color) => Some(<(BorderColor,)>::to_gtk_box),
            WindowRule::IdleIngibit(_idle_ingibit_mode) => Some(<(IdleIngibitMode,)>::to_gtk_box),
            WindowRule::Opacity(_hypr_opacity) => Some(<(HyprOpacity,)>::to_gtk_box),
            WindowRule::Tag(_tag_toggle_state, _tag) => {
                Some(|entry, _separator, names, custom_split| {
                    <(TagToggleState, String)>::to_gtk_box(
                        entry,
                        PLUG_SEPARATOR,
                        names,
                        custom_split,
                    )
                })
            }
            WindowRule::MaxSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            WindowRule::MinSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
        }
    }
}

register_togtkbox!(WindowRule);
register_togtkbox_with_separator!(Vec<WindowGroupOption>, HashSet<WindowEvent>);
register_togtkbox_with_separator_names!(
    (FullscreenState, FullscreenState),
    (HyprCoord,),
    (HyprSize,),
    (IdOrName,),
    (WorkspaceTarget,),
    (ContentType,),
    (u32,),
    (AnimationStyle,),
    (BorderColor,),
    (IdleIngibitMode,),
    (HyprOpacity,),
    (TagToggleState, String),
    (u32, u32),
);
