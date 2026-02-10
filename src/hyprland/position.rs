use rust_i18n::t;
use std::{fmt::Display, str::FromStr};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    #[default]
    Auto,
    AutoRight,
    AutoLeft,
    AutoUp,
    AutoDown,
    AutoCenterRight,
    AutoCenterLeft,
    AutoCenterUp,
    AutoCenterDown,
    Coordinates(i64, i64),
}

impl Position {
    pub fn get_fancy_list() -> [String; 10] {
        [
            t!("utils.auto").to_string(),
            t!("utils.auto_right").to_string(),
            t!("utils.auto_left").to_string(),
            t!("utils.auto_up").to_string(),
            t!("utils.auto_down").to_string(),
            t!("utils.auto_center_right").to_string(),
            t!("utils.auto_center_left").to_string(),
            t!("utils.auto_center_up").to_string(),
            t!("utils.auto_center_down").to_string(),
            t!("utils.coordinates").to_string(),
        ]
    }

    pub fn get_list() -> [&'static str; 10] {
        [
            "auto",
            "auto-right",
            "auto-left",
            "auto-up",
            "auto-down",
            "auto-center-right",
            "auto-center-left",
            "auto-center-up",
            "auto-center-down",
            "coordinates",
        ]
    }

    pub fn from_id(id: usize, x: Option<i64>, y: Option<i64>) -> Self {
        match id {
            0 => Position::Auto,
            1 => Position::AutoRight,
            2 => Position::AutoLeft,
            3 => Position::AutoUp,
            4 => Position::AutoDown,
            5 => Position::AutoCenterRight,
            6 => Position::AutoCenterLeft,
            7 => Position::AutoCenterUp,
            8 => Position::AutoCenterDown,
            _ => Position::Coordinates(x.unwrap_or(0), y.unwrap_or(0)),
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::Auto => write!(f, "auto"),
            Position::AutoRight => write!(f, "auto-right"),
            Position::AutoLeft => write!(f, "auto-left"),
            Position::AutoUp => write!(f, "auto-up"),
            Position::AutoDown => write!(f, "auto-down"),
            Position::AutoCenterRight => write!(f, "auto-center-right"),
            Position::AutoCenterLeft => write!(f, "auto-center-left"),
            Position::AutoCenterUp => write!(f, "auto-center-up"),
            Position::AutoCenterDown => write!(f, "auto-center-down"),
            Position::Coordinates(x, y) => write!(f, "{}x{}", x, y),
        }
    }
}

impl FromStr for Position {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "auto" => Ok(Position::Auto),
            "auto-right" => Ok(Position::AutoRight),
            "auto-left" => Ok(Position::AutoLeft),
            "auto-up" => Ok(Position::AutoUp),
            "auto-down" => Ok(Position::AutoDown),
            "auto-center-right" => Ok(Position::AutoCenterRight),
            "auto-center-left" => Ok(Position::AutoCenterLeft),
            "auto-center-up" => Ok(Position::AutoCenterUp),
            position => {
                if let Some((x_str, y_str)) = position.split_once('x') {
                    let x = x_str.parse::<i64>().map_err(|_| ())?;
                    let y = y_str.parse::<i64>().map_err(|_| ())?;
                    Ok(Position::Coordinates(x, y))
                } else {
                    Err(())
                }
            }
        }
    }
}
