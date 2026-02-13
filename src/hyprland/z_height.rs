use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum ZHeight {
    #[default]
    Top,
    Bottom,
}

impl FromStr for ZHeight {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "top" => Ok(ZHeight::Top),
            "bottom" => Ok(ZHeight::Bottom),
            _ => Err(()),
        }
    }
}

impl Display for ZHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZHeight::Top => write!(f, "top"),
            ZHeight::Bottom => write!(f, "bottom"),
        }
    }
}

impl EnumConfigForGtk for ZHeight {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.z_height.top"),
            &t!("hyprland.z_height.bottom"),
        ])
    }
}

register_togtkbox!(ZHeight);
