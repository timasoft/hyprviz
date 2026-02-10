use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum GestureDirection {
    #[default]
    Swipe,
    Horizontal,
    Vertical,
    Left,
    Right,
    Up,
    Down,
    Pinch,
    PinchIn,
    PinchOut,
}

impl FromStr for GestureDirection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "swipe" => Ok(GestureDirection::Swipe),
            "horizontal" => Ok(GestureDirection::Horizontal),
            "vertical" => Ok(GestureDirection::Vertical),
            "left" => Ok(GestureDirection::Left),
            "right" => Ok(GestureDirection::Right),
            "up" => Ok(GestureDirection::Up),
            "down" => Ok(GestureDirection::Down),
            "pinch" => Ok(GestureDirection::Pinch),
            "pinchin" => Ok(GestureDirection::PinchIn),
            "pinchout" => Ok(GestureDirection::PinchOut),
            _ => Err(()),
        }
    }
}

impl Display for GestureDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestureDirection::Swipe => write!(f, "swipe"),
            GestureDirection::Horizontal => write!(f, "horizontal"),
            GestureDirection::Vertical => write!(f, "vertical"),
            GestureDirection::Left => write!(f, "left"),
            GestureDirection::Right => write!(f, "right"),
            GestureDirection::Up => write!(f, "up"),
            GestureDirection::Down => write!(f, "down"),
            GestureDirection::Pinch => write!(f, "pinch"),
            GestureDirection::PinchIn => write!(f, "pinchin"),
            GestureDirection::PinchOut => write!(f, "pinchout"),
        }
    }
}

impl EnumConfigForGtk for GestureDirection {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.swipe"),
            &t!("gtk_converters.horizontal"),
            &t!("gtk_converters.vertical"),
            &t!("gtk_converters.left"),
            &t!("gtk_converters.right"),
            &t!("gtk_converters.up"),
            &t!("gtk_converters.down"),
            &t!("gtk_converters.pinch"),
            &t!("gtk_converters.pinchin"),
            &t!("gtk_converters.pinchout"),
        ])
    }
}

register_togtkbox!(GestureDirection);
