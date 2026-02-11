use super::{Direction, MonitorTarget};
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(MoveDirectionDiscriminant))]
pub enum MoveDirection {
    Direction(Direction),
    DirectionSilent(Direction),
    Monitor(MonitorTarget),
    MonitorSilent(MonitorTarget),
}

impl HasDiscriminant for MoveDirection {
    type Discriminant = MoveDirectionDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(Direction::default()),
            Self::Discriminant::DirectionSilent => Self::DirectionSilent(Direction::default()),
            Self::Discriminant::Monitor => Self::Monitor(MonitorTarget::default()),
            Self::Discriminant::MonitorSilent => Self::MonitorSilent(MonitorTarget::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(str.parse().unwrap_or_default()),
            Self::Discriminant::DirectionSilent => {
                Self::DirectionSilent(str.parse().unwrap_or_default())
            }
            Self::Discriminant::Monitor => Self::Monitor(str.parse().unwrap_or_default()),
            Self::Discriminant::MonitorSilent => {
                Self::MonitorSilent(str.parse().unwrap_or_default())
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            MoveDirection::Direction(direction) => Some(direction.to_string()),
            MoveDirection::DirectionSilent(direction) => Some(direction.to_string()),
            MoveDirection::Monitor(monitor) => Some(monitor.to_string()),
            MoveDirection::MonitorSilent(monitor) => Some(monitor.to_string()),
        }
    }
}

impl Default for MoveDirection {
    fn default() -> Self {
        MoveDirection::Direction(Direction::default())
    }
}

impl FromStr for MoveDirection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        let (part1, part2) = s.rsplit_once(' ').unwrap_or((s, ""));

        if let Some(stripped) = part1.strip_prefix("mon:") {
            let monitor = MonitorTarget::from_str(stripped.trim()).unwrap_or_default();
            match part2 {
                "silent" => Ok(MoveDirection::MonitorSilent(monitor)),
                _ => Ok(MoveDirection::Monitor(monitor)),
            }
        } else {
            let direction = Direction::from_str(part1).unwrap_or_default();
            match part2 {
                "silent" => Ok(MoveDirection::DirectionSilent(direction)),
                _ => Ok(MoveDirection::Direction(direction)),
            }
        }
    }
}

impl Display for MoveDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveDirection::Direction(direction) => {
                write!(f, "{}", direction)
            }
            MoveDirection::DirectionSilent(direction) => {
                write!(f, "{} silent", direction)
            }
            MoveDirection::Monitor(monitor) => {
                write!(f, "mon:{}", monitor)
            }
            MoveDirection::MonitorSilent(monitor) => {
                write!(f, "mon:{} silent", monitor)
            }
        }
    }
}

impl EnumConfigForGtk for MoveDirection {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.move_direction.direction"),
            &t!("hyprland.move_direction.direction_silent"),
            &t!("hyprland.move_direction.monitor"),
            &t!("hyprland.move_direction.monitor_silent"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            MoveDirection::Direction(_direction) => Some(<(Direction,)>::to_gtk_box),
            MoveDirection::DirectionSilent(_direction) => Some(<(Direction,)>::to_gtk_box),
            MoveDirection::Monitor(_monitor_target) => Some(<(MonitorTarget,)>::to_gtk_box),
            MoveDirection::MonitorSilent(_monitor_target) => Some(<(MonitorTarget,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(MoveDirection);
register_togtkbox_with_separator_names!((Direction,), (MonitorTarget,));
