use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum WorkspaceSelectorFullscreen {
    NoFullscreen,
    #[default]
    Fullscreen,
    Maximized,
    FullscreenWithoutFullscreenStateSentToTheWindow,
}

impl WorkspaceSelectorFullscreen {
    pub fn from_num(num: i8) -> Self {
        match num {
            -1 => WorkspaceSelectorFullscreen::NoFullscreen,
            0 => WorkspaceSelectorFullscreen::Fullscreen,
            1 => WorkspaceSelectorFullscreen::Maximized,
            2 => WorkspaceSelectorFullscreen::FullscreenWithoutFullscreenStateSentToTheWindow,
            _ => WorkspaceSelectorFullscreen::default(),
        }
    }

    pub fn to_num(self) -> i8 {
        match self {
            WorkspaceSelectorFullscreen::NoFullscreen => -1,
            WorkspaceSelectorFullscreen::Fullscreen => 0,
            WorkspaceSelectorFullscreen::Maximized => 1,
            WorkspaceSelectorFullscreen::FullscreenWithoutFullscreenStateSentToTheWindow => 2,
        }
    }
}

impl FromStr for WorkspaceSelectorFullscreen {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().parse::<i8>() {
            Ok(num) => Ok(WorkspaceSelectorFullscreen::from_num(num)),
            Err(_) => Err(()),
        }
    }
}

impl Display for WorkspaceSelectorFullscreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

impl EnumConfigForGtk for WorkspaceSelectorFullscreen {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.workspace_selector_fullscreen.no_fullscreen"),
            &t!("hyprland.workspace_selector_fullscreen.fullscreen"),
            &t!("hyprland.workspace_selector_fullscreen.maximized"),
            &t!(
                "hyprland.workspace_selector_fullscreen.fullscreen_without_window_fullscreen_state_sent_to_the_window"
            ),
        ])
    }
}

register_togtkbox!(WorkspaceSelectorFullscreen);
