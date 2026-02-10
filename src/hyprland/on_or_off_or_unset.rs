use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox, utils::parse_bool};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum OnOrOffOrUnset {
    #[default]
    Unset,
    On,
    Off,
}

impl FromStr for OnOrOffOrUnset {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_bool(s) {
            Some(true) => Ok(OnOrOffOrUnset::On),
            Some(false) => Ok(OnOrOffOrUnset::Off),
            None => Ok(OnOrOffOrUnset::Unset),
        }
    }
}

impl Display for OnOrOffOrUnset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnOrOffOrUnset::Unset => write!(f, "unset"),
            OnOrOffOrUnset::On => write!(f, "1"),
            OnOrOffOrUnset::Off => write!(f, "0"),
        }
    }
}

impl EnumConfigForGtk for OnOrOffOrUnset {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.unset"),
            &t!("gtk_converters.on"),
            &t!("gtk_converters.off"),
        ])
    }
}

register_togtkbox!(OnOrOffOrUnset);
