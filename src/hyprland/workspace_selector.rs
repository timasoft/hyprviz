use super::{
    MonitorSelector, Range, WorkspaceSelectorFullscreen, WorkspaceSelectorNamed,
    WorkspaceSelectorWindowCount,
};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, find_matching_bracket, parse_bool},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorkspaceSelectorDiscriminant))]
pub enum WorkspaceSelector {
    #[default]
    None,
    Range(Range),
    Special(bool),
    Named(WorkspaceSelectorNamed),
    Monitor(MonitorSelector),
    WindowCount(WorkspaceSelectorWindowCount),
    Fullscreen(WorkspaceSelectorFullscreen),
}

impl WorkspaceSelector {
    pub fn get_fancy_list() -> [String; 7] {
        [
            t!("hyprland.workspace_selector.none").to_string(),
            t!("hyprland.workspace_selector.range").to_string(),
            t!("hyprland.workspace_selector.special").to_string(),
            t!("hyprland.workspace_selector.named").to_string(),
            t!("hyprland.workspace_selector.monitor").to_string(),
            t!("hyprland.workspace_selector.window_count").to_string(),
            t!("hyprland.workspace_selector.fullscreen").to_string(),
        ]
    }
}

impl HasDiscriminant for WorkspaceSelector {
    type Discriminant = WorkspaceSelectorDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::None => Self::None,
            Self::Discriminant::Range => Self::Range(Range::default()),
            Self::Discriminant::Special => Self::Special(false),
            Self::Discriminant::Named => Self::Named(WorkspaceSelectorNamed::default()),
            Self::Discriminant::Monitor => Self::Monitor(MonitorSelector::default()),
            Self::Discriminant::WindowCount => {
                Self::WindowCount(WorkspaceSelectorWindowCount::default())
            }
            Self::Discriminant::Fullscreen => {
                Self::Fullscreen(WorkspaceSelectorFullscreen::default())
            }
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::None => Self::None,
            Self::Discriminant::Range => Self::Range(Range::from_str(str).unwrap_or_default()),
            Self::Discriminant::Special => Self::Special(str.parse().unwrap_or_default()),
            Self::Discriminant::Named => {
                Self::Named(WorkspaceSelectorNamed::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Monitor => {
                Self::Monitor(MonitorSelector::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::WindowCount => {
                Self::WindowCount(WorkspaceSelectorWindowCount::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Fullscreen => {
                Self::Fullscreen(WorkspaceSelectorFullscreen::from_str(str).unwrap_or_default())
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Range(Range { start, end }) => Some(format!("[{}-{}]", start, end)),
            Self::Special(is_special) => Some(format!("[{}]", is_special)),
            Self::Named(named) => Some(format!("[{}]", named)),
            Self::Monitor(monitor) => Some(format!("[{}]", monitor)),
            Self::WindowCount(window_count) => Some(format!("[{}]", window_count)),
            Self::Fullscreen(state) => Some(format!("[{}]", state)),
        }
    }
}

impl FromStr for WorkspaceSelector {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        match parse_single_selector(s) {
            Some((selector, _)) => Ok(selector),
            None => Err(()),
        }
    }
}

impl Display for WorkspaceSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSelector::None => write!(f, ""),
            WorkspaceSelector::Range(range) => write!(f, "r[{}]", range),
            WorkspaceSelector::Special(is_special) => {
                write!(f, "s[{}]", is_special)
            }
            WorkspaceSelector::Named(named) => {
                write!(f, "n[{}]", named)
            }
            WorkspaceSelector::Monitor(monitor) => write!(f, "m[{}]", monitor),
            WorkspaceSelector::WindowCount(window_count) => {
                write!(f, "w[{}]", window_count)
            }
            WorkspaceSelector::Fullscreen(state) => write!(f, "f[{}]", state),
        }
    }
}

pub fn parse_single_selector(input: &str) -> Option<(WorkspaceSelector, &str)> {
    if input.starts_with("r[") {
        if let Some(end_idx) = find_matching_bracket(input, "r[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let selector = match Range::from_str(content) {
                Ok(range) => WorkspaceSelector::Range(range),
                Err(_) => return None,
            };

            return Some((selector, rest));
        }
    } else if input.starts_with("s[") {
        if let Some(end_idx) = find_matching_bracket(input, "s[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let is_special = parse_bool(content).unwrap_or(false);
            return Some((WorkspaceSelector::Special(is_special), rest));
        }
    } else if input.starts_with("n[") {
        if let Some(end_idx) = find_matching_bracket(input, "n[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let selector = match WorkspaceSelectorNamed::from_str(content) {
                Ok(selector) => WorkspaceSelector::Named(selector),
                Err(_) => return None,
            };

            return Some((selector, rest));
        }
    } else if input.starts_with("m[") {
        if let Some(end_idx) = find_matching_bracket(input, "m[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let monitor = content.to_string();
            return Some((
                WorkspaceSelector::Monitor(MonitorSelector::from_str(&monitor).unwrap_or_default()),
                rest,
            ));
        }
    } else if input.starts_with("w[") {
        if let Some(end_idx) = find_matching_bracket(input, "w[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let selector = match WorkspaceSelectorWindowCount::from_str(content) {
                Ok(selector) => WorkspaceSelector::WindowCount(selector),
                Err(_) => return None,
            };

            return Some((selector, rest));
        }
    } else if input.starts_with("f[")
        && let Some(end_idx) = find_matching_bracket(input, "f[", ']')
    {
        let content = &input[2..end_idx];
        let rest = &input[end_idx + 1..];

        if let Ok(state) = content.parse::<WorkspaceSelectorFullscreen>() {
            return Some((WorkspaceSelector::Fullscreen(state), rest));
        }
    }

    None
}

impl EnumConfigForGtk for WorkspaceSelector {
    fn dropdown_items() -> StringList {
        let list = WorkspaceSelector::get_fancy_list();

        StringList::new(&list.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::None => None,
            Self::Range(_range) => Some(<(Range,)>::to_gtk_box),
            Self::Special(_is_special) => Some(<(bool,)>::to_gtk_box),
            Self::Named(_workspace_selector_named) => Some(<(WorkspaceSelectorNamed,)>::to_gtk_box),
            Self::Monitor(_monitor_selector) => Some(<(MonitorSelector,)>::to_gtk_box),
            Self::WindowCount(_workspace_selector_window_count) => {
                Some(<(WorkspaceSelectorWindowCount,)>::to_gtk_box)
            }
            Self::Fullscreen(_workspace_selector_fullscreen) => {
                Some(<(WorkspaceSelectorFullscreen,)>::to_gtk_box)
            }
        }
    }
}

register_togtkbox!(WorkspaceSelector);
register_togtkbox_with_separator_names!(
    (Range,),
    (WorkspaceSelectorNamed,),
    (MonitorSelector,),
    (WorkspaceSelectorWindowCount,),
    (WorkspaceSelectorFullscreen,)
);
