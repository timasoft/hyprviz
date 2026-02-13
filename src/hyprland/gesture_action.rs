use super::{Dispatcher, GestureFloating, GestureFullscreen};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, ToGtkBoxWithSeparatorAndNames, ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(GestureActionDiscriminant))]
pub enum GestureAction {
    Dispatcher(Dispatcher),
    Workspace,
    Move,
    Resize,
    Special(String),
    Close,
    Fullscreen(GestureFullscreen),
    Float(GestureFloating),
}

impl HasDiscriminant for GestureAction {
    type Discriminant = GestureActionDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            GestureActionDiscriminant::Dispatcher => {
                GestureAction::Dispatcher(Dispatcher::default())
            }
            GestureActionDiscriminant::Workspace => GestureAction::Workspace,
            GestureActionDiscriminant::Move => GestureAction::Move,
            GestureActionDiscriminant::Resize => GestureAction::Resize,
            GestureActionDiscriminant::Special => GestureAction::Special(String::default()),
            GestureActionDiscriminant::Close => GestureAction::Close,
            GestureActionDiscriminant::Fullscreen => {
                GestureAction::Fullscreen(GestureFullscreen::Fullscreen)
            }
            GestureActionDiscriminant::Float => GestureAction::Float(GestureFloating::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            GestureActionDiscriminant::Dispatcher => {
                GestureAction::Dispatcher(Dispatcher::from_str(str).unwrap_or_default())
            }
            GestureActionDiscriminant::Workspace => GestureAction::Workspace,
            GestureActionDiscriminant::Move => GestureAction::Move,
            GestureActionDiscriminant::Resize => GestureAction::Resize,
            GestureActionDiscriminant::Special => GestureAction::Special(str.to_string()),
            GestureActionDiscriminant::Close => GestureAction::Close,
            GestureActionDiscriminant::Fullscreen => {
                GestureAction::Fullscreen(GestureFullscreen::from_str(str).unwrap_or_default())
            }
            GestureActionDiscriminant::Float => {
                GestureAction::Float(GestureFloating::from_str(str).unwrap_or_default())
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            GestureAction::Dispatcher(dispatcher) => Some(dispatcher.to_string()),
            GestureAction::Workspace => None,
            GestureAction::Move => None,
            GestureAction::Resize => None,
            GestureAction::Special(special) => Some(special.to_string()),
            GestureAction::Close => None,
            GestureAction::Fullscreen(fullscreen) => Some(fullscreen.to_string()),
            GestureAction::Float(floating) => Some(floating.to_string()),
        }
    }
}

impl Default for GestureAction {
    fn default() -> Self {
        GestureAction::Dispatcher(Dispatcher::default())
    }
}

impl FromStr for GestureAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (action, arguments) = s.split_once(',').unwrap_or((s, ""));
        let action = action.trim();
        let arguments = arguments.trim();

        match action {
            "dispatcher" => Ok(GestureAction::Dispatcher(
                Dispatcher::from_str(arguments).unwrap_or_default(),
            )),
            "workspace" => Ok(GestureAction::Workspace),
            "move" => Ok(GestureAction::Move),
            "resize" => Ok(GestureAction::Resize),
            "special" => Ok(GestureAction::Special(arguments.to_string())),
            "close" => Ok(GestureAction::Close),
            "fullscreen" => Ok(GestureAction::Fullscreen(
                GestureFullscreen::from_str(arguments).unwrap_or_default(),
            )),
            "float" => Ok(GestureAction::Float(
                GestureFloating::from_str(arguments).unwrap_or_default(),
            )),
            _ => Err(()),
        }
    }
}

impl Display for GestureAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestureAction::Dispatcher(dispatcher) => write!(f, "dispatcher, {}", dispatcher),
            GestureAction::Workspace => write!(f, "workspace"),
            GestureAction::Move => write!(f, "move"),
            GestureAction::Resize => write!(f, "resize"),
            GestureAction::Special(special) => write!(f, "special, {}", special),
            GestureAction::Close => write!(f, "close"),
            GestureAction::Fullscreen(fullscreen) => write!(f, "fullscreen, {}", fullscreen),
            GestureAction::Float(floating) => write!(f, "float, {}", floating),
        }
    }
}

impl EnumConfigForGtk for GestureAction {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.gesture_action.dispatcher"),
            &t!("hyprland.gesture_action.workspace"),
            &t!("hyprland.gesture_action.move"),
            &t!("hyprland.gesture_action.resize"),
            &t!("hyprland.gesture_action.special"),
            &t!("hyprland.gesture_action.close"),
            &t!("hyprland.gesture_action.fullscreen"),
            &t!("hyprland.gesture_action.float"),
        ])
    }

    const SEPARATOR: Option<char> = Some(',');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            GestureAction::Dispatcher(_dispatcher) => Some(<(Dispatcher,)>::to_gtk_box),
            GestureAction::Workspace => None,
            GestureAction::Move => None,
            GestureAction::Resize => None,
            GestureAction::Special(_special) => Some(<(String,)>::to_gtk_box),
            GestureAction::Close => None,
            GestureAction::Fullscreen(_fullscreen) => Some(<(GestureFullscreen,)>::to_gtk_box),
            GestureAction::Float(_floating) => Some(<(GestureFloating,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(GestureAction);
register_togtkbox_with_separator_names!((Dispatcher,), (GestureFullscreen,), (GestureFloating,));
