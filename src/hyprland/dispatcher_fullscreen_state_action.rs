use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum DispatcherFullscreenStateAction {
    #[default]
    Toggle,
    Set,
}

impl FromStr for DispatcherFullscreenStateAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "toggle" => Ok(DispatcherFullscreenStateAction::Toggle),
            "set" => Ok(DispatcherFullscreenStateAction::Set),
            _ => Err(()),
        }
    }
}

impl Display for DispatcherFullscreenStateAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DispatcherFullscreenStateAction::Toggle => write!(f, "toggle"),
            DispatcherFullscreenStateAction::Set => write!(f, "set"),
        }
    }
}

impl EnumConfigForGtk for DispatcherFullscreenStateAction {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.fullscreen_action.toggle"),
            &t!("hyprland.fullscreen_action.set"),
        ])
    }
}

register_togtkbox!(DispatcherFullscreenStateAction);
