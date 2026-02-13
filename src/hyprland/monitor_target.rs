use super::Direction;
use crate::{
    gtk_converters::{
        EnumConfigForGtk, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(MonitorTargetDescriminants))]
#[derive(Default)]
pub enum MonitorTarget {
    Direction(Direction),
    Id(u32),
    Name(String),
    #[default]
    Current,
    Relative(i32),
}

impl HasDiscriminant for MonitorTarget {
    type Discriminant = MonitorTargetDescriminants;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(Direction::default()),
            Self::Discriminant::Id => Self::Id(0),
            Self::Discriminant::Name => Self::Name("".to_string()),
            Self::Discriminant::Current => Self::Current,
            Self::Discriminant::Relative => Self::Relative(0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Direction => {
                Self::Direction(Direction::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Id => Self::Id(str.parse().unwrap_or_default()),
            Self::Discriminant::Name => Self::Name(str.to_string()),
            Self::Discriminant::Current => Self::Current,
            Self::Discriminant::Relative => Self::Relative(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            MonitorTarget::Direction(direction) => Some(direction.to_string()),
            MonitorTarget::Id(id) => Some(id.to_string()),
            MonitorTarget::Name(name) => Some(name.to_string()),
            MonitorTarget::Current => None,
            MonitorTarget::Relative(rel_id) => Some(format!("{:+}", rel_id)),
        }
    }
}

impl FromStr for MonitorTarget {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if (s.starts_with("+") || s.starts_with("-"))
            && let Ok(rel_id) = s.parse::<i32>()
        {
            return Ok(MonitorTarget::Relative(rel_id));
        }

        if let Ok(dir) = s.parse::<Direction>() {
            Ok(MonitorTarget::Direction(dir))
        } else if let Ok(id) = s.parse::<u32>() {
            Ok(MonitorTarget::Id(id))
        } else if s == "current" {
            Ok(MonitorTarget::Current)
        } else if let Ok(rel_id) = s.parse::<i32>() {
            Ok(MonitorTarget::Relative(rel_id))
        } else {
            Ok(MonitorTarget::Name(s.to_string()))
        }
    }
}

impl Display for MonitorTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorTarget::Direction(dir) => write!(f, "{}", dir),
            MonitorTarget::Id(id) => write!(f, "{}", id),
            MonitorTarget::Name(name) => write!(f, "{}", name),
            MonitorTarget::Current => write!(f, "current"),
            MonitorTarget::Relative(rel_id) => write!(f, "{:+}", rel_id),
        }
    }
}

impl EnumConfigForGtk for MonitorTarget {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.monitor_target.direction"),
            &t!("hyprland.monitor_target.id"),
            &t!("hyprland.monitor_target.name"),
            &t!("hyprland.monitor_target.current"),
            &t!("hyprland.monitor_target.relative"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            MonitorTarget::Direction(_direction) => Some(<(Direction,)>::to_gtk_box),
            MonitorTarget::Id(_id) => Some(<(u32,)>::to_gtk_box),
            MonitorTarget::Name(_name) => Some(<(String,)>::to_gtk_box),
            MonitorTarget::Current => None,
            MonitorTarget::Relative(_relative) => Some(<(i32,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(MonitorTarget);
