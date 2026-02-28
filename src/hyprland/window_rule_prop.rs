use super::{ContentTypeInt, IdOrNameOrWorkspaceSelector, WindowRuleFullscreenState};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, parse_bool},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WindowRulePropDiscriminant))]
pub enum WindowRuleProp {
    Class(String),
    Title(String),
    InitialClass(String),
    InitialTitle(String),
    Tag(String),
    Xwayland,
    NotXwayland,
    Floating,
    NotFloating,
    Fullscreen,
    NotFullscreen,
    Pin,
    NotPin,
    Focus,
    NotFocus,
    Group,
    NotGroup,
    Modal,
    NotModal,
    FullscreenStateClient(WindowRuleFullscreenState),
    FullscreenStateInternal(WindowRuleFullscreenState),
    Workspace(IdOrNameOrWorkspaceSelector),
    Content(ContentTypeInt),
    XdgTag(String),
}

impl HasDiscriminant for WindowRuleProp {
    type Discriminant = WindowRulePropDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Class => Self::Class(String::new()),
            Self::Discriminant::Title => Self::Title(String::new()),
            Self::Discriminant::InitialClass => Self::InitialClass(String::new()),
            Self::Discriminant::InitialTitle => Self::InitialTitle(String::new()),
            Self::Discriminant::Tag => Self::Tag(String::new()),
            Self::Discriminant::Xwayland => Self::Xwayland,
            Self::Discriminant::NotXwayland => Self::NotXwayland,
            Self::Discriminant::Floating => Self::Floating,
            Self::Discriminant::NotFloating => Self::NotFloating,
            Self::Discriminant::Fullscreen => Self::Fullscreen,
            Self::Discriminant::NotFullscreen => Self::NotFullscreen,
            Self::Discriminant::Pin => Self::Pin,
            Self::Discriminant::NotPin => Self::NotPin,
            Self::Discriminant::Focus => Self::Focus,
            Self::Discriminant::NotFocus => Self::NotFocus,
            Self::Discriminant::Group => Self::Group,
            Self::Discriminant::NotGroup => Self::NotGroup,
            Self::Discriminant::Modal => Self::Modal,
            Self::Discriminant::NotModal => Self::NotModal,
            Self::Discriminant::FullscreenStateClient => {
                Self::FullscreenStateClient(WindowRuleFullscreenState::default())
            }
            Self::Discriminant::FullscreenStateInternal => {
                Self::FullscreenStateInternal(WindowRuleFullscreenState::default())
            }
            Self::Discriminant::Workspace => {
                Self::Workspace(IdOrNameOrWorkspaceSelector::default())
            }
            Self::Discriminant::Content => Self::Content(ContentTypeInt::default()),
            Self::Discriminant::XdgTag => Self::XdgTag(String::new()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Class => Self::Class(str.to_string()),
            Self::Discriminant::Title => Self::Title(str.to_string()),
            Self::Discriminant::InitialClass => Self::InitialClass(str.to_string()),
            Self::Discriminant::InitialTitle => Self::InitialTitle(str.to_string()),
            Self::Discriminant::Tag => Self::Tag(str.to_string()),
            Self::Discriminant::Xwayland => Self::Xwayland,
            Self::Discriminant::NotXwayland => Self::NotXwayland,
            Self::Discriminant::Floating => Self::Floating,
            Self::Discriminant::NotFloating => Self::NotFloating,
            Self::Discriminant::Fullscreen => Self::Fullscreen,
            Self::Discriminant::NotFullscreen => Self::NotFullscreen,
            Self::Discriminant::Pin => Self::Pin,
            Self::Discriminant::NotPin => Self::NotPin,
            Self::Discriminant::Focus => Self::Focus,
            Self::Discriminant::NotFocus => Self::NotFocus,
            Self::Discriminant::Group => Self::Group,
            Self::Discriminant::NotGroup => Self::NotGroup,
            Self::Discriminant::Modal => Self::Modal,
            Self::Discriminant::NotModal => Self::NotModal,
            Self::Discriminant::FullscreenStateClient => {
                Self::FullscreenStateClient(str.parse().unwrap_or_default())
            }
            Self::Discriminant::FullscreenStateInternal => {
                Self::FullscreenStateInternal(str.parse().unwrap_or_default())
            }
            Self::Discriminant::Workspace => Self::Workspace(str.parse().unwrap_or_default()),
            Self::Discriminant::Content => Self::Content(str.parse().unwrap_or_default()),
            Self::Discriminant::XdgTag => Self::XdgTag(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WindowRuleProp::Class(class) => Some(class.clone()),
            WindowRuleProp::Title(title) => Some(title.clone()),
            WindowRuleProp::InitialClass(initial_class) => Some(initial_class.clone()),
            WindowRuleProp::InitialTitle(initial_title) => Some(initial_title.clone()),
            WindowRuleProp::Tag(tag) => Some(tag.clone()),
            WindowRuleProp::Xwayland => Some("on".to_string()),
            WindowRuleProp::NotXwayland => Some("off".to_string()),
            WindowRuleProp::Floating => Some("on".to_string()),
            WindowRuleProp::NotFloating => Some("off".to_string()),
            WindowRuleProp::Fullscreen => Some("on".to_string()),
            WindowRuleProp::NotFullscreen => Some("off".to_string()),
            WindowRuleProp::Pin => Some("on".to_string()),
            WindowRuleProp::NotPin => Some("off".to_string()),
            WindowRuleProp::Focus => Some("on".to_string()),
            WindowRuleProp::NotFocus => Some("off".to_string()),
            WindowRuleProp::Group => Some("on".to_string()),
            WindowRuleProp::NotGroup => Some("off".to_string()),
            WindowRuleProp::Modal => Some("on".to_string()),
            WindowRuleProp::NotModal => Some("off".to_string()),
            WindowRuleProp::FullscreenStateClient(state) => Some(state.to_string()),
            WindowRuleProp::FullscreenStateInternal(state) => Some(state.to_string()),
            WindowRuleProp::Workspace(workspace_selector) => Some(workspace_selector.to_string()),
            WindowRuleProp::Content(content_type) => Some(u8::from(*content_type).to_string()),
            WindowRuleProp::XdgTag(tag) => Some(tag.clone()),
        }
    }
}

impl Default for WindowRuleProp {
    fn default() -> Self {
        Self::Class(String::new())
    }
}

impl FromStr for WindowRuleProp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (part1, part2) = s.trim().split_once(' ').unwrap_or((s, ""));

        match part1.trim_end() {
            "class" => Ok(WindowRuleProp::Class(part2.to_string())),
            "title" => Ok(WindowRuleProp::Title(part2.to_string())),
            "initial_class" => Ok(WindowRuleProp::InitialClass(part2.to_string())),
            "initial_title" => Ok(WindowRuleProp::InitialTitle(part2.to_string())),
            "tag" => Ok(WindowRuleProp::Tag(part2.to_string())),
            "xwayland" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleProp::Xwayland),
                Some(false) => Ok(WindowRuleProp::NotXwayland),
                None => Ok(WindowRuleProp::Xwayland),
            },
            "floating" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleProp::Floating),
                Some(false) => Ok(WindowRuleProp::NotFloating),
                None => Ok(WindowRuleProp::Floating),
            },
            "fullscreen" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleProp::Fullscreen),
                Some(false) => Ok(WindowRuleProp::NotFullscreen),
                None => Ok(WindowRuleProp::Fullscreen),
            },
            "pin" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleProp::Pin),
                Some(false) => Ok(WindowRuleProp::NotPin),
                None => Ok(WindowRuleProp::Pin),
            },
            "focus" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleProp::Focus),
                Some(false) => Ok(WindowRuleProp::NotFocus),
                None => Ok(WindowRuleProp::Focus),
            },
            "group" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleProp::Group),
                Some(false) => Ok(WindowRuleProp::NotGroup),
                None => Ok(WindowRuleProp::Group),
            },
            "modal" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleProp::Modal),
                Some(false) => Ok(WindowRuleProp::NotModal),
                None => Ok(WindowRuleProp::Modal),
            },
            "fullscreen_state_client" => Ok(WindowRuleProp::FullscreenStateClient(
                part2.parse().unwrap_or_default(),
            )),
            "fullscreen_state_internal" => Ok(WindowRuleProp::FullscreenStateInternal(
                part2.parse().unwrap_or_default(),
            )),
            "workspace" => Ok(WindowRuleProp::Workspace(part2.parse().unwrap_or_default())),
            "content" => Ok(WindowRuleProp::Content(part2.parse().unwrap_or_default())),
            "xdg_tag" => Ok(WindowRuleProp::XdgTag(part2.to_string())),
            _ => Err(()),
        }
    }
}

impl Display for WindowRuleProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRuleProp::Class(class) => write!(f, "class {}", class),
            WindowRuleProp::Title(title) => write!(f, "title {}", title),
            WindowRuleProp::InitialClass(initial_class) => {
                write!(f, "initialClass {}", initial_class)
            }
            WindowRuleProp::InitialTitle(initial_title) => {
                write!(f, "initialTitle {}", initial_title)
            }
            WindowRuleProp::Tag(tag) => write!(f, "tag {}", tag),
            WindowRuleProp::Xwayland => write!(f, "xwayland on"),
            WindowRuleProp::NotXwayland => write!(f, "xwayland off"),
            WindowRuleProp::Floating => write!(f, "floating on"),
            WindowRuleProp::NotFloating => write!(f, "floating off"),
            WindowRuleProp::Fullscreen => write!(f, "fullscreen on"),
            WindowRuleProp::NotFullscreen => write!(f, "fullscreen off"),
            WindowRuleProp::Pin => write!(f, "pin on"),
            WindowRuleProp::NotPin => write!(f, "pin off"),
            WindowRuleProp::Focus => write!(f, "focus on"),
            WindowRuleProp::NotFocus => write!(f, "focus off"),
            WindowRuleProp::Group => write!(f, "group on"),
            WindowRuleProp::NotGroup => write!(f, "group off"),
            WindowRuleProp::Modal => write!(f, "modal on"),
            WindowRuleProp::NotModal => write!(f, "modal off"),
            WindowRuleProp::FullscreenStateClient(state) => {
                write!(f, "fullscreen_state_client {}", state)
            }
            WindowRuleProp::FullscreenStateInternal(state) => {
                write!(f, "fullscreen_state_internal {}", state)
            }
            WindowRuleProp::Workspace(workspace) => write!(f, "workspace {}", workspace),
            WindowRuleProp::Content(content_type) => {
                write!(f, "content {}", u8::from(*content_type))
            }
            WindowRuleProp::XdgTag(tag) => write!(f, "xdgtag {}", tag),
        }
    }
}

impl EnumConfigForGtk for WindowRuleProp {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.window_rule_prop.rs.class"),
            &t!("hyprland.window_rule_prop.rs.title"),
            &t!("hyprland.window_rule_prop.rs.initial_class"),
            &t!("hyprland.window_rule_prop.rs.initial_title"),
            &t!("hyprland.window_rule_prop.rs.tag"),
            &t!("hyprland.window_rule_prop.rs.xwayland"),
            &t!("hyprland.window_rule_prop.rs.not_xwayland"),
            &t!("hyprland.window_rule_prop.rs.floating"),
            &t!("hyprland.window_rule_prop.rs.not_floating"),
            &t!("hyprland.window_rule_prop.rs.fullscreen"),
            &t!("hyprland.window_rule_prop.rs.not_fullscreen"),
            &t!("hyprland.window_rule_prop.rs.pin"),
            &t!("hyprland.window_rule_prop.rs.not_pin"),
            &t!("hyprland.window_rule_prop.rs.focus"),
            &t!("hyprland.window_rule_prop.rs.not_focus"),
            &t!("hyprland.window_rule_prop.rs.group"),
            &t!("hyprland.window_rule_prop.rs.not_group"),
            &t!("hyprland.window_rule_prop.rs.modal"),
            &t!("hyprland.window_rule_prop.rs.not_modal"),
            &t!("hyprland.window_rule_prop.rs.fullscreen_state_client"),
            &t!("hyprland.window_rule_prop.rs.fullscreen_state_internal"),
            &t!("hyprland.window_rule_prop.rs.workspace"),
            &t!("hyprland.window_rule_prop.rs.content"),
            &t!("hyprland.window_rule_prop.rs.xdg_tag"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::Class(_class) => Some(<(String,)>::to_gtk_box),
            Self::Title(_title) => Some(<(String,)>::to_gtk_box),
            Self::InitialClass(_class) => Some(<(String,)>::to_gtk_box),
            Self::InitialTitle(_title) => Some(<(String,)>::to_gtk_box),
            Self::Tag(_) => Some(<(String,)>::to_gtk_box),
            Self::Xwayland => None,
            Self::NotXwayland => None,
            Self::Floating => None,
            Self::NotFloating => None,
            Self::Fullscreen => None,
            Self::NotFullscreen => None,
            Self::Pin => None,
            Self::NotPin => None,
            Self::Focus => None,
            Self::NotFocus => None,
            Self::Group => None,
            Self::NotGroup => None,
            Self::Modal => None,
            Self::NotModal => None,
            Self::FullscreenStateClient(_state) => Some(<(WindowRuleFullscreenState,)>::to_gtk_box),
            Self::FullscreenStateInternal(_state) => {
                Some(<(WindowRuleFullscreenState,)>::to_gtk_box)
            }
            Self::Workspace(_) => Some(<(IdOrNameOrWorkspaceSelector,)>::to_gtk_box),
            Self::Content(_) => Some(<(ContentTypeInt,)>::to_gtk_box),
            Self::XdgTag(_) => Some(<(String,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(WindowRuleProp);
register_togtkbox_with_separator_names!((WindowRuleFullscreenState,));
