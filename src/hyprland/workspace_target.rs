use super::RelativeId;
use crate::{
    gtk_converters::{
        EnumConfigForGtk, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorkspaceTargetDiscriminant))]
pub enum WorkspaceTarget {
    Id(u32),
    Relative(i32),
    OnMonitor(RelativeId),
    OnMonitorIncludingEmptyWorkspace(RelativeId),
    Open(RelativeId),
    Name(String),
    Previous,
    PreviousPerMonitor,
    FirstAvailableEmptyWorkspace,
    NextAvailableEmptyWorkspace,
    FirstAvailableEmptyWorkspaceOnMonitor,
    NextAvailableEmptyWorkspaceOnMonitor,
    Special,
    SpecialWithName(String),
}

impl HasDiscriminant for WorkspaceTarget {
    type Discriminant = WorkspaceTargetDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(1),
            Self::Discriminant::Relative => Self::Relative(0),
            Self::Discriminant::OnMonitor => Self::OnMonitor(RelativeId::default()),
            Self::Discriminant::OnMonitorIncludingEmptyWorkspace => {
                Self::OnMonitorIncludingEmptyWorkspace(RelativeId::default())
            }
            Self::Discriminant::Open => Self::Open(RelativeId::default()),
            Self::Discriminant::Name => Self::Name("".to_string()),
            Self::Discriminant::Previous => Self::Previous,
            Self::Discriminant::PreviousPerMonitor => Self::PreviousPerMonitor,
            Self::Discriminant::FirstAvailableEmptyWorkspace => Self::FirstAvailableEmptyWorkspace,
            Self::Discriminant::NextAvailableEmptyWorkspace => Self::NextAvailableEmptyWorkspace,
            Self::Discriminant::FirstAvailableEmptyWorkspaceOnMonitor => {
                Self::FirstAvailableEmptyWorkspaceOnMonitor
            }
            Self::Discriminant::NextAvailableEmptyWorkspaceOnMonitor => {
                Self::NextAvailableEmptyWorkspaceOnMonitor
            }
            Self::Discriminant::Special => Self::Special,
            Self::Discriminant::SpecialWithName => Self::SpecialWithName("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(str.parse().unwrap_or(1)),
            Self::Discriminant::Relative => Self::Relative(str.parse().unwrap_or(0)),
            Self::Discriminant::OnMonitor => {
                Self::OnMonitor(str.parse().unwrap_or(RelativeId::default()))
            }
            Self::Discriminant::OnMonitorIncludingEmptyWorkspace => {
                Self::OnMonitorIncludingEmptyWorkspace(str.parse().unwrap_or(RelativeId::default()))
            }
            Self::Discriminant::Open => Self::Open(str.parse().unwrap_or(RelativeId::default())),
            Self::Discriminant::Name => Self::Name(str.to_string()),
            Self::Discriminant::Previous => Self::Previous,
            Self::Discriminant::PreviousPerMonitor => Self::PreviousPerMonitor,
            Self::Discriminant::FirstAvailableEmptyWorkspace => Self::FirstAvailableEmptyWorkspace,
            Self::Discriminant::NextAvailableEmptyWorkspace => Self::NextAvailableEmptyWorkspace,
            Self::Discriminant::FirstAvailableEmptyWorkspaceOnMonitor => {
                Self::FirstAvailableEmptyWorkspaceOnMonitor
            }
            Self::Discriminant::NextAvailableEmptyWorkspaceOnMonitor => {
                Self::NextAvailableEmptyWorkspaceOnMonitor
            }
            Self::Discriminant::Special => Self::Special,
            Self::Discriminant::SpecialWithName => Self::SpecialWithName(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WorkspaceTarget::Id(id) => Some(id.to_string()),
            WorkspaceTarget::Relative(rel_id) => Some(format!("{:+}", rel_id)),
            WorkspaceTarget::OnMonitor(rel_id) => Some(rel_id.to_string()),
            WorkspaceTarget::OnMonitorIncludingEmptyWorkspace(rel_id) => Some(rel_id.to_string()),
            WorkspaceTarget::Open(rel_id) => Some(rel_id.to_string()),
            WorkspaceTarget::Name(name) => Some(name.clone()),
            WorkspaceTarget::Previous => None,
            WorkspaceTarget::PreviousPerMonitor => None,
            WorkspaceTarget::FirstAvailableEmptyWorkspace => None,
            WorkspaceTarget::NextAvailableEmptyWorkspace => None,
            WorkspaceTarget::FirstAvailableEmptyWorkspaceOnMonitor => None,
            WorkspaceTarget::NextAvailableEmptyWorkspaceOnMonitor => None,
            WorkspaceTarget::Special => None,
            WorkspaceTarget::SpecialWithName(name) => Some(name.clone()),
        }
    }
}

impl Default for WorkspaceTarget {
    fn default() -> Self {
        WorkspaceTarget::Id(1)
    }
}

impl FromStr for WorkspaceTarget {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Ok(id) = s.parse::<u32>() {
            let id = match id {
                0 => 1,
                id => id,
            };
            Ok(WorkspaceTarget::Id(id))
        } else if let Ok(rel_id) = s.parse::<i32>() {
            Ok(WorkspaceTarget::Relative(rel_id))
        } else if let Some(s) = s.strip_prefix("m") {
            Ok(WorkspaceTarget::OnMonitor(s.parse().unwrap_or_default()))
        } else if let Some(s) = s.strip_prefix("r") {
            Ok(WorkspaceTarget::OnMonitorIncludingEmptyWorkspace(
                s.parse().unwrap_or_default(),
            ))
        } else if let Some(s) = s.strip_prefix("e") {
            Ok(WorkspaceTarget::Open(s.parse().unwrap_or_default()))
        } else if let Some(s) = s.strip_prefix("name:") {
            Ok(WorkspaceTarget::Name(
                s.trim_start_matches("name:").to_string(),
            ))
        } else if s == "previous" {
            Ok(WorkspaceTarget::Previous)
        } else if s == "previous_per_monitor" {
            Ok(WorkspaceTarget::PreviousPerMonitor)
        } else if s == "empty" {
            Ok(WorkspaceTarget::FirstAvailableEmptyWorkspace)
        } else if s == "emptyn" {
            Ok(WorkspaceTarget::NextAvailableEmptyWorkspace)
        } else if s == "emptym" {
            Ok(WorkspaceTarget::FirstAvailableEmptyWorkspaceOnMonitor)
        } else if s == "emptymn" || s == "emptynm" {
            Ok(WorkspaceTarget::NextAvailableEmptyWorkspaceOnMonitor)
        } else if s == "special" {
            Ok(WorkspaceTarget::Special)
        } else if let Some(s) = s.strip_prefix("special:") {
            Ok(WorkspaceTarget::SpecialWithName(
                s.trim_start_matches("special:").to_string(),
            ))
        } else {
            Err(())
        }
    }
}

impl Display for WorkspaceTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceTarget::Id(id) => write!(f, "{}", id),
            WorkspaceTarget::Relative(rel_id) => write!(f, "{:+}", rel_id),
            WorkspaceTarget::OnMonitor(rel_id) => write!(f, "m{}", rel_id),
            WorkspaceTarget::OnMonitorIncludingEmptyWorkspace(rel_id) => write!(f, "r{}", rel_id),
            WorkspaceTarget::Open(rel_id) => write!(f, "e{}", rel_id),
            WorkspaceTarget::Name(name) => write!(f, "name:{}", name),
            WorkspaceTarget::Previous => write!(f, "previous"),
            WorkspaceTarget::PreviousPerMonitor => write!(f, "previous_per_monitor"),
            WorkspaceTarget::FirstAvailableEmptyWorkspace => write!(f, "empty"),
            WorkspaceTarget::NextAvailableEmptyWorkspace => write!(f, "emptyn"),
            WorkspaceTarget::FirstAvailableEmptyWorkspaceOnMonitor => write!(f, "emptym"),
            WorkspaceTarget::NextAvailableEmptyWorkspaceOnMonitor => write!(f, "emptynm"),
            WorkspaceTarget::Special => write!(f, "special"),
            WorkspaceTarget::SpecialWithName(name) => write!(f, "special:{}", name),
        }
    }
}

impl EnumConfigForGtk for WorkspaceTarget {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.id"),
            &t!("gtk_converters.relative"),
            &t!("gtk_converters.on_monitor"),
            &t!("gtk_converters.on_monitor_including_empty_workspace"),
            &t!("gtk_converters.open"),
            &t!("gtk_converters.name"),
            &t!("gtk_converters.previous"),
            &t!("gtk_converters.previous_per_monitor"),
            &t!("gtk_converters.first_available_empty_workspace"),
            &t!("gtk_converters.next_available_empty_workspace"),
            &t!("gtk_converters.first_available_empty_workspace_on_monitor"),
            &t!("gtk_converters.next_available_empty_workspace_on_monitor"),
            &t!("gtk_converters.special"),
            &t!("gtk_converters.special_with_name"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            WorkspaceTarget::Id(_id) => Some(<(u32,)>::to_gtk_box),
            WorkspaceTarget::Relative(_relative) => Some(<(i32,)>::to_gtk_box),
            WorkspaceTarget::OnMonitor(_rel_id) => Some(<(RelativeId,)>::to_gtk_box),
            WorkspaceTarget::OnMonitorIncludingEmptyWorkspace(_rel_id) => {
                Some(<(RelativeId,)>::to_gtk_box)
            }
            WorkspaceTarget::Open(_rel_id) => Some(<(bool,)>::to_gtk_box),
            WorkspaceTarget::Name(_name) => Some(<(String,)>::to_gtk_box),
            WorkspaceTarget::Previous => None,
            WorkspaceTarget::PreviousPerMonitor => None,
            WorkspaceTarget::FirstAvailableEmptyWorkspace => None,
            WorkspaceTarget::NextAvailableEmptyWorkspace => None,
            WorkspaceTarget::FirstAvailableEmptyWorkspaceOnMonitor => None,
            WorkspaceTarget::NextAvailableEmptyWorkspaceOnMonitor => None,
            WorkspaceTarget::Special => None,
            WorkspaceTarget::SpecialWithName(_name) => Some(<(String,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(WorkspaceTarget);
register_togtkbox_with_separator_names!((u32,), (i32,), (RelativeId,), (String,),);
