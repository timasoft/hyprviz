use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum FullscreenMode {
    #[default]
    Fullscreen,
    Maximize,
}

impl FullscreenMode {
    pub fn from_num(num: u8) -> Self {
        match num {
            0 => FullscreenMode::Fullscreen,
            1 => FullscreenMode::Maximize,
            _ => FullscreenMode::Fullscreen,
        }
    }

    pub fn to_num(self) -> u8 {
        match self {
            FullscreenMode::Fullscreen => 0,
            FullscreenMode::Maximize => 1,
        }
    }
}

impl FromStr for FullscreenMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Ok(num) = s.parse::<u8>() {
            Ok(FullscreenMode::from_num(num))
        } else {
            Err(())
        }
    }
}

impl Display for FullscreenMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

impl EnumConfigForGtk for FullscreenMode {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.fullscreen_mode.fullscreen"),
            &t!("hyprland.fullscreen_mode.maximize"),
        ])
    }
}

register_togtkbox!(FullscreenMode);
