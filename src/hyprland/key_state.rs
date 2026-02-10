use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum KeyState {
    #[default]
    Down,
    Repeat,
    Up,
}

impl FromStr for KeyState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "down" => Ok(KeyState::Down),
            "repeat" => Ok(KeyState::Repeat),
            "up" => Ok(KeyState::Up),
            _ => Err(()),
        }
    }
}

impl Display for KeyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyState::Down => write!(f, "down"),
            KeyState::Repeat => write!(f, "repeat"),
            KeyState::Up => write!(f, "up"),
        }
    }
}

impl EnumConfigForGtk for KeyState {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.down"),
            &t!("gtk_converters.repeat"),
            &t!("gtk_converters.up"),
        ])
    }
}

register_togtkbox!(KeyState);
