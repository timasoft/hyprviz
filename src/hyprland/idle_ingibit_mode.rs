use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum IdleIngibitMode {
    #[default]
    None,
    Always,
    Focus,
    Fullscreen,
}

impl FromStr for IdleIngibitMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "none" => Ok(IdleIngibitMode::None),
            "always" => Ok(IdleIngibitMode::Always),
            "focus" => Ok(IdleIngibitMode::Focus),
            "fullscreen" => Ok(IdleIngibitMode::Fullscreen),
            _ => Err(()),
        }
    }
}

impl Display for IdleIngibitMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdleIngibitMode::None => write!(f, "none"),
            IdleIngibitMode::Always => write!(f, "always"),
            IdleIngibitMode::Focus => write!(f, "focus"),
            IdleIngibitMode::Fullscreen => write!(f, "fullscreen"),
        }
    }
}

impl EnumConfigForGtk for IdleIngibitMode {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.none"),
            &t!("gtk_converters.always"),
            &t!("gtk_converters.focus"),
            &t!("gtk_converters.fullscreen"),
        ])
    }
}

register_togtkbox!(IdleIngibitMode);
