use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum WindowEvent {
    #[default]
    Fullscreen,
    Maximize,
    Activate,
    ActivateFocus,
    FullscreenOutput,
}

impl FromStr for WindowEvent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        match s {
            "fullscreen" => Ok(WindowEvent::Fullscreen),
            "maximize" => Ok(WindowEvent::Maximize),
            "activate" => Ok(WindowEvent::Activate),
            "activatefocus" => Ok(WindowEvent::ActivateFocus),
            "fullscreenoutput" => Ok(WindowEvent::FullscreenOutput),
            _ => Err(()),
        }
    }
}

impl Display for WindowEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowEvent::Fullscreen => write!(f, "fullscreen"),
            WindowEvent::Maximize => write!(f, "maximize"),
            WindowEvent::Activate => write!(f, "activate"),
            WindowEvent::ActivateFocus => write!(f, "activatefocus"),
            WindowEvent::FullscreenOutput => write!(f, "fullscreenoutput"),
        }
    }
}

impl EnumConfigForGtk for WindowEvent {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.fullscreen"),
            &t!("gtk_converters.maximize"),
            &t!("gtk_converters.activate"),
            &t!("gtk_converters.activatefocus"),
            &t!("gtk_converters.fullscreenoutput"),
        ])
    }
}

register_togtkbox!(WindowEvent);
