use crate::{
    gtk_converters::{
        EnumConfigForGtk, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WindowTargetDiscriminant))]
pub enum WindowTarget {
    Class(String),
    InitialClass(String),
    Title(String),
    InitialTitle(String),
    Tag(String),
    Pid(String),
    Address(String),
    ActiveWindow,
    Floating,
    Tiled,
}

impl HasDiscriminant for WindowTarget {
    type Discriminant = WindowTargetDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Class => Self::Class("".to_string()),
            Self::Discriminant::InitialClass => Self::InitialClass("".to_string()),
            Self::Discriminant::Title => Self::Title("".to_string()),
            Self::Discriminant::InitialTitle => Self::InitialTitle("".to_string()),
            Self::Discriminant::Tag => Self::Tag("".to_string()),
            Self::Discriminant::Pid => Self::Pid("".to_string()),
            Self::Discriminant::Address => Self::Address("".to_string()),
            Self::Discriminant::ActiveWindow => Self::ActiveWindow,
            Self::Discriminant::Floating => Self::Floating,
            Self::Discriminant::Tiled => Self::Tiled,
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Class => Self::Class(str.to_string()),
            Self::Discriminant::InitialClass => Self::InitialClass(str.to_string()),
            Self::Discriminant::Title => Self::Title(str.to_string()),
            Self::Discriminant::InitialTitle => Self::InitialTitle(str.to_string()),
            Self::Discriminant::Tag => Self::Tag(str.to_string()),
            Self::Discriminant::Pid => Self::Pid(str.to_string()),
            Self::Discriminant::Address => Self::Address(str.to_string()),
            Self::Discriminant::ActiveWindow => Self::ActiveWindow,
            Self::Discriminant::Floating => Self::Floating,
            Self::Discriminant::Tiled => Self::Tiled,
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WindowTarget::Class(class) => Some(class.clone()),
            WindowTarget::InitialClass(initial_class) => Some(initial_class.clone()),
            WindowTarget::Title(title) => Some(title.clone()),
            WindowTarget::InitialTitle(initial_title) => Some(initial_title.clone()),
            WindowTarget::Tag(tag) => Some(tag.clone()),
            WindowTarget::Pid(pid) => Some(pid.clone()),
            WindowTarget::Address(addr) => Some(addr.clone()),
            WindowTarget::ActiveWindow => None,
            WindowTarget::Floating => None,
            WindowTarget::Tiled => None,
        }
    }
}

impl Default for WindowTarget {
    fn default() -> Self {
        WindowTarget::Class("".to_string())
    }
}

impl FromStr for WindowTarget {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if s.starts_with("class:") {
            Ok(WindowTarget::Class(
                s.trim_start_matches("class:").to_string(),
            ))
        } else if s.starts_with("initialclass:") {
            Ok(WindowTarget::InitialClass(
                s.trim_start_matches("initialclass:").to_string(),
            ))
        } else if s.starts_with("title:") {
            Ok(WindowTarget::Title(
                s.trim_start_matches("title:").to_string(),
            ))
        } else if s.starts_with("initialtitle:") {
            Ok(WindowTarget::InitialTitle(
                s.trim_start_matches("initialtitle:").to_string(),
            ))
        } else if s.starts_with("tag:") {
            Ok(WindowTarget::Tag(s.trim_start_matches("tag:").to_string()))
        } else if s.starts_with("pid:") {
            Ok(WindowTarget::Pid(s.trim_start_matches("pid:").to_string()))
        } else if s == "address" {
            Ok(WindowTarget::Address(s.to_string()))
        } else if s == "activewindow" {
            Ok(WindowTarget::ActiveWindow)
        } else if s == "floating" {
            Ok(WindowTarget::Floating)
        } else if s == "tiled" {
            Ok(WindowTarget::Tiled)
        } else {
            Ok(WindowTarget::Class(s.to_string()))
        }
    }
}

impl Display for WindowTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowTarget::Class(class) => write!(f, "{}", class),
            WindowTarget::InitialClass(initial_class) => {
                write!(f, "initialclass:{}", initial_class)
            }
            WindowTarget::Title(title) => write!(f, "title:{}", title),
            WindowTarget::InitialTitle(initial_title) => {
                write!(f, "initialtitle:{}", initial_title)
            }
            WindowTarget::Tag(tag) => write!(f, "tag:{}", tag),
            WindowTarget::Pid(pid) => write!(f, "pid:{}", pid),
            WindowTarget::Address(addr) => write!(f, "address:{}", addr),
            WindowTarget::ActiveWindow => write!(f, "activewindow"),
            WindowTarget::Floating => write!(f, "floating"),
            WindowTarget::Tiled => write!(f, "tiled"),
        }
    }
}

impl EnumConfigForGtk for WindowTarget {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.window_target.class"),
            &t!("hyprland.window_target.initial_class"),
            &t!("hyprland.window_target.title"),
            &t!("hyprland.window_target.initial_title"),
            &t!("hyprland.window_target.tag"),
            &t!("hyprland.window_target.pid"),
            &t!("hyprland.window_target.address"),
            &t!("hyprland.window_target.active_window"),
            &t!("hyprland.window_target.floating"),
            &t!("hyprland.window_target.tiled"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            WindowTarget::Class(_class) => Some(<(String,)>::to_gtk_box),
            WindowTarget::InitialClass(_class) => Some(<(String,)>::to_gtk_box),
            WindowTarget::Title(_title) => Some(<(String,)>::to_gtk_box),
            WindowTarget::InitialTitle(_title) => Some(<(String,)>::to_gtk_box),
            WindowTarget::Tag(_tag) => Some(<(String,)>::to_gtk_box),
            WindowTarget::Pid(_pid) => Some(<(String,)>::to_gtk_box),
            WindowTarget::Address(_address) => Some(<(String,)>::to_gtk_box),
            WindowTarget::ActiveWindow => None,
            WindowTarget::Floating => None,
            WindowTarget::Tiled => None,
        }
    }
}

register_togtkbox!(WindowTarget);
register_togtkbox_with_separator_names!((String,));
