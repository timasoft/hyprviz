use super::{
    ChangeGroupActive, CursorCorner, CycleNext, Direction, DispatcherFullscreenState, FloatValue,
    FullscreenMode, GroupLockAction, KeyState, Modifier, MonitorTarget, MoveDirection,
    ResizeParams, SetProp, SwapDirection, SwapNext, TagToggleState, ToggleState, WindowRule,
    WindowTarget, WorkspaceTarget, ZHeight, modifier::parse_modifiers,
};
use crate::{
    advanced_editors::create_entry,
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, ToGtkBox, ToGtkBoxWithSeparator,
        ToGtkBoxWithSeparatorAndNames, ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, cow_to_static_str, join_with_separator},
};
use gtk::{Box as GtkBox, Label, Orientation as GtkOrientation, StringList, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, collections::HashSet, fmt::Display, rc::Rc, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(DispatcherDiscriminant))]
pub enum Dispatcher {
    Exec(Vec<WindowRule>, String),
    Execr(String),
    Pass(WindowTarget),
    SendShortcut(HashSet<Modifier>, String, Option<WindowTarget>),
    SendKeyState(HashSet<Modifier>, String, KeyState, WindowTarget),
    KillActive,
    ForceKillActive,
    CloseWindow(WindowTarget),
    KillWindow(WindowTarget),
    Signal(String),
    SignalWindow(WindowTarget, String),
    Workspace(WorkspaceTarget),
    MoveToWorkspace(WorkspaceTarget, Option<WindowTarget>),
    MoveToWorkspaceSilent(WorkspaceTarget, Option<WindowTarget>),
    ToggleFloating(Option<WindowTarget>),
    SetFloating(Option<WindowTarget>),
    SetTiled(Option<WindowTarget>),
    Fullscreen(FullscreenMode),
    FullscreenState(DispatcherFullscreenState, DispatcherFullscreenState),
    Dpms(ToggleState, Option<String>),
    Pin(Option<WindowTarget>),
    MoveFocus(Direction),
    MoveWindow(MoveDirection),
    SwapWindow(SwapDirection),
    CenterWindow(bool),
    ResizeActive(ResizeParams),
    MoveActive(ResizeParams),
    ResizeWindowPixel(ResizeParams, WindowTarget),
    MoveWindowPixel(ResizeParams, WindowTarget),
    CycleNext(CycleNext),
    SwapNext(SwapNext),
    TagWindow(TagToggleState, String, Option<WindowTarget>),
    FocusWindow(WindowTarget),
    FocusMonitor(MonitorTarget),
    SplitRatio(FloatValue),
    MoveCursorToCorner(CursorCorner),
    MoveCursor(u32, u32),
    RenameWorkspace(u32, String),
    Exit,
    ForceRendererReload,
    MoveCurrentWorkspaceToMonitor(MonitorTarget),
    FocusWorkspaceOnCurrentMonitor(WorkspaceTarget),
    MoveWorkspaceToMonitor(WorkspaceTarget, MonitorTarget),
    SwapActiveWorkspaces(MonitorTarget, MonitorTarget),
    BringActiveToTop,
    AlterZOrder(ZHeight, Option<WindowTarget>),
    ToggleSpecialWorkspace(Option<String>),
    FocusUrgentOrLast,
    ToggleGroup,
    ChangeGroupActive(ChangeGroupActive),
    FocusCurrentOrLast,
    LockGroups(GroupLockAction),
    LockActiveGroup(GroupLockAction),
    MoveIntoGroup(Direction),
    MoveOutOfGroup(Option<WindowTarget>),
    MoveWindowOrGroup(Direction),
    MoveGroupWindow(bool),
    DenyWindowFromGroup(ToggleState),
    SetIgnoreGroupLock(ToggleState),
    Global(String),
    Event(String),
    SetProp(SetProp),
    ToggleSwallow,
}

impl HasDiscriminant for Dispatcher {
    type Discriminant = DispatcherDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Exec => Self::Exec(Vec::new(), "".to_string()),
            Self::Discriminant::Execr => Self::Execr("".to_string()),
            Self::Discriminant::Pass => Self::Pass(WindowTarget::default()),
            Self::Discriminant::SendShortcut => {
                Self::SendShortcut(HashSet::new(), "".to_string(), None)
            }
            Self::Discriminant::SendKeyState => Self::SendKeyState(
                HashSet::new(),
                "".to_string(),
                KeyState::default(),
                WindowTarget::default(),
            ),
            Self::Discriminant::KillActive => Self::KillActive,
            Self::Discriminant::ForceKillActive => Self::ForceKillActive,
            Self::Discriminant::CloseWindow => Self::CloseWindow(WindowTarget::default()),
            Self::Discriminant::KillWindow => Self::KillWindow(WindowTarget::default()),
            Self::Discriminant::Signal => Self::Signal("".to_string()),
            Self::Discriminant::SignalWindow => {
                Self::SignalWindow(WindowTarget::default(), "".to_string())
            }
            Self::Discriminant::Workspace => Self::Workspace(WorkspaceTarget::default()),
            Self::Discriminant::MoveToWorkspace => {
                Self::MoveToWorkspace(WorkspaceTarget::default(), None)
            }
            Self::Discriminant::MoveToWorkspaceSilent => {
                Self::MoveToWorkspaceSilent(WorkspaceTarget::default(), None)
            }
            Self::Discriminant::ToggleFloating => Self::ToggleFloating(None),
            Self::Discriminant::SetFloating => Self::SetFloating(None),
            Self::Discriminant::SetTiled => Self::SetTiled(None),
            Self::Discriminant::Fullscreen => Self::Fullscreen(FullscreenMode::default()),
            Self::Discriminant::FullscreenState => Self::FullscreenState(
                DispatcherFullscreenState::default(),
                DispatcherFullscreenState::default(),
            ),
            Self::Discriminant::Dpms => Self::Dpms(ToggleState::default(), None),
            Self::Discriminant::Pin => Self::Pin(None),
            Self::Discriminant::MoveFocus => Self::MoveFocus(Direction::default()),
            Self::Discriminant::MoveWindow => Self::MoveWindow(MoveDirection::default()),
            Self::Discriminant::SwapWindow => Self::SwapWindow(SwapDirection::default()),
            Self::Discriminant::CenterWindow => Self::CenterWindow(false),
            Self::Discriminant::ResizeActive => Self::ResizeActive(ResizeParams::default()),
            Self::Discriminant::MoveActive => Self::MoveActive(ResizeParams::default()),
            Self::Discriminant::ResizeWindowPixel => {
                Self::ResizeWindowPixel(ResizeParams::default(), WindowTarget::default())
            }
            Self::Discriminant::MoveWindowPixel => {
                Self::MoveWindowPixel(ResizeParams::default(), WindowTarget::default())
            }
            Self::Discriminant::CycleNext => Self::CycleNext(CycleNext::default()),
            Self::Discriminant::SwapNext => Self::SwapNext(SwapNext::default()),
            Self::Discriminant::TagWindow => {
                Self::TagWindow(TagToggleState::Toggle, "".to_string(), None)
            }
            Self::Discriminant::FocusWindow => Self::FocusWindow(WindowTarget::default()),
            Self::Discriminant::FocusMonitor => Self::FocusMonitor(MonitorTarget::default()),
            Self::Discriminant::SplitRatio => Self::SplitRatio(FloatValue::default()),
            Self::Discriminant::MoveCursorToCorner => {
                Self::MoveCursorToCorner(CursorCorner::default())
            }
            Self::Discriminant::MoveCursor => Self::MoveCursor(0, 0),
            Self::Discriminant::RenameWorkspace => Self::RenameWorkspace(1, "".to_string()),
            Self::Discriminant::Exit => Self::Exit,
            Self::Discriminant::ForceRendererReload => Self::ForceRendererReload,
            Self::Discriminant::MoveCurrentWorkspaceToMonitor => {
                Self::MoveCurrentWorkspaceToMonitor(MonitorTarget::default())
            }
            Self::Discriminant::FocusWorkspaceOnCurrentMonitor => {
                Self::FocusWorkspaceOnCurrentMonitor(WorkspaceTarget::default())
            }
            Self::Discriminant::MoveWorkspaceToMonitor => {
                Self::MoveWorkspaceToMonitor(WorkspaceTarget::default(), MonitorTarget::default())
            }
            Self::Discriminant::SwapActiveWorkspaces => {
                Self::SwapActiveWorkspaces(MonitorTarget::default(), MonitorTarget::default())
            }
            Self::Discriminant::BringActiveToTop => Self::BringActiveToTop,
            Self::Discriminant::AlterZOrder => Self::AlterZOrder(ZHeight::default(), None),
            Self::Discriminant::ToggleSpecialWorkspace => Self::ToggleSpecialWorkspace(None),
            Self::Discriminant::FocusUrgentOrLast => Self::FocusUrgentOrLast,
            Self::Discriminant::ToggleGroup => Self::ToggleGroup,
            Self::Discriminant::ChangeGroupActive => {
                Self::ChangeGroupActive(ChangeGroupActive::default())
            }
            Self::Discriminant::FocusCurrentOrLast => Self::FocusCurrentOrLast,
            Self::Discriminant::LockGroups => Self::LockGroups(GroupLockAction::default()),
            Self::Discriminant::LockActiveGroup => {
                Self::LockActiveGroup(GroupLockAction::default())
            }
            Self::Discriminant::MoveIntoGroup => Self::MoveIntoGroup(Direction::default()),
            Self::Discriminant::MoveOutOfGroup => Self::MoveOutOfGroup(None),
            Self::Discriminant::MoveWindowOrGroup => Self::MoveWindowOrGroup(Direction::default()),
            Self::Discriminant::MoveGroupWindow => Self::MoveGroupWindow(false),
            Self::Discriminant::DenyWindowFromGroup => {
                Self::DenyWindowFromGroup(ToggleState::default())
            }
            Self::Discriminant::SetIgnoreGroupLock => {
                Self::SetIgnoreGroupLock(ToggleState::default())
            }
            Self::Discriminant::Global => Self::Global("".to_string()),
            Self::Discriminant::Event => Self::Event("".to_string()),
            Self::Discriminant::SetProp => Self::SetProp(SetProp::default()),
            Self::Discriminant::ToggleSwallow => Self::ToggleSwallow,
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Exec => {
                if str.starts_with('[') {
                    let mut rules = Vec::new();
                    let mut rule = String::new();
                    let mut in_brackets = false;
                    let mut command = String::new();

                    for c in str.chars() {
                        if c == '[' {
                            in_brackets = true;
                        } else if c == ']' {
                            if !rule.trim().is_empty() {
                                rules.push(rule.parse().unwrap_or_default());
                                rule.clear();
                            }
                            in_brackets = false;
                        } else if c == ';' && in_brackets {
                            if !rule.trim().is_empty() {
                                rules.push(rule.parse().unwrap_or_default());
                                rule.clear();
                            }
                        } else if in_brackets {
                            rule.push(c);
                        } else {
                            command.push(c);
                        }
                    }

                    Self::Exec(rules, command.trim_start().to_string())
                } else {
                    Self::Exec(Vec::new(), str.to_string())
                }
            }
            Self::Discriminant::Execr => Self::Execr(str.to_string()),
            Self::Discriminant::Pass => Self::Pass(str.parse().unwrap_or_default()),
            Self::Discriminant::SendShortcut => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() == 1 {
                    Self::SendShortcut(parse_modifiers(parts[0]), String::new(), None)
                } else if parts.len() == 2 {
                    Self::SendShortcut(
                        parse_modifiers(parts[0]),
                        parts[1].parse().unwrap_or_default(),
                        None,
                    )
                } else {
                    Self::SendShortcut(
                        parse_modifiers(parts[0]),
                        parts[1].parse().unwrap_or_default(),
                        Some(WindowTarget::from_str(parts[2]).unwrap_or_default()),
                    )
                }
            }
            Self::Discriminant::SendKeyState => {
                let parts: Vec<&str> = str.split(',').collect();
                let mods = parse_modifiers(parts.first().unwrap_or(&""));
                let key = parts.get(1).unwrap_or(&"").to_string();
                let state = parts.get(2).unwrap_or(&"").parse().unwrap_or_default();
                let window_target = parts.get(3).unwrap_or(&"").parse().unwrap_or_default();
                Self::SendKeyState(mods, key, state, window_target)
            }
            Self::Discriminant::KillActive => Self::KillActive,
            Self::Discriminant::ForceKillActive => Self::ForceKillActive,
            Self::Discriminant::CloseWindow => Self::CloseWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::KillWindow => Self::KillWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::Signal => Self::Signal(str.to_string()),
            Self::Discriminant::SignalWindow => {
                let parts: Vec<&str> = str.split(',').collect();
                let window_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                let signal = parts.get(1).unwrap_or(&"").to_string();
                Self::SignalWindow(window_target, signal)
            }
            Self::Discriminant::Workspace => Self::Workspace(str.parse().unwrap_or_default()),
            Self::Discriminant::MoveToWorkspace => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() == 1 {
                    Self::MoveToWorkspace(str.parse().unwrap_or_default(), None)
                } else {
                    let workspace_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                    let window_target = parts.get(1).unwrap_or(&"").parse().unwrap_or_default();
                    Self::MoveToWorkspace(workspace_target, Some(window_target))
                }
            }
            Self::Discriminant::MoveToWorkspaceSilent => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() == 1 {
                    Self::MoveToWorkspaceSilent(str.parse().unwrap_or_default(), None)
                } else {
                    let workspace_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                    let window_target = parts.get(1).unwrap_or(&"").parse().unwrap_or_default();
                    Self::MoveToWorkspaceSilent(workspace_target, Some(window_target))
                }
            }
            Self::Discriminant::ToggleFloating => {
                if str.is_empty() || str == "active" {
                    Self::ToggleFloating(None)
                } else {
                    Self::ToggleFloating(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::SetFloating => {
                if str.is_empty() || str == "active" {
                    Self::SetFloating(None)
                } else {
                    Self::SetFloating(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::SetTiled => {
                if str.is_empty() || str == "active" {
                    Self::SetTiled(None)
                } else {
                    Self::SetTiled(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::Fullscreen => {
                Self::Fullscreen(FullscreenMode::from_num(str.parse().unwrap_or(0)))
            }
            Self::Discriminant::FullscreenState => {
                let (internal, client) = str.split_once(' ').unwrap_or((str, ""));
                let internal = internal.parse().unwrap_or(0);
                let client = client.parse().unwrap_or(0);
                Self::FullscreenState(
                    DispatcherFullscreenState::from_num(internal),
                    DispatcherFullscreenState::from_num(client),
                )
            }
            Self::Discriminant::Dpms => {
                let (state, monitor_name) = str.split_once(' ').unwrap_or((str, ""));
                let state = state.parse().unwrap_or_default();
                let monitor_name = match monitor_name {
                    "" => None,
                    name => Some(name.to_string()),
                };
                Self::Dpms(state, monitor_name)
            }
            Self::Discriminant::Pin => {
                if str.is_empty() || str == "active" {
                    Self::Pin(None)
                } else {
                    Self::Pin(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::MoveFocus => Self::MoveFocus(str.parse().unwrap_or_default()),
            Self::Discriminant::MoveWindow => Self::MoveWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::SwapWindow => Self::SwapWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::CenterWindow => Self::CenterWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::ResizeActive => Self::ResizeActive(str.parse().unwrap_or_default()),
            Self::Discriminant::MoveActive => Self::MoveActive(str.parse().unwrap_or_default()),
            Self::Discriminant::ResizeWindowPixel => {
                let (resize_params, window_target) = str.split_once(' ').unwrap_or((str, ""));
                let resize_params = ResizeParams::from_str(resize_params).unwrap_or_default();
                let window_target = window_target.parse().unwrap_or_default();
                Self::ResizeWindowPixel(resize_params, window_target)
            }
            Self::Discriminant::MoveWindowPixel => {
                let (resize_params, window_target) = str.split_once(' ').unwrap_or((str, ""));
                let resize_params = ResizeParams::from_str(resize_params).unwrap_or_default();
                let window_target = window_target.parse().unwrap_or_default();
                Self::MoveWindowPixel(resize_params, window_target)
            }
            Self::Discriminant::CycleNext => Self::CycleNext(str.parse().unwrap_or_default()),
            Self::Discriminant::SwapNext => Self::SwapNext(str.parse().unwrap_or_default()),
            Self::Discriminant::TagWindow => {
                let (tag, window_target) = str.split_once(' ').unwrap_or((str, ""));
                let tag_toggle_state = if tag.starts_with("+") {
                    TagToggleState::Set
                } else if tag.starts_with("-") {
                    TagToggleState::Unset
                } else {
                    TagToggleState::Toggle
                };
                let tag = tag
                    .trim_start_matches("+")
                    .trim_start_matches("-")
                    .to_string();
                let window_target = match window_target {
                    "" => None,
                    window_target => Some(window_target.parse().unwrap_or_default()),
                };
                Self::TagWindow(tag_toggle_state, tag, window_target)
            }
            Self::Discriminant::FocusWindow => Self::FocusWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::FocusMonitor => Self::FocusMonitor(str.parse().unwrap_or_default()),
            Self::Discriminant::SplitRatio => Self::SplitRatio(str.parse().unwrap_or_default()),
            Self::Discriminant::MoveCursorToCorner => {
                Self::MoveCursorToCorner(CursorCorner::from_num(str.parse().unwrap_or_default()))
            }
            Self::Discriminant::MoveCursor => {
                let (x, y) = str.split_once(' ').unwrap_or((str, ""));
                let x = x.parse().unwrap_or_default();
                let y = y.parse().unwrap_or_default();
                Self::MoveCursor(x, y)
            }
            Self::Discriminant::RenameWorkspace => {
                let (workspace, name) = str.split_once(' ').unwrap_or((str, ""));
                let workspace_id = match workspace.parse().unwrap_or_default() {
                    0 => 1,
                    id => id,
                };
                let name = name.to_string();
                Self::RenameWorkspace(workspace_id, name)
            }
            Self::Discriminant::Exit => Self::Exit,
            Self::Discriminant::ForceRendererReload => Self::ForceRendererReload,
            Self::Discriminant::MoveCurrentWorkspaceToMonitor => {
                Self::MoveCurrentWorkspaceToMonitor(str.parse().unwrap_or_default())
            }
            Self::Discriminant::FocusWorkspaceOnCurrentMonitor => {
                Self::FocusWorkspaceOnCurrentMonitor(str.parse().unwrap_or_default())
            }
            Self::Discriminant::MoveWorkspaceToMonitor => {
                let (workspace_target, monitor_target) = str.split_once(' ').unwrap_or((str, ""));
                let workspace_target = workspace_target.parse().unwrap_or_default();
                let monitor_target = monitor_target.parse().unwrap_or_default();
                Self::MoveWorkspaceToMonitor(workspace_target, monitor_target)
            }
            Self::Discriminant::SwapActiveWorkspaces => {
                let (first_monitor, second_monitor) = str.split_once(' ').unwrap_or((str, ""));
                let first_monitor = first_monitor.parse().unwrap_or_default();
                let second_monitor = second_monitor.parse().unwrap_or_default();
                Self::SwapActiveWorkspaces(first_monitor, second_monitor)
            }
            Self::Discriminant::BringActiveToTop => Self::BringActiveToTop,
            Self::Discriminant::AlterZOrder => {
                let (zheight, window_target) = str.split_once(' ').unwrap_or((str, ""));
                let zheight = zheight.parse().unwrap_or_default();
                let window_target = match window_target {
                    "" => None,
                    window_target => Some(window_target.parse().unwrap_or_default()),
                };
                Self::AlterZOrder(zheight, window_target)
            }
            Self::Discriminant::ToggleSpecialWorkspace => match str {
                "" => Self::ToggleSpecialWorkspace(None),
                name => Self::ToggleSpecialWorkspace(Some(name.to_string())),
            },
            Self::Discriminant::FocusUrgentOrLast => Self::FocusUrgentOrLast,
            Self::Discriminant::ToggleGroup => Self::ToggleGroup,
            Self::Discriminant::ChangeGroupActive => {
                Self::ChangeGroupActive(str.parse().unwrap_or_default())
            }
            Self::Discriminant::FocusCurrentOrLast => Self::FocusCurrentOrLast,
            Self::Discriminant::LockGroups => Self::LockGroups(str.parse().unwrap_or_default()),
            Self::Discriminant::LockActiveGroup => {
                Self::LockActiveGroup(str.parse().unwrap_or_default())
            }
            Self::Discriminant::MoveIntoGroup => {
                Self::MoveIntoGroup(str.parse().unwrap_or_default())
            }
            Self::Discriminant::MoveOutOfGroup => {
                if str.is_empty() || str == "active" {
                    Self::MoveOutOfGroup(None)
                } else {
                    Self::MoveOutOfGroup(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::MoveWindowOrGroup => {
                Self::MoveWindowOrGroup(str.parse().unwrap_or_default())
            }
            Self::Discriminant::MoveGroupWindow => Self::MoveGroupWindow(str == "b"),
            Self::Discriminant::DenyWindowFromGroup => {
                Self::DenyWindowFromGroup(str.parse().unwrap_or_default())
            }
            Self::Discriminant::SetIgnoreGroupLock => {
                Self::SetIgnoreGroupLock(str.parse().unwrap_or_default())
            }
            Self::Discriminant::Global => Self::Global(str.to_string()),
            Self::Discriminant::Event => Self::Event(str.to_string()),
            Self::Discriminant::SetProp => Self::SetProp(str.parse().unwrap_or_default()),
            Self::Discriminant::ToggleSwallow => Self::ToggleSwallow,
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Dispatcher::Exec(window_rules, command) => {
                if window_rules.is_empty() {
                    Some(command.clone())
                } else {
                    Some(format!(
                        "[{}] {}",
                        join_with_separator(window_rules, "; "),
                        command
                    ))
                }
            }
            Dispatcher::Execr(command) => Some(command.clone()),
            Dispatcher::Pass(window_target) => Some(window_target.to_string()),
            Dispatcher::SendShortcut(modifiers, key, None) => {
                Some(format!("{} {}", join_with_separator(modifiers, "_"), key))
            }
            Dispatcher::SendShortcut(modifiers, key, Some(window_target)) => Some(format!(
                "{} {} {}",
                join_with_separator(modifiers, "_"),
                key,
                window_target
            )),
            Dispatcher::SendKeyState(modifiers, key, state, window_target) => Some(format!(
                "{} {} {} {}",
                join_with_separator(modifiers, "_"),
                key,
                state,
                window_target
            )),
            Dispatcher::KillActive => None,
            Dispatcher::ForceKillActive => None,
            Dispatcher::CloseWindow(window_target) => Some(window_target.to_string()),
            Dispatcher::KillWindow(window_target) => Some(window_target.to_string()),
            Dispatcher::Signal(signal) => Some(signal.clone()),
            Dispatcher::SignalWindow(window_target, signal) => {
                Some(format!("{} {}", window_target, signal))
            }
            Dispatcher::Workspace(workspace_target) => Some(workspace_target.to_string()),
            Dispatcher::MoveToWorkspace(workspace_target, None) => {
                Some(workspace_target.to_string())
            }
            Dispatcher::MoveToWorkspace(workspace_target, Some(window_target)) => {
                Some(format!("{} {}", workspace_target, window_target))
            }
            Dispatcher::MoveToWorkspaceSilent(workspace_target, None) => {
                Some(workspace_target.to_string())
            }
            Dispatcher::MoveToWorkspaceSilent(workspace_target, Some(window_target)) => {
                Some(format!("{} {}", workspace_target, window_target))
            }
            Dispatcher::ToggleFloating(None) => None,
            Dispatcher::ToggleFloating(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::SetFloating(None) => None,
            Dispatcher::SetFloating(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::SetTiled(None) => None,
            Dispatcher::SetTiled(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::Fullscreen(mode) => Some(mode.to_num().to_string()),
            Dispatcher::FullscreenState(internal, client) => {
                Some(format!("{} {}", internal.to_num(), client.to_num()))
            }
            Dispatcher::Dpms(state, None) => Some(state.to_string()),
            Dispatcher::Dpms(state, Some(name)) => Some(format!("{} {}", state, name)),
            Dispatcher::Pin(None) => None,
            Dispatcher::Pin(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::MoveFocus(direction) => Some(direction.to_string()),
            Dispatcher::MoveWindow(move_direction) => Some(move_direction.to_string()),
            Dispatcher::SwapWindow(swap_direction) => Some(swap_direction.to_string()),
            Dispatcher::CenterWindow(false) => None,
            Dispatcher::CenterWindow(true) => Some("1".to_string()),
            Dispatcher::ResizeActive(resize_params) => Some(resize_params.to_string()),
            Dispatcher::MoveActive(resize_params) => Some(resize_params.to_string()),
            Dispatcher::ResizeWindowPixel(resize_params, window_target) => {
                Some(format!("{} {}", resize_params, window_target))
            }
            Dispatcher::MoveWindowPixel(move_params, window_target) => {
                Some(format!("{} {}", move_params, window_target))
            }
            Dispatcher::CycleNext(cycle_next) => Some(cycle_next.to_string()),
            Dispatcher::SwapNext(swap_next) => Some(swap_next.to_string()),
            Dispatcher::TagWindow(TagToggleState::Toggle, tag, None) => Some(tag.clone()),
            Dispatcher::TagWindow(TagToggleState::Toggle, tag, Some(window_target)) => {
                Some(format!("{} {}", tag, window_target))
            }
            Dispatcher::TagWindow(TagToggleState::Set, tag, None) => Some(format!("+{}", tag)),
            Dispatcher::TagWindow(TagToggleState::Set, tag, Some(window_target)) => {
                Some(format!("+{} {}", tag, window_target))
            }
            Dispatcher::TagWindow(TagToggleState::Unset, tag, None) => Some(format!("-{}", tag)),
            Dispatcher::TagWindow(TagToggleState::Unset, tag, Some(window_target)) => {
                Some(format!("-{} {}", tag, window_target))
            }
            Dispatcher::FocusWindow(window_target) => Some(window_target.to_string()),
            Dispatcher::FocusMonitor(monitor_target) => Some(monitor_target.to_string()),
            Dispatcher::SplitRatio(float_value) => Some(float_value.to_string()),
            Dispatcher::MoveCursorToCorner(corner) => Some(corner.to_num().to_string()),
            Dispatcher::MoveCursor(x, y) => Some(format!("{} {}", x, y)),
            Dispatcher::RenameWorkspace(id, name) => Some(format!("{} {}", id, name)),
            Dispatcher::Exit => None,
            Dispatcher::ForceRendererReload => None,
            Dispatcher::MoveCurrentWorkspaceToMonitor(monitor_target) => {
                Some(monitor_target.to_string())
            }
            Dispatcher::FocusWorkspaceOnCurrentMonitor(workspace_target) => {
                Some(workspace_target.to_string())
            }
            Dispatcher::MoveWorkspaceToMonitor(workspace_target, monitor_target) => {
                Some(format!("{} {}", workspace_target, monitor_target))
            }
            Dispatcher::SwapActiveWorkspaces(first_monitor, second_monitor) => {
                Some(format!("{} {}", first_monitor, second_monitor))
            }
            Dispatcher::BringActiveToTop => None,
            Dispatcher::AlterZOrder(zheight, None) => Some(zheight.to_string()),
            Dispatcher::AlterZOrder(zheight, Some(window_target)) => {
                Some(format!("{} {}", zheight, window_target))
            }
            Dispatcher::ToggleSpecialWorkspace(None) => None,
            Dispatcher::ToggleSpecialWorkspace(Some(name)) => Some(name.clone()),
            Dispatcher::FocusUrgentOrLast => None,
            Dispatcher::ToggleGroup => None,
            Dispatcher::ChangeGroupActive(change_group_active) => {
                Some(change_group_active.to_string())
            }
            Dispatcher::FocusCurrentOrLast => None,
            Dispatcher::LockGroups(group_lock_action) => Some(group_lock_action.to_string()),
            Dispatcher::LockActiveGroup(group_lock_action) => Some(group_lock_action.to_string()),
            Dispatcher::MoveIntoGroup(direction) => Some(direction.to_string()),
            Dispatcher::MoveOutOfGroup(None) => None,
            Dispatcher::MoveOutOfGroup(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::MoveWindowOrGroup(direction) => Some(direction.to_string()),
            Dispatcher::MoveGroupWindow(true) => Some("b".to_string()),
            Dispatcher::MoveGroupWindow(false) => Some("f".to_string()),
            Dispatcher::DenyWindowFromGroup(toggle_state) => Some(toggle_state.to_string()),
            Dispatcher::SetIgnoreGroupLock(toggle_state) => Some(toggle_state.to_string()),
            Dispatcher::Global(name) => Some(name.clone()),
            Dispatcher::Event(event) => Some(event.clone()),
            Dispatcher::SetProp(set_prop) => Some(set_prop.to_string()),
            Dispatcher::ToggleSwallow => None,
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Dispatcher::Exec(vec![], String::new())
    }
}

impl FromStr for Dispatcher {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        if s.is_empty() {
            return Err(());
        }

        let (dispatcher, params) = s.split_once(",").unwrap_or((&s, ""));

        let dispatcher = dispatcher.trim().to_lowercase();

        let params = params.trim();

        match dispatcher.as_str() {
            "exec" => {
                if params.starts_with("[") {
                    let mut rules = Vec::new();
                    let mut rule = String::new();
                    let mut in_brackets = false;
                    let mut command = String::new();

                    for c in params.chars() {
                        if c == '[' {
                            in_brackets = true;
                        } else if c == ']' {
                            if !rule.trim().is_empty() {
                                rules.push(rule.parse().unwrap_or_default());
                                rule.clear();
                            }
                            in_brackets = false;
                        } else if c == ';' && in_brackets {
                            if !rule.trim().is_empty() {
                                rules.push(rule.parse().unwrap_or_default());
                                rule.clear();
                            }
                        } else if in_brackets {
                            rule.push(c);
                        } else {
                            command.push(c);
                        }
                    }

                    Ok(Dispatcher::Exec(rules, command.trim_start().to_string()))
                } else {
                    Ok(Dispatcher::Exec(Vec::new(), params.to_string()))
                }
            }
            "execr" => Ok(Dispatcher::Execr(params.to_string())),
            "pass" => Ok(Dispatcher::Pass(params.parse().unwrap_or_default())),
            "sendshortcut" => {
                let parts: Vec<&str> = params.split(' ').collect();
                if parts.len() == 1 {
                    Ok(Dispatcher::SendShortcut(
                        parse_modifiers(parts[0]),
                        String::new(),
                        None,
                    ))
                } else if parts.len() == 2 {
                    Ok(Dispatcher::SendShortcut(
                        parse_modifiers(parts[0]),
                        parts[1].parse().unwrap_or_default(),
                        None,
                    ))
                } else {
                    Ok(Dispatcher::SendShortcut(
                        parse_modifiers(parts[0]),
                        parts[1].parse().unwrap_or_default(),
                        Some(WindowTarget::from_str(parts[2]).unwrap_or_default()),
                    ))
                }
            }
            "sendkeystate" => {
                let parts: Vec<&str> = params.split(' ').collect();

                let mods = parse_modifiers(parts.first().unwrap_or(&""));
                let key = parts.get(1).unwrap_or(&"").to_string();
                let state = parts.get(2).unwrap_or(&"").parse().unwrap_or_default();
                let window_target = parts.get(3).unwrap_or(&"").parse().unwrap_or_default();

                Ok(Dispatcher::SendKeyState(mods, key, state, window_target))
            }
            "killactive" => Ok(Dispatcher::KillActive),
            "forcekillactive" => Ok(Dispatcher::ForceKillActive),
            "closewindow" => Ok(Dispatcher::CloseWindow(params.parse().unwrap_or_default())),
            "killwindow" => Ok(Dispatcher::KillWindow(params.parse().unwrap_or_default())),
            "signal" => Ok(Dispatcher::Signal(params.to_string())),
            "signalwindow" => {
                let parts: Vec<&str> = params.split(' ').collect();

                let window_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                let signal = parts.get(1).unwrap_or(&"").to_string();

                Ok(Dispatcher::SignalWindow(window_target, signal))
            }
            "workspace" => Ok(Dispatcher::Workspace(params.parse().unwrap_or_default())),
            "movetoworkspace" => {
                let parts: Vec<&str> = params.split(' ').collect();

                if parts.len() == 1 {
                    Ok(Dispatcher::MoveToWorkspace(
                        params.parse().unwrap_or_default(),
                        None,
                    ))
                } else {
                    let workspace_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                    let window_target = parts.get(1).unwrap_or(&"").parse().unwrap_or_default();

                    Ok(Dispatcher::MoveToWorkspace(
                        workspace_target,
                        Some(window_target),
                    ))
                }
            }
            "movetoworkspacesilent" => {
                let parts: Vec<&str> = params.split(' ').collect();

                if parts.len() == 1 {
                    Ok(Dispatcher::MoveToWorkspaceSilent(
                        params.parse().unwrap_or_default(),
                        None,
                    ))
                } else {
                    let workspace_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                    let window_target = parts.get(1).unwrap_or(&"").parse().unwrap_or_default();

                    Ok(Dispatcher::MoveToWorkspaceSilent(
                        workspace_target,
                        Some(window_target),
                    ))
                }
            }
            "togglefloating" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::ToggleFloating(None))
                } else {
                    Ok(Dispatcher::ToggleFloating(Some(
                        params.parse().unwrap_or_default(),
                    )))
                }
            }
            "setfloating" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::SetFloating(None))
                } else {
                    Ok(Dispatcher::SetFloating(Some(
                        params.parse().unwrap_or_default(),
                    )))
                }
            }
            "settiled" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::SetTiled(None))
                } else {
                    Ok(Dispatcher::SetTiled(Some(
                        params.parse().unwrap_or_default(),
                    )))
                }
            }
            "fullscreen" => Ok(Dispatcher::Fullscreen(FullscreenMode::from_num(
                params.parse().unwrap_or(0),
            ))),
            "fullscreenstate" => {
                let (internal, client) = params.split_once(' ').unwrap_or((params, ""));
                let internal = internal.parse().unwrap_or(0);
                let client = client.parse().unwrap_or(0);

                Ok(Dispatcher::FullscreenState(
                    DispatcherFullscreenState::from_num(internal),
                    DispatcherFullscreenState::from_num(client),
                ))
            }
            "dpms" => {
                let (state, monitor_name) = params.split_once(' ').unwrap_or((params, ""));

                let state = state.parse().unwrap_or_default();
                let monitor_name = match monitor_name {
                    "" => None,
                    name => Some(name.to_string()),
                };

                Ok(Dispatcher::Dpms(state, monitor_name))
            }
            "pin" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::Pin(None))
                } else {
                    Ok(Dispatcher::Pin(Some(params.parse().unwrap_or_default())))
                }
            }
            "movefocus" => Ok(Dispatcher::MoveFocus(params.parse().unwrap_or_default())),
            "movewindow" => Ok(Dispatcher::MoveWindow(params.parse().unwrap_or_default())),
            "swapwindow" => Ok(Dispatcher::SwapWindow(params.parse().unwrap_or_default())),
            "centerwindow" => Ok(Dispatcher::CenterWindow(params.parse().unwrap_or_default())),
            "resizeactive" => Ok(Dispatcher::ResizeActive(params.parse().unwrap_or_default())),
            "moveactive" => Ok(Dispatcher::MoveActive(params.parse().unwrap_or_default())),
            "resizewindowpixel" => {
                let (resize_params, window_target) = params.split_once(' ').unwrap_or((params, ""));

                let resize_params = ResizeParams::from_str(resize_params).unwrap_or_default();
                let window_target = window_target.parse().unwrap_or_default();

                Ok(Dispatcher::ResizeWindowPixel(resize_params, window_target))
            }
            "movewindowpixel" => {
                let (resize_params, window_target) = params.split_once(' ').unwrap_or((params, ""));

                let resize_params = ResizeParams::from_str(resize_params).unwrap_or_default();
                let window_target = window_target.parse().unwrap_or_default();

                Ok(Dispatcher::MoveWindowPixel(resize_params, window_target))
            }
            "cyclenext" => Ok(Dispatcher::CycleNext(params.parse().unwrap_or_default())),
            "swapnext" => Ok(Dispatcher::SwapNext(params.parse().unwrap_or_default())),
            "tagwindow" => {
                let (tag, window_target) = params.split_once(' ').unwrap_or((params, ""));

                let tag_toggle_state = if tag.starts_with("+") {
                    TagToggleState::Set
                } else if tag.starts_with("-") {
                    TagToggleState::Unset
                } else {
                    TagToggleState::Toggle
                };
                let tag = tag
                    .trim_start_matches("+")
                    .trim_start_matches("-")
                    .to_string();
                let window_target = match window_target {
                    "" => None,
                    window_target => Some(window_target.parse().unwrap_or_default()),
                };

                Ok(Dispatcher::TagWindow(tag_toggle_state, tag, window_target))
            }
            "focuswindow" => Ok(Dispatcher::FocusWindow(params.parse().unwrap_or_default())),
            "focusmonitor" => Ok(Dispatcher::FocusMonitor(params.parse().unwrap_or_default())),
            "splitratio" => Ok(Dispatcher::SplitRatio(params.parse().unwrap_or_default())),
            "movecursortocorner" => Ok(Dispatcher::MoveCursorToCorner(CursorCorner::from_num(
                params.parse().unwrap_or_default(),
            ))),
            "movecursor" => {
                let (x, y) = params.split_once(' ').unwrap_or((params, ""));

                let x = x.parse().unwrap_or_default();
                let y = y.parse().unwrap_or_default();

                Ok(Dispatcher::MoveCursor(x, y))
            }
            "renameworkspace" => {
                let (workspace, name) = params.split_once(' ').unwrap_or((params, ""));

                let workspace_id = match workspace.parse().unwrap_or_default() {
                    0 => 1,
                    id => id,
                };
                let name = name.to_string();

                Ok(Dispatcher::RenameWorkspace(workspace_id, name))
            }
            "exit" => Ok(Dispatcher::Exit),
            "forcerendererreload" => Ok(Dispatcher::ForceRendererReload),
            "movecurrentworkspacetomonitor" => Ok(Dispatcher::MoveCurrentWorkspaceToMonitor(
                params.parse().unwrap_or_default(),
            )),
            "focusworkspaceoncurrentmonitor" => Ok(Dispatcher::FocusWorkspaceOnCurrentMonitor(
                params.parse().unwrap_or_default(),
            )),
            "moveworkspacetomonitor" => {
                let (workspace_target, monitor_target) =
                    params.split_once(' ').unwrap_or((params, ""));

                let workspace_target = workspace_target.parse().unwrap_or_default();
                let monitor_target = monitor_target.parse().unwrap_or_default();

                Ok(Dispatcher::MoveWorkspaceToMonitor(
                    workspace_target,
                    monitor_target,
                ))
            }
            "swapactiveworkspaces" => {
                let (first_monitor, second_monitor) =
                    params.split_once(' ').unwrap_or((params, ""));

                let first_monitor = first_monitor.parse().unwrap_or_default();
                let second_monitor = second_monitor.parse().unwrap_or_default();

                Ok(Dispatcher::SwapActiveWorkspaces(
                    first_monitor,
                    second_monitor,
                ))
            }
            "bringactivetotop" => Ok(Dispatcher::BringActiveToTop),
            "alterzorder" => {
                let (zheight, window_target) = params.split_once(' ').unwrap_or((params, ""));

                let zheight = zheight.parse().unwrap_or_default();
                let window_target = match window_target {
                    "" => None,
                    window_target => Some(window_target.parse().unwrap_or_default()),
                };

                Ok(Dispatcher::AlterZOrder(zheight, window_target))
            }
            "togglespecialworkspace" => match params {
                "" => Ok(Dispatcher::ToggleSpecialWorkspace(None)),
                name => Ok(Dispatcher::ToggleSpecialWorkspace(Some(name.to_string()))),
            },
            "focusurgentorlast" => Ok(Dispatcher::FocusUrgentOrLast),
            "togglegroup" => Ok(Dispatcher::ToggleGroup),
            "changegroupactive" => Ok(Dispatcher::ChangeGroupActive(
                params.parse().unwrap_or_default(),
            )),
            "focuscurrentorlast" => Ok(Dispatcher::FocusCurrentOrLast),
            "lockgroups" => Ok(Dispatcher::LockGroups(params.parse().unwrap_or_default())),
            "lockactivegroup" => Ok(Dispatcher::LockActiveGroup(
                params.parse().unwrap_or_default(),
            )),
            "moveintogroup" => Ok(Dispatcher::MoveIntoGroup(
                params.parse().unwrap_or_default(),
            )),
            "moveoutofgroup" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::MoveOutOfGroup(None))
                } else {
                    Ok(Dispatcher::MoveOutOfGroup(Some(
                        params.parse().unwrap_or_default(),
                    )))
                }
            }
            "movewindoworgroup" => Ok(Dispatcher::MoveWindowOrGroup(
                params.parse().unwrap_or_default(),
            )),
            "movegroupwindow" => Ok(Dispatcher::MoveGroupWindow(params == "b")),
            "denywindowfromgroup" => Ok(Dispatcher::DenyWindowFromGroup(
                params.parse().unwrap_or_default(),
            )),
            "setignoregrouplock" => Ok(Dispatcher::SetIgnoreGroupLock(
                params.parse().unwrap_or_default(),
            )),
            "global" => Ok(Dispatcher::Global(params.to_string())),
            "event" => Ok(Dispatcher::Event(params.to_string())),
            "setprop" => Ok(Dispatcher::SetProp(params.parse().unwrap_or_default())),
            "toggleswallow" => Ok(Dispatcher::ToggleSwallow),
            _ => Err(()),
        }
    }
}

impl Display for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dispatcher::Exec(window_rules, command) => {
                if window_rules.is_empty() {
                    write!(f, "exec, {}", command)
                } else {
                    write!(
                        f,
                        "exec, [{}] {}",
                        join_with_separator(window_rules, "; "),
                        command.trim()
                    )
                }
            }
            Dispatcher::Execr(command) => write!(f, "execr, {}", command),
            Dispatcher::Pass(window_target) => write!(f, "pass, {}", window_target),
            Dispatcher::SendShortcut(modifiers, key, None) => {
                write!(
                    f,
                    "sendshortcut, {} {}",
                    join_with_separator(modifiers, "_"),
                    key
                )
            }
            Dispatcher::SendShortcut(modifiers, key, Some(window_target)) => write!(
                f,
                "sendshortcut, {} {} {}",
                join_with_separator(modifiers, "_"),
                key,
                window_target,
            ),
            Dispatcher::SendKeyState(modifiers, key, state, window_target) => write!(
                f,
                "sendkeystate, {} {} {} {}",
                join_with_separator(modifiers, "_"),
                key,
                state,
                window_target,
            ),
            Dispatcher::KillActive => write!(f, "killactive"),
            Dispatcher::ForceKillActive => write!(f, "forcekillactive"),
            Dispatcher::CloseWindow(window_target) => {
                write!(f, "killwindow, {}", window_target)
            }
            Dispatcher::KillWindow(window_target) => {
                write!(f, "killwindow, {}", window_target)
            }
            Dispatcher::Signal(signal) => write!(f, "signal, {}", signal),
            Dispatcher::SignalWindow(window_target, signal) => {
                write!(f, "killwindow, {} {}", window_target, signal)
            }
            Dispatcher::Workspace(workspace_target) => write!(f, "workspace, {}", workspace_target),
            Dispatcher::MoveToWorkspace(workspace_target, None) => {
                write!(f, "movetoworkspace, {}", workspace_target)
            }
            Dispatcher::MoveToWorkspace(workspace_target, Some(window_target)) => {
                write!(f, "movetoworkspace, {} {}", workspace_target, window_target,)
            }
            Dispatcher::MoveToWorkspaceSilent(workspace_target, None) => {
                write!(f, "movetoworkspace, {}", workspace_target)
            }
            Dispatcher::MoveToWorkspaceSilent(workspace_target, Some(window_target)) => {
                write!(f, "movetoworkspace, {} {}", workspace_target, window_target,)
            }
            Dispatcher::ToggleFloating(None) => write!(f, "togglefloating"),
            Dispatcher::ToggleFloating(Some(window_target)) => {
                write!(f, "togglefloating, {}", window_target,)
            }
            Dispatcher::SetFloating(None) => write!(f, "setfloating"),
            Dispatcher::SetFloating(Some(window_target)) => {
                write!(f, "setfloating, {}", window_target)
            }
            Dispatcher::SetTiled(None) => write!(f, "settiled"),
            Dispatcher::SetTiled(Some(window_target)) => write!(f, "settiled, {}", window_target),
            Dispatcher::Fullscreen(mode) => write!(f, "fullscreen, {}", mode.to_num()),
            Dispatcher::FullscreenState(internal, client) => {
                write!(f, "fullscreen, {} {}", internal.to_num(), client.to_num())
            }
            Dispatcher::Dpms(state, None) => {
                write!(f, "dpms, {}", state)
            }
            Dispatcher::Dpms(state, Some(name)) => {
                write!(f, "dpms, {} {}", state, name)
            }
            Dispatcher::Pin(None) => write!(f, "pin"),
            Dispatcher::Pin(Some(window_target)) => write!(f, "pin, {}", window_target),
            Dispatcher::MoveFocus(direction) => write!(f, "movefocus, {}", direction),
            Dispatcher::MoveWindow(move_direction) => write!(f, "movewindow, {}", move_direction),
            Dispatcher::SwapWindow(swap_direction) => write!(f, "swapwindow, {}", swap_direction),
            Dispatcher::CenterWindow(false) => write!(f, "centerwindow"),
            Dispatcher::CenterWindow(true) => write!(f, "centerwindow, 1"),
            Dispatcher::ResizeActive(resize_params) => write!(f, "resizeactive, {}", resize_params),
            Dispatcher::MoveActive(resize_params) => write!(f, "moveactive, {}", resize_params),
            Dispatcher::ResizeWindowPixel(resize_params, window_target) => {
                write!(f, "resizewindowpixel, {} {}", resize_params, window_target)
            }
            Dispatcher::MoveWindowPixel(move_params, window_target) => {
                write!(f, "movewindowpixel, {} {}", move_params, window_target)
            }
            Dispatcher::CycleNext(cycle_next) => write!(f, "cyclenext, {}", cycle_next),
            Dispatcher::SwapNext(swap_next) => write!(f, "swapnext, {}", swap_next),
            Dispatcher::TagWindow(TagToggleState::Toggle, tag, None) => {
                write!(f, "tagwindow, {}", tag)
            }
            Dispatcher::TagWindow(TagToggleState::Toggle, tag, Some(window_target)) => {
                write!(f, "tagwindow, {} {}", tag, window_target)
            }
            Dispatcher::TagWindow(TagToggleState::Set, tag, None) => {
                write!(f, "tagwindow, +{}", tag)
            }
            Dispatcher::TagWindow(TagToggleState::Set, tag, Some(window_target)) => {
                write!(f, "tagwindow, +{} {}", tag, window_target)
            }
            Dispatcher::TagWindow(TagToggleState::Unset, tag, None) => {
                write!(f, "tagwindow, -{}", tag)
            }
            Dispatcher::TagWindow(TagToggleState::Unset, tag, Some(window_target)) => {
                write!(f, "tagwindow, -{} {}", tag, window_target)
            }
            Dispatcher::FocusWindow(window_target) => write!(f, "focuswindow, {}", window_target),
            Dispatcher::FocusMonitor(monitor_target) => {
                write!(f, "focusmonitor, {}", monitor_target)
            }
            Dispatcher::SplitRatio(float_value) => write!(f, "splitratio, {}", float_value),
            Dispatcher::MoveCursorToCorner(corner) => {
                write!(f, "movecursortocorner, {}", corner.to_num())
            }
            Dispatcher::MoveCursor(x, y) => write!(f, "movecursor, {} {}", x, y),
            Dispatcher::RenameWorkspace(id, name) => write!(f, "renameworkspace, {} {}", id, name),
            Dispatcher::Exit => write!(f, "exit"),
            Dispatcher::ForceRendererReload => write!(f, "forcerendererreload"),
            Dispatcher::MoveCurrentWorkspaceToMonitor(monitor_target) => {
                write!(f, "movecurrentworkspacetomonitor, {}", monitor_target)
            }
            Dispatcher::FocusWorkspaceOnCurrentMonitor(workspace_target) => {
                write!(f, "focusworkspaceoncurrentmonitor, {}", workspace_target)
            }
            Dispatcher::MoveWorkspaceToMonitor(workspace_target, monitor_target) => {
                write!(
                    f,
                    "moveworkspacetomonitor, {} {}",
                    workspace_target, monitor_target
                )
            }
            Dispatcher::SwapActiveWorkspaces(first_monitor, second_monitor) => {
                write!(
                    f,
                    "swapactiveworkspaces, {} {}",
                    first_monitor, second_monitor
                )
            }
            Dispatcher::BringActiveToTop => write!(f, "bringactivetotop"),
            Dispatcher::AlterZOrder(zheight, None) => {
                write!(f, "alterzorder, {}", zheight)
            }
            Dispatcher::AlterZOrder(zheight, Some(window_target)) => {
                write!(f, "alterzorder, {} {}", zheight, window_target)
            }
            Dispatcher::ToggleSpecialWorkspace(None) => write!(f, "togglespecialworkspace"),
            Dispatcher::ToggleSpecialWorkspace(Some(name)) => {
                write!(f, "togglespecialworkspace, {}", name)
            }
            Dispatcher::FocusUrgentOrLast => write!(f, "focusurgentorlast"),
            Dispatcher::ToggleGroup => write!(f, "togglegroup"),
            Dispatcher::ChangeGroupActive(change_group_active) => {
                write!(f, "changegroupactive, {}", change_group_active)
            }
            Dispatcher::FocusCurrentOrLast => write!(f, "focuscurrentorlast"),
            Dispatcher::LockGroups(group_lock_action) => {
                write!(f, "lockgroups, {}", group_lock_action)
            }
            Dispatcher::LockActiveGroup(group_lock_action) => {
                write!(f, "lockactivegroup, {}", group_lock_action)
            }
            Dispatcher::MoveIntoGroup(direction) => {
                write!(f, "moveintogroup, {}", direction)
            }
            Dispatcher::MoveOutOfGroup(None) => {
                write!(f, "moveoutofgroup")
            }
            Dispatcher::MoveOutOfGroup(Some(window_target)) => {
                write!(f, "moveoutofgroup, {}", window_target)
            }
            Dispatcher::MoveWindowOrGroup(direction) => {
                write!(f, "movewindoworgroup, {}", direction)
            }
            Dispatcher::MoveGroupWindow(true) => {
                write!(f, "movegroupwindow, b")
            }
            Dispatcher::MoveGroupWindow(false) => {
                write!(f, "movegroupwindow, f")
            }
            Dispatcher::DenyWindowFromGroup(toggle_state) => {
                write!(f, "denywindowfromgroup, {}", toggle_state)
            }
            Dispatcher::SetIgnoreGroupLock(toggle_state) => {
                write!(f, "setignoregrouplock, {}", toggle_state)
            }
            Dispatcher::Global(name) => write!(f, "global, {}", name),
            Dispatcher::Event(event) => write!(f, "event, {}", event),
            Dispatcher::SetProp(set_prop) => write!(f, "setprop, {}", set_prop),
            Dispatcher::ToggleSwallow => write!(f, "toggleswallow"),
        }
    }
}

impl EnumConfigForGtk for Dispatcher {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.exec"),
            &t!("gtk_converters.execr"),
            &t!("gtk_converters.pass"),
            &t!("gtk_converters.send_shortcut"),
            &t!("gtk_converters.send_key_state"),
            &t!("gtk_converters.kill_active"),
            &t!("gtk_converters.force_kill_active"),
            &t!("gtk_converters.close_window"),
            &t!("gtk_converters.kill_window"),
            &t!("gtk_converters.signal"),
            &t!("gtk_converters.signal_window"),
            &t!("gtk_converters.workspace"),
            &t!("gtk_converters.move_to_workspace"),
            &t!("gtk_converters.move_to_workspace_silent"),
            &t!("gtk_converters.toggle_floating"),
            &t!("gtk_converters.set_floating"),
            &t!("gtk_converters.set_tiled"),
            &t!("gtk_converters.fullscreen"),
            &t!("gtk_converters.fullscreen_state"),
            &t!("gtk_converters.dpms"),
            &t!("gtk_converters.pin"),
            &t!("gtk_converters.move_focus"),
            &t!("gtk_converters.move_window"),
            &t!("gtk_converters.swap_window"),
            &t!("gtk_converters.center_window"),
            &t!("gtk_converters.resize_active"),
            &t!("gtk_converters.move_active"),
            &t!("gtk_converters.resize_window_pixel"),
            &t!("gtk_converters.move_window_pixel"),
            &t!("gtk_converters.cycle_next"),
            &t!("gtk_converters.swap_next"),
            &t!("gtk_converters.tag_window"),
            &t!("gtk_converters.focus_window"),
            &t!("gtk_converters.focus_monitor"),
            &t!("gtk_converters.split_ratio"),
            &t!("gtk_converters.move_cursor_to_corner"),
            &t!("gtk_converters.move_cursor"),
            &t!("gtk_converters.rename_workspace"),
            &t!("gtk_converters.exit"),
            &t!("gtk_converters.force_renderer_reload"),
            &t!("gtk_converters.move_current_workspace_to_monitor"),
            &t!("gtk_converters.focus_workspace_on_current_monitor"),
            &t!("gtk_converters.move_workspace_to_monitor"),
            &t!("gtk_converters.swap_active_workspaces"),
            &t!("gtk_converters.bring_active_to_top"),
            &t!("gtk_converters.alter_z_order"),
            &t!("gtk_converters.toggle_special_workspace"),
            &t!("gtk_converters.focus_urgent_or_last"),
            &t!("gtk_converters.toggle_group"),
            &t!("gtk_converters.change_group_active"),
            &t!("gtk_converters.focus_current_or_last"),
            &t!("gtk_converters.lock_groups"),
            &t!("gtk_converters.lock_active_group"),
            &t!("gtk_converters.move_into_group"),
            &t!("gtk_converters.move_out_of_group"),
            &t!("gtk_converters.move_window_or_group"),
            &t!("gtk_converters.move_group_window"),
            &t!("gtk_converters.deny_window_from_group"),
            &t!("gtk_converters.set_ignore_group_lock"),
            &t!("gtk_converters.global"),
            &t!("gtk_converters.event"),
            &t!("gtk_converters.set_prop"),
            &t!("gtk_converters.toggle_swallow"),
        ])
    }

    const SEPARATOR: Option<char> = Some(' ');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::Exec(_window_rules, _command) => Some(|entry, separator, _names, _| {
                let is_updating = Rc::new(Cell::new(false));
                let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                let window_rules_mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
                window_rules_mother_box
                    .append(&Label::new(Some(&t!("gtk_converters.window_rules"))));
                let window_rules_entry = create_entry();
                let window_rules_box = Vec::<WindowRule>::to_gtk_box(&window_rules_entry, ';');
                window_rules_mother_box.append(&window_rules_box);
                mother_box.append(&window_rules_mother_box);

                let command_entry = create_entry();
                mother_box.append(&command_entry);

                let window_rules_entry_clone = window_rules_entry.clone();
                let command_entry_clone = command_entry.clone();
                let update_ui = move |(window_rules, command): (Vec<WindowRule>, String)| {
                    window_rules_entry_clone.set_text(&join_with_separator(&window_rules, ";"));
                    command_entry_clone.set_text(&command);
                };

                let parse_value = |str: &str| {
                    let dispatcher =
                        Self::from_discriminant_and_str(DispatcherDiscriminant::Exec, str);
                    match dispatcher {
                        Dispatcher::Exec(window_rules, command) => (window_rules, command),
                        _ => (vec![], String::new()),
                    }
                };

                update_ui(parse_value(entry.text().as_str()));

                let command_entry_clone = command_entry.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                window_rules_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);

                    match entry.text().is_empty() {
                        true => entry_clone.set_text(&command_entry_clone.text()),
                        false => entry_clone.set_text(&format!(
                            "[{}]{}{}",
                            entry.text(),
                            separator,
                            command_entry_clone.text()
                        )),
                    }

                    is_updating_clone.set(false);
                });

                let window_rules_entry_clone = window_rules_entry.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                command_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);

                    match window_rules_entry_clone.text().is_empty() {
                        true => entry_clone.set_text(&entry.text()),
                        false => entry_clone.set_text(&format!(
                            "[{}]{}{}",
                            window_rules_entry_clone.text(),
                            separator,
                            entry.text()
                        )),
                    }

                    is_updating_clone.set(false);
                });

                let is_updating_clone = is_updating.clone();
                entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let (window_rules, command) = parse_value(entry.text().as_str());
                    update_ui((window_rules, command));
                    is_updating_clone.set(false);
                });

                mother_box
            }),
            Self::Execr(_command) => Some(<(String,)>::to_gtk_box),
            Self::Pass(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
            Self::SendShortcut(_modifiers, _key, _window_target) => {
                Some(|entry, separator, _names, _| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let modifiers_entry = create_entry();
                    let modifiers_box = HashSet::<Modifier>::to_gtk_box(&modifiers_entry, '_');
                    mother_box.append(&modifiers_box);

                    let key_entry = create_entry();
                    mother_box.append(&key_entry);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box
                        .append(&Label::new(Some(&t!("gtk_converters.window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(modifiers, key, window_target): (
                        HashSet<Modifier>,
                        String,
                        Option<WindowTarget>,
                    )| {
                        modifiers_entry_clone.set_text(&join_with_separator(&modifiers, "_"));
                        key_entry_clone.set_text(&key);
                        let window_target_str = match window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::SendShortcut,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::SendShortcut(modifiers, key, window_target) => {
                                (modifiers, key, window_target)
                            }
                            _ => (HashSet::new(), String::new(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let key_entry_clone = key_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    modifiers_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => {
                                format!("{}{}{}", entry.text(), separator, key_entry_clone.text())
                            }
                            window_target_str => format!(
                                "{}{}{}{}{}",
                                entry.text(),
                                separator,
                                key_entry_clone.text(),
                                separator,
                                window_target_str
                            ),
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    key_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => {
                                format!(
                                    "{}{}{}",
                                    modifiers_entry_clone.text(),
                                    separator,
                                    entry.text()
                                )
                            }
                            window_target_str => format!(
                                "{}{}{}{}{}",
                                modifiers_entry_clone.text(),
                                separator,
                                entry.text(),
                                separator,
                                window_target_str
                            ),
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => {
                                format!(
                                    "{}{}{}",
                                    modifiers_entry_clone.text(),
                                    separator,
                                    key_entry_clone.text()
                                )
                            }
                            window_target_str => format!(
                                "{}{}{}{}{}",
                                modifiers_entry_clone.text(),
                                separator,
                                key_entry_clone.text(),
                                separator,
                                window_target_str
                            ),
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (modifiers, key, window_target) = parse_value(entry.text().as_str());
                        update_ui((modifiers, key, window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::SendKeyState(_modifiers, _key, _state, _window_target) => {
                Some(|entry, separator, _names, _| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let modifiers_entry = create_entry();
                    let modifiers_box = HashSet::<Modifier>::to_gtk_box(&modifiers_entry, '_');
                    mother_box.append(&modifiers_box);

                    let key_entry = create_entry();
                    mother_box.append(&key_entry);

                    let state_entry = create_entry();
                    let state_box = KeyState::to_gtk_box(&state_entry);
                    mother_box.append(&state_box);

                    let window_target_entry = create_entry();
                    let window_target_box = WindowTarget::to_gtk_box(&window_target_entry);
                    mother_box.append(&window_target_box);

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let state_entry_clone = state_entry.clone();
                    let window_target_entry_clone = window_target_entry.clone();
                    let update_ui = move |(modifiers, key, state, window_target): (
                        HashSet<Modifier>,
                        String,
                        KeyState,
                        WindowTarget,
                    )| {
                        modifiers_entry_clone.set_text(&join_with_separator(&modifiers, "_"));
                        key_entry_clone.set_text(&key);
                        state_entry_clone.set_text(&state.to_string());
                        window_target_entry_clone.set_text(&window_target.to_string());
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::SendKeyState,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::SendKeyState(modifiers, key, state, window_target) => {
                                (modifiers, key, state, window_target)
                            }
                            _ => (
                                HashSet::new(),
                                String::new(),
                                KeyState::default(),
                                WindowTarget::default(),
                            ),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let key_entry_clone = key_entry.clone();
                    let state_entry_clone = state_entry.clone();
                    let window_target_entry_clone = window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    modifiers_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        entry_clone.set_text(&format!(
                            "{}{}{}{}{}{}{}",
                            entry.text(),
                            separator,
                            key_entry_clone.text(),
                            separator,
                            state_entry_clone.text(),
                            separator,
                            window_target_entry_clone.text()
                        ));
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let state_entry_clone = state_entry.clone();
                    let window_target_entry_clone = window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    key_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        entry_clone.set_text(&format!(
                            "{}{}{}{}{}{}{}",
                            modifiers_entry_clone.text(),
                            separator,
                            entry.text(),
                            separator,
                            state_entry_clone.text(),
                            separator,
                            window_target_entry_clone.text()
                        ));
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let window_target_entry_clone = window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    state_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        entry_clone.set_text(&format!(
                            "{}{}{}{}{}{}{}",
                            modifiers_entry_clone.text(),
                            separator,
                            key_entry_clone.text(),
                            separator,
                            entry.text(),
                            separator,
                            window_target_entry_clone.text()
                        ));
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let state_entry_clone = state_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        entry_clone.set_text(&format!(
                            "{}{}{}{}{}{}{}",
                            modifiers_entry_clone.text(),
                            separator,
                            key_entry_clone.text(),
                            separator,
                            state_entry_clone.text(),
                            separator,
                            entry.text()
                        ));
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (modifiers, key, state, window_target) =
                            parse_value(entry.text().as_str());
                        update_ui((modifiers, key, state, window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::KillActive => None,
            Self::ForceKillActive => None,
            Self::CloseWindow(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
            Self::KillWindow(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
            Self::Signal(_signal) => Some(<(String,)>::to_gtk_box),
            Self::SignalWindow(_window_target, _signal) => {
                Some(<(WindowTarget, String)>::to_gtk_box)
            }
            Self::Workspace(_workspace_target) => Some(<(WorkspaceTarget,)>::to_gtk_box),
            Self::MoveToWorkspace(_workspace_target, _optional_window_target) => {
                Some(|entry, separator, _names, _| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let workspace_target_entry = create_entry();
                    let workspace_target_box = WorkspaceTarget::to_gtk_box(&workspace_target_entry);
                    mother_box.append(&workspace_target_box);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box
                        .append(&Label::new(Some(&t!("gtk_converters.window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let workspace_target_entry_clone = workspace_target_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(workspace_target, optional_window_target): (
                        WorkspaceTarget,
                        Option<WindowTarget>,
                    )| {
                        workspace_target_entry_clone.set_text(&workspace_target.to_string());
                        let window_target_str = match optional_window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::MoveToWorkspace,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::MoveToWorkspace(workspace_target, window_target) => {
                                (workspace_target, window_target)
                            }
                            _ => (WorkspaceTarget::default(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    workspace_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => entry.text().to_string(),
                            window_target => {
                                format!("{}{}{}", entry.text(), separator, window_target)
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let workspace_target_entry_clone = workspace_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => workspace_target_entry_clone.text().to_string(),
                            window_target => {
                                format!(
                                    "{}{}{}",
                                    workspace_target_entry_clone.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (workspace_target, optional_window_target) =
                            parse_value(entry.text().as_str());
                        update_ui((workspace_target, optional_window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::MoveToWorkspaceSilent(_workspace_target, _optional_window_target) => {
                Some(|entry, separator, _names, _| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let workspace_target_entry = create_entry();
                    let workspace_target_box = WorkspaceTarget::to_gtk_box(&workspace_target_entry);
                    mother_box.append(&workspace_target_box);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box
                        .append(&Label::new(Some(&t!("gtk_converters.window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let workspace_target_entry_clone = workspace_target_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(workspace_target, optional_window_target): (
                        WorkspaceTarget,
                        Option<WindowTarget>,
                    )| {
                        workspace_target_entry_clone.set_text(&workspace_target.to_string());
                        let window_target_str = match optional_window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::MoveToWorkspaceSilent,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::MoveToWorkspaceSilent(workspace_target, window_target) => {
                                (workspace_target, window_target)
                            }
                            _ => (WorkspaceTarget::default(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    workspace_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => entry.text().to_string(),
                            window_target => {
                                format!("{}{}{}", entry.text(), separator, window_target)
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let workspace_target_entry_clone = workspace_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => workspace_target_entry_clone.text().to_string(),
                            window_target => {
                                format!(
                                    "{}{}{}",
                                    workspace_target_entry_clone.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (workspace_target, optional_window_target) =
                            parse_value(entry.text().as_str());
                        update_ui((workspace_target, optional_window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::ToggleFloating(_optional_window_target)
            | Self::SetFloating(_optional_window_target)
            | Self::SetTiled(_optional_window_target) => {
                Some(|entry, _separator, _names, _| Option::<WindowTarget>::to_gtk_box(entry))
            }
            Self::Fullscreen(_fullscreen_mode) => Some(<(FullscreenMode,)>::to_gtk_box),
            Self::FullscreenState(_fullscreen_state1, _fullscreen_state2) => {
                Some(<(DispatcherFullscreenState, DispatcherFullscreenState)>::to_gtk_box)
            }
            Self::Dpms(_toggle_state, _optional_monitor_name) => {
                Some(|entry, separator, _names, _| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let toggle_state_entry = create_entry();
                    let toggle_state_box = ToggleState::to_gtk_box(&toggle_state_entry);
                    mother_box.append(&toggle_state_box);

                    let monitor_name_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    monitor_name_box.append(&Label::new(Some(&t!("gtk_converters.monitor_name"))));
                    let optional_monitor_name_entry = create_entry();
                    let optional_monitor_name_box =
                        Option::<String>::to_gtk_box(&optional_monitor_name_entry);
                    monitor_name_box.append(&optional_monitor_name_box);
                    mother_box.append(&monitor_name_box);

                    let toggle_state_entry_clone = toggle_state_entry.clone();
                    let optional_monitor_name_entry_clone = optional_monitor_name_entry.clone();
                    let update_ui =
                        move |(toggle_state, monitor_name): (ToggleState, Option<String>)| {
                            toggle_state_entry_clone.set_text(&toggle_state.to_string());
                            let monitor_name_str = monitor_name.unwrap_or_default();
                            optional_monitor_name_entry_clone.set_text(&monitor_name_str);
                        };

                    let parse_value = |str: &str| {
                        let dispatcher =
                            Self::from_discriminant_and_str(DispatcherDiscriminant::Dpms, str);
                        match dispatcher {
                            Dispatcher::Dpms(toggle_state, monitor_name) => {
                                (toggle_state, monitor_name)
                            }
                            _ => (ToggleState::default(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let optional_monitor_name_entry_clone = optional_monitor_name_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    toggle_state_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_monitor_name_entry_clone.text().as_str() {
                            "" => entry.text().to_string(),
                            monitor_name => {
                                format!("{}{}{}", entry.text(), separator, monitor_name)
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let toggle_state_entry_clone = toggle_state_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_monitor_name_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => toggle_state_entry_clone.text().to_string(),
                            monitor_name => {
                                format!(
                                    "{}{}{}",
                                    toggle_state_entry_clone.text(),
                                    separator,
                                    monitor_name
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (toggle_state, monitor_name) = parse_value(entry.text().as_str());
                        update_ui((toggle_state, monitor_name));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::Pin(_optional_window_target) => {
                Some(|entry, _separator, _names, _| Option::<WindowTarget>::to_gtk_box(entry))
            }
            Self::MoveFocus(_direction) => Some(<(Direction,)>::to_gtk_box),
            Self::MoveWindow(_move_direction) => Some(<(MoveDirection,)>::to_gtk_box),
            Self::SwapWindow(_swap_direction) => Some(<(SwapDirection,)>::to_gtk_box),
            Self::CenterWindow(_respect_monitor_reserved_area) => Some(<(bool,)>::to_gtk_box),
            Self::ResizeActive(_resize_params) => Some(<(ResizeParams,)>::to_gtk_box),
            Self::MoveActive(_resize_params) => Some(<(ResizeParams,)>::to_gtk_box),
            Self::ResizeWindowPixel(_resize_params, _window_target)
            | Self::MoveWindowPixel(_resize_params, _window_target) => {
                Some(<(ResizeParams, WindowTarget)>::to_gtk_box)
            }
            Self::CycleNext(_cycle_next) => Some(<(CycleNext,)>::to_gtk_box),
            Self::SwapNext(_swap_next) => Some(<(SwapNext,)>::to_gtk_box),
            Self::TagWindow(_tag_toggle_state, _tag, _optional_window_target) => {
                Some(|entry, separator, _names, _| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let tag_toggle_state_entry = create_entry();
                    let tag_toggle_state_box = TagToggleState::to_gtk_box(&tag_toggle_state_entry);
                    mother_box.append(&tag_toggle_state_box);

                    let tag_entry = create_entry();
                    mother_box.append(&tag_entry);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box
                        .append(&Label::new(Some(&t!("gtk_converters.window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let tag_toggle_state_entry_clone = tag_toggle_state_entry.clone();
                    let tag_entry_clone = tag_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(tag_toggle_state, tag, optional_window_target): (
                        TagToggleState,
                        String,
                        Option<WindowTarget>,
                    )| {
                        tag_toggle_state_entry_clone.set_text(&tag_toggle_state.to_string());
                        tag_entry_clone.set_text(&tag);
                        let window_target_str = match optional_window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher =
                            Self::from_discriminant_and_str(DispatcherDiscriminant::TagWindow, str);
                        match dispatcher {
                            Dispatcher::TagWindow(tag_toggle_state, tag, window_target) => {
                                (tag_toggle_state, tag, window_target)
                            }
                            _ => (TagToggleState::default(), String::new(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let tag_entry_clone = tag_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    tag_toggle_state_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => {
                                format!("{}{}{}", entry.text(), separator, tag_entry_clone.text())
                            }
                            window_target => format!(
                                "{}{}{}{}{}",
                                entry.text(),
                                separator,
                                tag_entry_clone.text(),
                                separator,
                                window_target
                            ),
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let tag_toggle_state_entry_clone = tag_toggle_state_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    tag_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => {
                                format!(
                                    "{}{}{}",
                                    tag_toggle_state_entry_clone.text(),
                                    separator,
                                    entry.text()
                                )
                            }
                            window_target => {
                                format!(
                                    "{}{}{}{}{}",
                                    tag_toggle_state_entry_clone.text(),
                                    separator,
                                    entry.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let tag_toggle_state_entry_clone = tag_toggle_state_entry.clone();
                    let tag_entry_clone = tag_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => {
                                format!(
                                    "{}{}{}",
                                    tag_toggle_state_entry_clone.text(),
                                    separator,
                                    tag_entry_clone.text()
                                )
                            }
                            window_target => {
                                format!(
                                    "{}{}{}{}{}",
                                    tag_toggle_state_entry_clone.text(),
                                    separator,
                                    tag_entry_clone.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (tag_toggle_state, tag, optional_window_target) =
                            parse_value(entry.text().as_str());
                        update_ui((tag_toggle_state, tag, optional_window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::FocusWindow(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
            Self::FocusMonitor(_monitor_target) => Some(<(MonitorTarget,)>::to_gtk_box),
            Self::SplitRatio(_float_value) => Some(<(FloatValue,)>::to_gtk_box),
            Self::MoveCursorToCorner(_cursor_corner) => Some(<(CursorCorner,)>::to_gtk_box),
            Self::MoveCursor(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            Self::RenameWorkspace(_id, _new_name) => Some(<(u32, String)>::to_gtk_box),
            Self::Exit => None,
            Self::ForceRendererReload => None,
            Self::MoveCurrentWorkspaceToMonitor(_monitor_target) => {
                Some(<(MonitorTarget,)>::to_gtk_box)
            }
            Self::FocusWorkspaceOnCurrentMonitor(_workspace_target) => {
                Some(<(WorkspaceTarget,)>::to_gtk_box)
            }
            Self::MoveWorkspaceToMonitor(_workspace_target, _monitor_target) => {
                Some(<(WorkspaceTarget, MonitorTarget)>::to_gtk_box)
            }
            Self::SwapActiveWorkspaces(_monitor_target1, _monitor_target2) => {
                Some(<(MonitorTarget, MonitorTarget)>::to_gtk_box)
            }
            Self::BringActiveToTop => None,
            Self::AlterZOrder(_z_height, _optional_window_target) => {
                Some(|entry, separator, _names, _| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let z_height_entry = create_entry();
                    let z_height_box = ZHeight::to_gtk_box(&z_height_entry);
                    mother_box.append(&z_height_box);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box
                        .append(&Label::new(Some(&t!("gtk_converters.window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let z_height_entry_clone = z_height_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(z_height, optional_window_target): (
                        ZHeight,
                        Option<WindowTarget>,
                    )| {
                        z_height_entry_clone.set_text(&z_height.to_string());
                        let window_target_str = match optional_window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::AlterZOrder,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::AlterZOrder(z_height, optional_window_target) => {
                                (z_height, optional_window_target)
                            }
                            _ => (ZHeight::default(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    z_height_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => entry.text().to_string(),
                            window_target => {
                                format!("{}{}{}", entry.text(), separator, window_target)
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let z_height_entry_clone = z_height_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => z_height_entry_clone.text().to_string(),
                            window_target => {
                                format!(
                                    "{}{}{}",
                                    z_height_entry_clone.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (z_height, optional_window_target) = parse_value(entry.text().as_str());
                        update_ui((z_height, optional_window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::ToggleSpecialWorkspace(_optional_name) => {
                Some(|entry, _separator, _names, _| Option::<String>::to_gtk_box(entry))
            }
            Self::FocusUrgentOrLast => None,
            Self::ToggleGroup => None,
            Self::ChangeGroupActive(_change_group_active) => {
                Some(<(ChangeGroupActive,)>::to_gtk_box)
            }
            Self::FocusCurrentOrLast => None,
            Self::LockGroups(_group_lock_action) => Some(<(GroupLockAction,)>::to_gtk_box),
            Self::LockActiveGroup(_group_lock_action) => Some(<(GroupLockAction,)>::to_gtk_box),
            Self::MoveIntoGroup(_direction) => Some(<(Direction,)>::to_gtk_box),
            Self::MoveOutOfGroup(_optional_window_target) => {
                Some(|entry, _separator, _names, _| Option::<WindowTarget>::to_gtk_box(entry))
            }
            Self::MoveWindowOrGroup(_direction) => Some(<(Direction,)>::to_gtk_box),
            Self::MoveGroupWindow(_is_back) => Some(<(bool,)>::to_gtk_box),
            Self::DenyWindowFromGroup(_toggle_state) => Some(<(ToggleState,)>::to_gtk_box),
            Self::SetIgnoreGroupLock(_toggle_state) => Some(<(ToggleState,)>::to_gtk_box),
            Self::Global(_name) => Some(<(String,)>::to_gtk_box),
            Self::Event(_data) => Some(<(String,)>::to_gtk_box),
            Self::SetProp(_set_prop) => Some(<(SetProp,)>::to_gtk_box),
            Self::ToggleSwallow => None,
        }
    }

    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        Some(vec![
            // Exec(Vec<WindowRule>, String),
            vec![],
            // Execr(String),
            vec![FieldLabel::Unnamed],
            // Pass(WindowTarget),
            vec![FieldLabel::Unnamed],
            // SendShortcut(HashSet<Modifier>, String, Option<WindowTarget>),
            vec![],
            // SendKeyState(HashSet<Modifier>, String, KeyState, WindowTarget),
            vec![],
            // KillActive,
            vec![],
            // ForceKillActive,
            vec![],
            // CloseWindow(WindowTarget),
            vec![FieldLabel::Unnamed],
            // KillWindow(WindowTarget),
            vec![FieldLabel::Unnamed],
            // Signal(String),
            vec![FieldLabel::Unnamed],
            // SignalWindow(WindowTarget, String),
            vec![FieldLabel::Unnamed, FieldLabel::Unnamed],
            // Workspace(WorkspaceTarget),
            vec![FieldLabel::Unnamed],
            // MoveToWorkspace(WorkspaceTarget, Option<WindowTarget>),
            vec![],
            // MoveToWorkspaceSilent(WorkspaceTarget, Option<WindowTarget>),
            vec![],
            // ToggleFloating(Option<WindowTarget>),
            vec![],
            // SetFloating(Option<WindowTarget>),
            vec![],
            // SetTiled(Option<WindowTarget>),
            vec![],
            // Fullscreen(FullscreenMode),
            vec![FieldLabel::Named(cow_to_static_str(t!(
                "gtk_converters.fullscreen_mode"
            )))],
            // FullscreenState(DispatcherFullscreenState, DispatcherFullscreenState),
            vec![
                FieldLabel::Named(cow_to_static_str(t!("gtk_converters.internal_state"))),
                FieldLabel::Named(cow_to_static_str(t!("gtk_converters.client_state"))),
            ],
            // other options does not need to be labelled
        ])
    }
}

register_togtkbox!(Dispatcher);
register_togtkbox_with_separator!(Vec<WindowRule>, HashSet<Modifier>);
register_togtkbox_with_separator_names!(
    (String,),
    (WindowTarget,),
    (WindowTarget, String),
    (WorkspaceTarget,),
    (FullscreenMode,),
    (DispatcherFullscreenState, DispatcherFullscreenState),
    (Direction,),
    (MoveDirection,),
    (SwapDirection,),
    (bool,),
    (ResizeParams,),
    (ResizeParams, WindowTarget),
    (CycleNext,),
    (SwapNext,),
    (MonitorTarget,),
    (FloatValue,),
    (CursorCorner,),
    (u32, u32),
    (u32, String),
    (WorkspaceTarget, MonitorTarget),
    (MonitorTarget, MonitorTarget),
    (ChangeGroupActive,),
    (GroupLockAction,),
    (ToggleState,),
    (SetProp,),
);
