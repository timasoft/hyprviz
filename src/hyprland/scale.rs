use rust_i18n::t;
use std::{fmt::Display, str::FromStr};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Scale {
    #[default]
    Auto,
    Manual(f64),
}

impl Scale {
    pub fn get_fancy_list() -> [String; 2] {
        [
            t!("hyprland.scale.auto").to_string(),
            t!("hyprland.scale.manual").to_string(),
        ]
    }

    pub fn from_id(id: usize, value: Option<f64>) -> Self {
        match id {
            0 => Scale::Auto,
            _ => Scale::Manual(value.unwrap_or(1.0)),
        }
    }
}

impl Display for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scale::Auto => write!(f, "auto"),
            Scale::Manual(scale) => write!(f, "{:.2}", scale),
        }
    }
}

impl FromStr for Scale {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "auto" => Ok(Scale::Auto),
            scale => Ok(Scale::Manual(scale.parse::<f64>().map_err(|_| ())?)),
        }
    }
}
