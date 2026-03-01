use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Default, EnumIter)]
pub enum HyprLayout {
    #[default]
    Dwindle,
    Master,
    Scrolling,
    Monocle,
}

impl FromStr for HyprLayout {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "dwindle" => Ok(HyprLayout::Dwindle),
            "master" => Ok(HyprLayout::Master),
            "scrolling" => Ok(HyprLayout::Scrolling),
            "monocle" => Ok(HyprLayout::Monocle),
            _ => Err(format!("Invalid orientation: {}", s)),
        }
    }
}

impl Display for HyprLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprLayout::Dwindle => write!(f, "dwindle"),
            HyprLayout::Master => write!(f, "master"),
            HyprLayout::Scrolling => write!(f, "scrolling"),
            HyprLayout::Monocle => write!(f, "monocle"),
        }
    }
}
