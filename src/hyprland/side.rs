use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Default)]
pub enum Side {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

impl Side {
    pub fn get_list() -> [&'static str; 4] {
        ["left", "right", "top", "bottom"]
    }

    pub fn get_fancy_list() -> [String; 4] {
        [
            t!("hyprland.side.left").to_string(),
            t!("hyprland.side.right").to_string(),
            t!("hyprland.side.top").to_string(),
            t!("hyprland.side.bottom").to_string(),
        ]
    }

    pub fn get_id(&self) -> usize {
        match self {
            Side::Left => 0,
            Side::Right => 1,
            Side::Top => 2,
            Side::Bottom => 3,
        }
    }
}

impl FromStr for Side {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "left" => Ok(Side::Left),
            "right" => Ok(Side::Right),
            "top" => Ok(Side::Top),
            "bottom" => Ok(Side::Bottom),
            _ => Err(()),
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Left => write!(f, "left"),
            Side::Right => write!(f, "right"),
            Side::Top => write!(f, "top"),
            Side::Bottom => write!(f, "bottom"),
        }
    }
}

impl EnumConfigForGtk for Side {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.side.left"),
            &t!("hyprland.side.right"),
            &t!("hyprland.side.top"),
            &t!("hyprland.side.bottom"),
        ])
    }
}

register_togtkbox!(Side);
