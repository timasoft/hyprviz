use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum GroupLockAction {
    Lock,
    Unlock,
    #[default]
    Toggle,
}

impl FromStr for GroupLockAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "lock" => Ok(GroupLockAction::Lock),
            "unlock" => Ok(GroupLockAction::Unlock),
            "toggle" => Ok(GroupLockAction::Toggle),
            _ => Err(()),
        }
    }
}

impl Display for GroupLockAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupLockAction::Lock => write!(f, "lock"),
            GroupLockAction::Unlock => write!(f, "unlock"),
            GroupLockAction::Toggle => write!(f, "toggle"),
        }
    }
}

impl EnumConfigForGtk for GroupLockAction {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.lock"),
            &t!("gtk_converters.unlock"),
            &t!("gtk_converters.toggle"),
        ])
    }
}

register_togtkbox!(GroupLockAction);
