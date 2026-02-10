use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Default, EnumIter)]
pub enum Orientation {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Orientation::Left => write!(f, "left"),
            Orientation::Right => write!(f, "right"),
            Orientation::Top => write!(f, "top"),
            Orientation::Bottom => write!(f, "bottom"),
            Orientation::Center => write!(f, "center"),
        }
    }
}

impl FromStr for Orientation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "left" => Ok(Orientation::Left),
            "right" => Ok(Orientation::Right),
            "top" => Ok(Orientation::Top),
            "bottom" => Ok(Orientation::Bottom),
            "center" => Ok(Orientation::Center),
            _ => Err(format!("Invalid orientation: {}", s)),
        }
    }
}
