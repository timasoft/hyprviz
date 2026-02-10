use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum Direction {
    #[default]
    Left,
    Right,
    Up,
    Down,
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "l" | "left" => Ok(Direction::Left),
            "r" | "right" => Ok(Direction::Right),
            "u" | "up" => Ok(Direction::Up),
            "d" | "down" => Ok(Direction::Down),
            _ => Err(()),
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Left => write!(f, "left"),
            Direction::Right => write!(f, "right"),
            Direction::Up => write!(f, "up"),
            Direction::Down => write!(f, "down"),
        }
    }
}

impl EnumConfigForGtk for Direction {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.left"),
            &t!("gtk_converters.right"),
            &t!("gtk_converters.up"),
            &t!("gtk_converters.down"),
        ])
    }
}

register_togtkbox!(Direction);
