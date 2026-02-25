use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
#[allow(clippy::enum_variant_names)]
pub enum AboveLock {
    #[default]
    BelowLock,
    AboveLock,
    AboveLockInteractable,
}

impl TryFrom<u8> for AboveLock {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BelowLock),
            1 => Ok(Self::AboveLock),
            2 => Ok(Self::AboveLockInteractable),
            _ => Err(()),
        }
    }
}

impl From<AboveLock> for u8 {
    fn from(value: AboveLock) -> Self {
        match value {
            AboveLock::BelowLock => 0,
            AboveLock::AboveLock => 1,
            AboveLock::AboveLockInteractable => 2,
        }
    }
}

impl FromStr for AboveLock {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let uint: u8 = s.parse().map_err(|_| ())?;
        Self::try_from(uint)
    }
}

impl Display for AboveLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u8::from(*self))
    }
}

impl EnumConfigForGtk for AboveLock {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.above_lock.below_lock"),
            &t!("hyprland.above_lock.above_lock"),
            &t!("hyprland.above_lock.above_lock_interactable"),
        ])
    }
}

register_togtkbox!(AboveLock);
