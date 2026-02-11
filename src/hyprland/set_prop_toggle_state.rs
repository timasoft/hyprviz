use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox, utils::parse_bool};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum SetPropToggleState {
    Off,
    On,
    #[default]
    Toggle,
    Unset,
}

impl FromStr for SetPropToggleState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        match parse_bool(&s.to_lowercase()) {
            Some(true) => Ok(SetPropToggleState::On),
            Some(false) => Ok(SetPropToggleState::Off),
            None => match s.to_lowercase().as_str() {
                "toggle" => Ok(SetPropToggleState::Toggle),
                "unset" => Ok(SetPropToggleState::Unset),
                _ => Err(()),
            },
        }
    }
}

impl Display for SetPropToggleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetPropToggleState::Off => write!(f, "0"),
            SetPropToggleState::On => write!(f, "1"),
            SetPropToggleState::Toggle => write!(f, "toggle"),
            SetPropToggleState::Unset => write!(f, "unset"),
        }
    }
}

impl EnumConfigForGtk for SetPropToggleState {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.set_prop_toggle_state.on"),
            &t!("hyprland.set_prop_toggle_state.off"),
            &t!("hyprland.set_prop_toggle_state.toggle"),
            &t!("hyprland.set_prop_toggle_state.unset"),
        ])
    }
}

register_togtkbox!(SetPropToggleState);
