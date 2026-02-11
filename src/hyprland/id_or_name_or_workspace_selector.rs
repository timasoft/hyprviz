use super::{WorkspaceSelector, workspace_type::parse_workspace_selector};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, ToGtkBoxWithSeparator, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, join_with_separator},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(IdOrNameOrWorkspaceSelectorDiscriminant))]
pub enum IdOrNameOrWorkspaceSelector {
    Id(u32),
    Name(String),
    WorkspaceSelector(Vec<WorkspaceSelector>),
}

impl HasDiscriminant for IdOrNameOrWorkspaceSelector {
    type Discriminant = IdOrNameOrWorkspaceSelectorDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(1),
            Self::Discriminant::Name => Self::Name("".to_string()),
            Self::Discriminant::WorkspaceSelector => Self::WorkspaceSelector(Vec::new()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(str.parse().unwrap_or_default()),
            Self::Discriminant::Name => Self::Name(str.to_string()),
            Self::Discriminant::WorkspaceSelector => {
                Self::WorkspaceSelector(parse_workspace_selector(str))
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            IdOrNameOrWorkspaceSelector::Id(id) => Some(id.to_string()),
            IdOrNameOrWorkspaceSelector::Name(name) => Some(name.clone()),
            IdOrNameOrWorkspaceSelector::WorkspaceSelector(workspace_selector) => {
                Some(join_with_separator(workspace_selector, ""))
            }
        }
    }
}

impl Default for IdOrNameOrWorkspaceSelector {
    fn default() -> Self {
        IdOrNameOrWorkspaceSelector::Id(1)
    }
}

impl FromStr for IdOrNameOrWorkspaceSelector {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(name) = s.strip_prefix("name:") {
            Ok(IdOrNameOrWorkspaceSelector::Name(name.to_string()))
        } else if let Ok(id) = s.parse::<u32>() {
            Ok(IdOrNameOrWorkspaceSelector::Id(id))
        } else {
            Ok(IdOrNameOrWorkspaceSelector::WorkspaceSelector(
                parse_workspace_selector(s),
            ))
        }
    }
}

impl Display for IdOrNameOrWorkspaceSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdOrNameOrWorkspaceSelector::Id(id) => write!(f, "{id}"),
            IdOrNameOrWorkspaceSelector::Name(name) => write!(f, "name:{name}"),
            IdOrNameOrWorkspaceSelector::WorkspaceSelector(workspace_selector) => {
                write!(f, "{}", join_with_separator(workspace_selector, ""))
            }
        }
    }
}

impl EnumConfigForGtk for IdOrNameOrWorkspaceSelector {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.id_or_name_or_workspace_selector.id"),
            &t!("hyprland.id_or_name_or_workspace_selector.name"),
            &t!("hyprland.id_or_name_or_workspace_selector.workspace_selector"),
        ])
    }

    const SEPARATOR: Option<char> = Some(',');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            IdOrNameOrWorkspaceSelector::Id(_id) => Some(<(u32,)>::to_gtk_box),
            IdOrNameOrWorkspaceSelector::Name(_name) => Some(<(String,)>::to_gtk_box),
            IdOrNameOrWorkspaceSelector::WorkspaceSelector(_workspace_selector) => {
                Some(|entry, _char, _names, _| Vec::<WorkspaceSelector>::to_gtk_box(entry, ' '))
            }
        }
    }
}

register_togtkbox!(IdOrNameOrWorkspaceSelector);
register_togtkbox_with_separator!(Vec<WorkspaceSelector>);
register_togtkbox_with_separator_names!((u32,), (String,));
