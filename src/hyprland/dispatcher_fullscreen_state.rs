use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum DispatcherFullscreenState {
    Current,
    #[default]
    None,
    Maximize,
    Fullscreen,
    MaximizeAndFullscreen,
}

impl DispatcherFullscreenState {
    pub fn from_num(num: i8) -> Self {
        match num {
            -1 => DispatcherFullscreenState::Current,
            0 => DispatcherFullscreenState::None,
            1 => DispatcherFullscreenState::Maximize,
            2 => DispatcherFullscreenState::Fullscreen,
            3 => DispatcherFullscreenState::MaximizeAndFullscreen,
            _ => DispatcherFullscreenState::Current,
        }
    }

    pub fn to_num(self) -> i8 {
        match self {
            DispatcherFullscreenState::Current => -1,
            DispatcherFullscreenState::None => 0,
            DispatcherFullscreenState::Maximize => 1,
            DispatcherFullscreenState::Fullscreen => 2,
            DispatcherFullscreenState::MaximizeAndFullscreen => 3,
        }
    }
}

impl FromStr for DispatcherFullscreenState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_num(s.trim().parse().unwrap_or(-1)))
    }
}

impl Display for DispatcherFullscreenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

impl EnumConfigForGtk for DispatcherFullscreenState {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.dispatcher_fullscreen_state.current"),
            &t!("hyprland.dispatcher_fullscreen_state.none"),
            &t!("hyprland.dispatcher_fullscreen_state.maximize"),
            &t!("hyprland.dispatcher_fullscreen_state.fullscreen"),
            &t!("hyprland.dispatcher_fullscreen_state.maximize_and_fullscreen"),
        ])
    }
}

register_togtkbox!(DispatcherFullscreenState);
