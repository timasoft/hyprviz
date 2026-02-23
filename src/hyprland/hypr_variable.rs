use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum HyprVariable {
    #[default]
    MonitorW,
    MonitorH,
    WindowX,
    WindowY,
    WindowW,
    WindowH,
    CursorX,
    CursorY,
}

impl FromStr for HyprVariable {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .to_lowercase()
            .as_str()
        {
            "monitor_w" => Ok(HyprVariable::MonitorW),
            "monitor_h" => Ok(HyprVariable::MonitorH),
            "window_x" => Ok(HyprVariable::WindowX),
            "window_y" => Ok(HyprVariable::WindowY),
            "window_w" => Ok(HyprVariable::WindowW),
            "window_h" => Ok(HyprVariable::WindowH),
            "cursor_x" => Ok(HyprVariable::CursorX),
            "cursor_y" => Ok(HyprVariable::CursorY),
            _ => Err(()),
        }
    }
}

impl Display for HyprVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprVariable::MonitorW => write!(f, "monitor_w"),
            HyprVariable::MonitorH => write!(f, "monitor_h"),
            HyprVariable::WindowX => write!(f, "window_x"),
            HyprVariable::WindowY => write!(f, "window_y"),
            HyprVariable::WindowW => write!(f, "window_w"),
            HyprVariable::WindowH => write!(f, "window_h"),
            HyprVariable::CursorX => write!(f, "cursor_x"),
            HyprVariable::CursorY => write!(f, "cursor_y"),
        }
    }
}

impl EnumConfigForGtk for HyprVariable {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.hypr_variable.monitor_w"),
            &t!("hyprland.hypr_variable.monitor_h"),
            &t!("hyprland.hypr_variable.window_x"),
            &t!("hyprland.hypr_variable.window_y"),
            &t!("hyprland.hypr_variable.window_w"),
            &t!("hyprland.hypr_variable.window_h"),
            &t!("hyprland.hypr_variable.cursor_x"),
            &t!("hyprland.hypr_variable.cursor_y"),
        ])
    }
}

register_togtkbox!(HyprVariable);
