use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum TagToggleState {
    Set,
    Unset,
    #[default]
    Toggle,
}

impl FromStr for TagToggleState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "+" => Ok(Self::Set),
            "-" => Ok(Self::Unset),
            _ => Ok(Self::Toggle),
        }
    }
}

impl Display for TagToggleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagToggleState::Set => write!(f, "+"),
            TagToggleState::Unset => write!(f, "-"),
            TagToggleState::Toggle => write!(f, ""),
        }
    }
}

impl EnumConfigForGtk for TagToggleState {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.set"),
            &t!("gtk_converters.unset"),
            &t!("gtk_converters.toggle"),
        ])
    }
}

register_togtkbox!(TagToggleState);
