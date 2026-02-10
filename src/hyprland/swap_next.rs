use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum SwapNext {
    #[default]
    Next,
    Prev,
}

impl FromStr for SwapNext {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "prev" => Ok(SwapNext::Prev),
            _ => Ok(SwapNext::Next),
        }
    }
}

impl Display for SwapNext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapNext::Next => write!(f, ""),
            SwapNext::Prev => write!(f, "prev"),
        }
    }
}

impl EnumConfigForGtk for SwapNext {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("gtk_converters.next"), &t!("gtk_converters.prev")])
    }
}

register_togtkbox!(SwapNext);
