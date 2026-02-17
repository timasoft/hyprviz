use super::{ContentType, IdOrName, IdOrNameOrWorkspaceSelector, WindowRuleFullscreenState};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, cow_to_static_str, parse_bool},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WindowRuleParameterDiscriminant))]
pub enum WindowRuleParameter {
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
    Pinned,
    NotPinned,
    Focus,
    NotFocus,
    Group,
    NotGroup,
    Modal,
    NotModal,
    FullscreenState(WindowRuleFullscreenState, WindowRuleFullscreenState),
    Workspace(IdOrName),
    OnWorkspace(IdOrNameOrWorkspaceSelector),
    Content(ContentType),
    XdgTag(String),
}

impl HasDiscriminant for WindowRuleParameter {
    type Discriminant = WindowRuleParameterDiscriminant;

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
            Self::Discriminant::Pinned => Self::Pinned,
            Self::Discriminant::NotPinned => Self::NotPinned,
            Self::Discriminant::Focus => Self::Focus,
            Self::Discriminant::NotFocus => Self::NotFocus,
            Self::Discriminant::Group => Self::Group,
            Self::Discriminant::NotGroup => Self::NotGroup,
            Self::Discriminant::Modal => Self::Modal,
            Self::Discriminant::NotModal => Self::NotModal,
            Self::Discriminant::FullscreenState => Self::FullscreenState(
                WindowRuleFullscreenState::default(),
                WindowRuleFullscreenState::default(),
            ),
            Self::Discriminant::Workspace => Self::Workspace(IdOrName::default()),
            Self::Discriminant::OnWorkspace => {
                Self::OnWorkspace(IdOrNameOrWorkspaceSelector::default())
            }
            Self::Discriminant::Content => Self::Content(ContentType::default()),
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
            Self::Discriminant::Pinned => Self::Pinned,
            Self::Discriminant::NotPinned => Self::NotPinned,
            Self::Discriminant::Focus => Self::Focus,
            Self::Discriminant::NotFocus => Self::NotFocus,
            Self::Discriminant::Group => Self::Group,
            Self::Discriminant::NotGroup => Self::NotGroup,
            Self::Discriminant::Modal => Self::Modal,
            Self::Discriminant::NotModal => Self::NotModal,
            Self::Discriminant::FullscreenState => {
                let (state1, state2) = str.split_once(' ').unwrap_or((str, ""));
                Self::FullscreenState(
                    state1.parse().unwrap_or_default(),
                    state2.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::Workspace => Self::Workspace(str.parse().unwrap_or_default()),
            Self::Discriminant::OnWorkspace => Self::OnWorkspace(str.parse().unwrap_or_default()),
            Self::Discriminant::Content => Self::Content(str.parse().unwrap_or_default()),
            Self::Discriminant::XdgTag => Self::XdgTag(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WindowRuleParameter::Class(class) => Some(class.clone()),
            WindowRuleParameter::Title(title) => Some(title.clone()),
            WindowRuleParameter::InitialClass(initial_class) => Some(initial_class.clone()),
            WindowRuleParameter::InitialTitle(initial_title) => Some(initial_title.clone()),
            WindowRuleParameter::Tag(tag) => Some(tag.clone()),
            WindowRuleParameter::Xwayland => Some("1".to_string()),
            WindowRuleParameter::NotXwayland => Some("0".to_string()),
            WindowRuleParameter::Floating => Some("1".to_string()),
            WindowRuleParameter::NotFloating => Some("0".to_string()),
            WindowRuleParameter::Fullscreen => Some("1".to_string()),
            WindowRuleParameter::NotFullscreen => Some("0".to_string()),
            WindowRuleParameter::Pinned => Some("1".to_string()),
            WindowRuleParameter::NotPinned => Some("0".to_string()),
            WindowRuleParameter::Focus => Some("1".to_string()),
            WindowRuleParameter::NotFocus => Some("0".to_string()),
            WindowRuleParameter::Group => Some("1".to_string()),
            WindowRuleParameter::NotGroup => Some("0".to_string()),
            WindowRuleParameter::Modal => Some("1".to_string()),
            WindowRuleParameter::NotModal => Some("0".to_string()),
            WindowRuleParameter::FullscreenState(state1, state2) => {
                Some(format!("{} {}", state1, state2))
            }
            WindowRuleParameter::Workspace(workspace) => Some(workspace.to_string()),
            WindowRuleParameter::OnWorkspace(workspace_selector) => {
                Some(workspace_selector.to_string())
            }
            WindowRuleParameter::Content(content_type) => Some(content_type.to_string()),
            WindowRuleParameter::XdgTag(tag) => Some(tag.clone()),
        }
    }
}

impl Default for WindowRuleParameter {
    fn default() -> Self {
        Self::Class(String::new())
    }
}

impl FromStr for WindowRuleParameter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (part1, part2) = s.split_once(':').unwrap_or((s, ""));

        match part1.trim() {
            "class" => Ok(WindowRuleParameter::Class(part2.to_string())),
            "title" => Ok(WindowRuleParameter::Title(part2.to_string())),
            "initialClass" => Ok(WindowRuleParameter::InitialClass(part2.to_string())),
            "initialTitle" => Ok(WindowRuleParameter::InitialTitle(part2.to_string())),
            "tag" => Ok(WindowRuleParameter::Tag(part2.to_string())),
            "xwayland" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Xwayland),
                Some(false) => Ok(WindowRuleParameter::NotXwayland),
                None => Ok(WindowRuleParameter::Xwayland),
            },
            "floating" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Floating),
                Some(false) => Ok(WindowRuleParameter::NotFloating),
                None => Ok(WindowRuleParameter::Floating),
            },
            "fullscreen" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Fullscreen),
                Some(false) => Ok(WindowRuleParameter::NotFullscreen),
                None => Ok(WindowRuleParameter::Fullscreen),
            },
            "pinned" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Pinned),
                Some(false) => Ok(WindowRuleParameter::NotPinned),
                None => Ok(WindowRuleParameter::Pinned),
            },
            "focus" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Focus),
                Some(false) => Ok(WindowRuleParameter::NotFocus),
                None => Ok(WindowRuleParameter::Focus),
            },
            "group" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Group),
                Some(false) => Ok(WindowRuleParameter::NotGroup),
                None => Ok(WindowRuleParameter::Group),
            },
            "modal" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Modal),
                Some(false) => Ok(WindowRuleParameter::NotModal),
                None => Ok(WindowRuleParameter::Modal),
            },
            "fullscreenState" => {
                let (state1, state2) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRuleParameter::FullscreenState(
                    state1.parse().unwrap_or_default(),
                    state2.parse().unwrap_or_default(),
                ))
            }
            "workspace" => Ok(WindowRuleParameter::Workspace(
                part2.parse().unwrap_or_default(),
            )),
            "onworkspace" => Ok(WindowRuleParameter::OnWorkspace(
                part2.parse().unwrap_or_default(),
            )),
            "content" => Ok(WindowRuleParameter::Content(
                part2.parse().unwrap_or_default(),
            )),
            "xdgtag" => Ok(WindowRuleParameter::XdgTag(part2.to_string())),
            _ => Err(()),
        }
    }
}

impl Display for WindowRuleParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRuleParameter::Class(class) => write!(f, "class:{}", class),
            WindowRuleParameter::Title(title) => write!(f, "title:{}", title),
            WindowRuleParameter::InitialClass(initial_class) => {
                write!(f, "initialClass:{}", initial_class)
            }
            WindowRuleParameter::InitialTitle(initial_title) => {
                write!(f, "initialTitle:{}", initial_title)
            }
            WindowRuleParameter::Tag(tag) => write!(f, "tag:{}", tag),
            WindowRuleParameter::Xwayland => write!(f, "xwayland:1"),
            WindowRuleParameter::NotXwayland => write!(f, "xwayland:0"),
            WindowRuleParameter::Floating => write!(f, "floating:1"),
            WindowRuleParameter::NotFloating => write!(f, "floating:0"),
            WindowRuleParameter::Fullscreen => write!(f, "fullscreen:1"),
            WindowRuleParameter::NotFullscreen => write!(f, "fullscreen:0"),
            WindowRuleParameter::Pinned => write!(f, "pinned:1"),
            WindowRuleParameter::NotPinned => write!(f, "pinned:0"),
            WindowRuleParameter::Focus => write!(f, "focus:1"),
            WindowRuleParameter::NotFocus => write!(f, "focus:0"),
            WindowRuleParameter::Group => write!(f, "group:1"),
            WindowRuleParameter::NotGroup => write!(f, "group:0"),
            WindowRuleParameter::Modal => write!(f, "modal:1"),
            WindowRuleParameter::NotModal => write!(f, "modal:0"),
            WindowRuleParameter::FullscreenState(state1, state2) => {
                write!(f, "fullscreenState:{} {}", state1, state2)
            }
            WindowRuleParameter::Workspace(workspace) => write!(f, "workspace:{}", workspace),
            WindowRuleParameter::OnWorkspace(workspace_selector) => {
                write!(f, "onworkspace:{}", workspace_selector)
            }
            WindowRuleParameter::Content(content_type) => write!(f, "content:{}", content_type),
            WindowRuleParameter::XdgTag(tag) => write!(f, "xdgtag:{}", tag),
        }
    }
}

impl EnumConfigForGtk for WindowRuleParameter {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.window_rule_parameter.class"),
            &t!("hyprland.window_rule_parameter.title"),
            &t!("hyprland.window_rule_parameter.initial_class"),
            &t!("hyprland.window_rule_parameter.initial_title"),
            &t!("hyprland.window_rule_parameter.tag"),
            &t!("hyprland.window_rule_parameter.xwayland"),
            &t!("hyprland.window_rule_parameter.not_xwayland"),
            &t!("hyprland.window_rule_parameter.floating"),
            &t!("hyprland.window_rule_parameter.not_floating"),
            &t!("hyprland.window_rule_parameter.fullscreen"),
            &t!("hyprland.window_rule_parameter.not_fullscreen"),
            &t!("hyprland.window_rule_parameter.pinned"),
            &t!("hyprland.window_rule_parameter.not_pinned"),
            &t!("hyprland.window_rule_parameter.focus"),
            &t!("hyprland.window_rule_parameter.not_focus"),
            &t!("hyprland.window_rule_parameter.group"),
            &t!("hyprland.window_rule_parameter.not_group"),
            &t!("hyprland.window_rule_parameter.modal"),
            &t!("hyprland.window_rule_parameter.not_modal"),
            &t!("hyprland.window_rule_parameter.fullscreen_state"),
            &t!("hyprland.window_rule_parameter.workspace"),
            &t!("hyprland.window_rule_parameter.on_workspace"),
            &t!("hyprland.window_rule_parameter.content"),
            &t!("hyprland.window_rule_parameter.xdg_tag"),
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
            Self::Pinned => None,
            Self::NotPinned => None,
            Self::Focus => None,
            Self::NotFocus => None,
            Self::Group => None,
            Self::NotGroup => None,
            Self::Modal => None,
            Self::NotModal => None,
            Self::FullscreenState(_state, _state2) => {
                Some(<(WindowRuleFullscreenState,)>::to_gtk_box)
            }
            Self::Workspace(_) => Some(<(IdOrName,)>::to_gtk_box),
            Self::OnWorkspace(_) => Some(<(IdOrNameOrWorkspaceSelector,)>::to_gtk_box),
            Self::Content(_) => Some(<(ContentType,)>::to_gtk_box),
            Self::XdgTag(_) => Some(<(String,)>::to_gtk_box),
        }
    }

    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        Some(vec![
            // Class(String),
            vec![],
            // Title(String),
            vec![],
            // InitialClass(String),
            vec![],
            // InitialTitle(String),
            vec![],
            // Tag(String),
            vec![],
            // Xwayland,
            vec![],
            // NotXwayland,
            vec![],
            // Floating,
            vec![],
            // NotFloating,
            vec![],
            // Fullscreen,
            vec![],
            // NotFullscreen,
            vec![],
            // Pinned,
            vec![],
            // NotPinned,
            vec![],
            // Focus,
            vec![],
            // NotFocus,
            vec![],
            // Group,
            vec![],
            // NotGroup,
            vec![],
            // Modal,
            vec![],
            // NotModal,
            vec![],
            // FullscreenState(WindowRuleFullscreenState, WindowRuleFullscreenState),
            vec![
                FieldLabel::Named(cow_to_static_str(t!(
                    "hyprland.window_rule_parameter.internal_state"
                ))),
                FieldLabel::Named(cow_to_static_str(t!(
                    "hyprland.window_rule_parameter.client_state"
                ))),
            ],
            // Workspace(IdOrName),
            // OnWorkspace(IdOrNameOrWorkspaceSelector),
            // Content(ContentType),
            // XdgTag(String),
        ])
    }
}

register_togtkbox!(WindowRuleParameter);
register_togtkbox_with_separator_names!(
    (WindowRuleFullscreenState, WindowRuleFullscreenState),
    (IdOrNameOrWorkspaceSelector,),
);
