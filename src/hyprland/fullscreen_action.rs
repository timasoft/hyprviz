use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum FullscreenAction {
    #[default]
    Toggle,
    Set,
    Unset,
}

impl FromStr for FullscreenAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "toggle" => Ok(FullscreenAction::Toggle),
            "set" => Ok(FullscreenAction::Set),
            "unset" => Ok(FullscreenAction::Unset),
            _ => Err(()),
        }
    }
}

impl Display for FullscreenAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FullscreenAction::Toggle => write!(f, "toggle"),
            FullscreenAction::Set => write!(f, "set"),
            FullscreenAction::Unset => write!(f, "unset"),
        }
    }
}

impl EnumConfigForGtk for FullscreenAction {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.fullscreen_action.toggle"),
            &t!("hyprland.fullscreen_action.set"),
            &t!("hyprland.fullscreen_action.unset"),
        ])
    }
}

register_togtkbox!(FullscreenAction);
