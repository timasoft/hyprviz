use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum WindowGroupOption {
    #[default]
    Set,
    SetAlways,
    New,
    Lock,
    LockAlways,
    Barred,
    Deny,
    Invade,
    Override,
    Unset,
}

impl FromStr for WindowGroupOption {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        match s {
            "set" => Ok(WindowGroupOption::Set),
            "set always" => Ok(WindowGroupOption::SetAlways),
            "new" => Ok(WindowGroupOption::New),
            "lock" => Ok(WindowGroupOption::Lock),
            "lock always" => Ok(WindowGroupOption::LockAlways),
            "barred" => Ok(WindowGroupOption::Barred),
            "deny" => Ok(WindowGroupOption::Deny),
            "invade" => Ok(WindowGroupOption::Invade),
            "override" => Ok(WindowGroupOption::Override),
            "unset" => Ok(WindowGroupOption::Unset),
            _ => Err(()),
        }
    }
}

impl Display for WindowGroupOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowGroupOption::Set => write!(f, "set"),
            WindowGroupOption::SetAlways => write!(f, "set always"),
            WindowGroupOption::New => write!(f, "new"),
            WindowGroupOption::Lock => write!(f, "lock"),
            WindowGroupOption::LockAlways => write!(f, "lock always"),
            WindowGroupOption::Barred => write!(f, "barred"),
            WindowGroupOption::Deny => write!(f, "deny"),
            WindowGroupOption::Invade => write!(f, "invade"),
            WindowGroupOption::Override => write!(f, "override"),
            WindowGroupOption::Unset => write!(f, "unset"),
        }
    }
}

impl EnumConfigForGtk for WindowGroupOption {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.window_group_option.set"),
            &t!("hyprland.window_group_option.set_always"),
            &t!("hyprland.window_group_option.new"),
            &t!("hyprland.window_group_option.lock"),
            &t!("hyprland.window_group_option.lock_always"),
            &t!("hyprland.window_group_option.barred"),
            &t!("hyprland.window_group_option.deny"),
            &t!("hyprland.window_group_option.invade"),
            &t!("hyprland.window_group_option.override"),
            &t!("hyprland.window_group_option.unset"),
        ])
    }
}

register_togtkbox!(WindowGroupOption);
