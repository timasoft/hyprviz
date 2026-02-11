use super::{Direction, WindowTarget};
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
#[strum_discriminants(name(SwapDirectionDiscriminant))]
pub enum SwapDirection {
    Direction(Direction),
    Window(WindowTarget),
}

impl HasDiscriminant for SwapDirection {
    type Discriminant = SwapDirectionDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(Direction::default()),
            Self::Discriminant::Window => Self::Window(WindowTarget::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(str.parse().unwrap_or_default()),
            Self::Discriminant::Window => Self::Window(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            SwapDirection::Direction(direction) => Some(direction.to_string()),
            SwapDirection::Window(window) => Some(window.to_string()),
        }
    }
}

impl Default for SwapDirection {
    fn default() -> Self {
        SwapDirection::Direction(Direction::default())
    }
}

impl FromStr for SwapDirection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        if let Ok(direction) = Direction::from_str(s) {
            Ok(SwapDirection::Direction(direction))
        } else if let Ok(window) = WindowTarget::from_str(s) {
            Ok(SwapDirection::Window(window))
        } else {
            Err(())
        }
    }
}

impl Display for SwapDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapDirection::Direction(direction) => {
                write!(f, "{}", direction)
            }
            SwapDirection::Window(window) => {
                write!(f, "{}", window)
            }
        }
    }
}

impl EnumConfigForGtk for SwapDirection {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.swap_direction.direction"),
            &t!("hyprland.swap_direction.window"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            SwapDirection::Direction(_direction) => Some(<(Direction,)>::to_gtk_box),
            SwapDirection::Window(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(SwapDirection);
register_togtkbox_with_separator_names!((Direction,), (WindowTarget,));
