use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum FullscreenState {
    #[default]
    None,
    Maximize,
    Fullscreen,
    MaximizeAndFullscreen,
}

impl FullscreenState {
    pub fn from_num(num: u8) -> Self {
        match num {
            0 => FullscreenState::None,
            1 => FullscreenState::Maximize,
            2 => FullscreenState::Fullscreen,
            3 => FullscreenState::MaximizeAndFullscreen,
            _ => FullscreenState::None,
        }
    }

    pub fn to_num(self) -> u8 {
        match self {
            FullscreenState::None => 0,
            FullscreenState::Maximize => 1,
            FullscreenState::Fullscreen => 2,
            FullscreenState::MaximizeAndFullscreen => 3,
        }
    }
}

impl FromStr for FullscreenState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        if let Ok(num) = s.parse::<u8>() {
            Ok(FullscreenState::from_num(num))
        } else {
            Err(())
        }
    }
}

impl Display for FullscreenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

impl EnumConfigForGtk for FullscreenState {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.none"),
            &t!("gtk_converters.maximize"),
            &t!("gtk_converters.fullscreen"),
            &t!("gtk_converters.maximize_and_fullscreen"),
        ])
    }
}

register_togtkbox!(FullscreenState);
