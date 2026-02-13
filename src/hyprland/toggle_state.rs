use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum ToggleState {
    On,
    Off,
    #[default]
    Toggle,
}

impl FromStr for ToggleState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "on" => Ok(ToggleState::On),
            "off" => Ok(ToggleState::Off),
            "toggle" => Ok(ToggleState::Toggle),
            _ => Err(()),
        }
    }
}

impl Display for ToggleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToggleState::On => write!(f, "on"),
            ToggleState::Off => write!(f, "off"),
            ToggleState::Toggle => write!(f, "toggle"),
        }
    }
}

impl EnumConfigForGtk for ToggleState {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.toggle_state.on"),
            &t!("hyprland.toggle_state.off"),
            &t!("hyprland.toggle_state.toggle"),
        ])
    }
}

register_togtkbox!(ToggleState);
