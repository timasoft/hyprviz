use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum GestureFullscreen {
    #[default]
    Fullscreen,
    Maximize,
}

impl FromStr for GestureFullscreen {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "" | "fullscreen" => Ok(GestureFullscreen::Fullscreen),
            "maximize" => Ok(GestureFullscreen::Maximize),
            _ => Err(()),
        }
    }
}

impl Display for GestureFullscreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestureFullscreen::Fullscreen => write!(f, ""),
            GestureFullscreen::Maximize => write!(f, "maximize"),
        }
    }
}

impl EnumConfigForGtk for GestureFullscreen {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.fullscreen"),
            &t!("gtk_converters.maximize"),
        ])
    }
}

register_togtkbox!(GestureFullscreen);
