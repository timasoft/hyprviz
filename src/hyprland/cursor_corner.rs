use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum CursorCorner {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl CursorCorner {
    pub fn to_num(self) -> u8 {
        match self {
            CursorCorner::TopLeft => 0,
            CursorCorner::TopRight => 1,
            CursorCorner::BottomLeft => 2,
            CursorCorner::BottomRight => 3,
        }
    }

    pub fn from_num(num: u8) -> Self {
        match num {
            0 => CursorCorner::TopLeft,
            1 => CursorCorner::TopRight,
            2 => CursorCorner::BottomLeft,
            3 => CursorCorner::BottomRight,
            _ => CursorCorner::TopLeft,
        }
    }
}

impl FromStr for CursorCorner {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_num(s.parse().unwrap_or_default()))
    }
}

impl Display for CursorCorner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

impl EnumConfigForGtk for CursorCorner {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.cursor_corner.top_left"),
            &t!("hyprland.cursor_corner.top_right"),
            &t!("hyprland.cursor_corner.bottom_left"),
            &t!("hyprland.cursor_corner.bottom_right"),
        ])
    }
}

register_togtkbox!(CursorCorner);
