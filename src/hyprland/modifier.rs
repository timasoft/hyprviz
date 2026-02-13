use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use std::{collections::HashSet, fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum Modifier {
    Shift,
    Caps,
    Ctrl,
    Alt,
    Mod2,
    Mod3,
    #[default]
    Super,
    Mod5,
}

impl FromStr for Modifier {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "SHIFT" => Ok(Modifier::Shift),
            "CAPS" => Ok(Modifier::Caps),
            "CTRL" | "CONTROL" => Ok(Modifier::Ctrl),
            "ALT" => Ok(Modifier::Alt),
            "MOD2" => Ok(Modifier::Mod2),
            "MOD3" => Ok(Modifier::Mod3),
            "SUPER" | "WIN" | "LOGO" | "MOD4" => Ok(Modifier::Super),
            "MOD5" => Ok(Modifier::Mod5),
            _ => Err(()),
        }
    }
}

impl Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Modifier::Shift => write!(f, "SHIFT"),
            Modifier::Caps => write!(f, "CAPS"),
            Modifier::Ctrl => write!(f, "CTRL"),
            Modifier::Alt => write!(f, "ALT"),
            Modifier::Mod2 => write!(f, "MOD2"),
            Modifier::Mod3 => write!(f, "MOD3"),
            Modifier::Super => write!(f, "SUPER"),
            Modifier::Mod5 => write!(f, "MOD5"),
        }
    }
}

pub fn parse_modifiers(s: &str) -> HashSet<Modifier> {
    let mut mods = HashSet::new();

    let s_upper = s.trim_start().to_uppercase();
    if s_upper.contains("SUPER")
        || s_upper.contains("WIN")
        || s_upper.contains("LOGO")
        || s_upper.contains("MOD4")
    {
        mods.insert(Modifier::Super);
    }
    if s_upper.contains("SHIFT") {
        mods.insert(Modifier::Shift);
    }
    if s_upper.contains("CTRL") || s_upper.contains("CONTROL") {
        mods.insert(Modifier::Ctrl);
    }
    if s_upper.contains("ALT") {
        mods.insert(Modifier::Alt);
    }
    if s_upper.contains("CAPS") {
        mods.insert(Modifier::Caps);
    }
    if s_upper.contains("MOD2") {
        mods.insert(Modifier::Mod2);
    }
    if s_upper.contains("MOD3") {
        mods.insert(Modifier::Mod3);
    }
    if s_upper.contains("MOD5") {
        mods.insert(Modifier::Mod5);
    }

    mods
}

impl EnumConfigForGtk for Modifier {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            "SHIFT", "CAPS", "CTRL", "ALT", "MOD2", "MOD3", "SUPER", "MOD5",
        ])
    }
}

register_togtkbox!(Modifier);
