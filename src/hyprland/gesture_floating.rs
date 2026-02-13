use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum GestureFloating {
    #[default]
    Toggle,
    Float,
    Tile,
}

impl FromStr for GestureFloating {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "" | "toggle" => Ok(GestureFloating::Toggle),
            "float" => Ok(GestureFloating::Float),
            "tile" => Ok(GestureFloating::Tile),
            _ => Err(()),
        }
    }
}

impl Display for GestureFloating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestureFloating::Toggle => write!(f, ""),
            GestureFloating::Float => write!(f, "float"),
            GestureFloating::Tile => write!(f, "tile"),
        }
    }
}

impl EnumConfigForGtk for GestureFloating {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.gesture_floating.toggle"),
            &t!("hyprland.gesture_floating.float"),
            &t!("hyprland.gesture_floating.tile"),
        ])
    }
}

register_togtkbox!(GestureFloating);
