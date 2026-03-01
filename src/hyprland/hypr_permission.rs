use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum HyprPermission {
    #[default]
    ScreenCopy,
    Plugin,
    Keyboard,
}

impl FromStr for HyprPermission {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "screencopy" => Ok(HyprPermission::ScreenCopy),
            "plugin" => Ok(HyprPermission::Plugin),
            "keyboard" => Ok(HyprPermission::Keyboard),
            _ => Err(()),
        }
    }
}

impl Display for HyprPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprPermission::ScreenCopy => write!(f, "screencopy"),
            HyprPermission::Plugin => write!(f, "plugin"),
            HyprPermission::Keyboard => write!(f, "keyboard"),
        }
    }
}

impl EnumConfigForGtk for HyprPermission {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.hypr_permission.screencopy"),
            &t!("hyprland.hypr_permission.plugin"),
            &t!("hyprland.hypr_permission.keyboard"),
        ])
    }
}

register_togtkbox!(HyprPermission);
