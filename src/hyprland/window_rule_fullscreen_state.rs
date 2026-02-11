use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum WindowRuleFullscreenState {
    #[default]
    Any,
    None,
    Maximize,
    Fullscreen,
    MaximizeAndFullscreen,
}

impl FromStr for WindowRuleFullscreenState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let first_byte = match s.trim_start().as_bytes().first() {
            Some(byte) => byte,
            None => return Err(()),
        };
        match *first_byte {
            b'*' => Ok(WindowRuleFullscreenState::Any),
            b'0' => Ok(WindowRuleFullscreenState::None),
            b'1' => Ok(WindowRuleFullscreenState::Maximize),
            b'2' => Ok(WindowRuleFullscreenState::Fullscreen),
            b'3' => Ok(WindowRuleFullscreenState::MaximizeAndFullscreen),
            _ => Err(()),
        }
    }
}

impl Display for WindowRuleFullscreenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRuleFullscreenState::Any => write!(f, "*"),
            WindowRuleFullscreenState::None => write!(f, "0"),
            WindowRuleFullscreenState::Maximize => write!(f, "1"),
            WindowRuleFullscreenState::Fullscreen => write!(f, "2"),
            WindowRuleFullscreenState::MaximizeAndFullscreen => write!(f, "3"),
        }
    }
}

impl EnumConfigForGtk for WindowRuleFullscreenState {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.window_rule_fullscreen_state.any"),
            &t!("hyprland.window_rule_fullscreen_state.none"),
            &t!("hyprland.window_rule_fullscreen_state.maximize"),
            &t!("hyprland.window_rule_fullscreen_state.fullscreen"),
            &t!("hyprland.window_rule_fullscreen_state.maximize_and_fullscreen"),
        ])
    }
}

register_togtkbox!(WindowRuleFullscreenState);
