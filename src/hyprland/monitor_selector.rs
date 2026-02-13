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

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(MonitorSelectorDiscriminant))]
pub enum MonitorSelector {
    #[default]
    All,
    Name(String),
    Description(String),
}

impl HasDiscriminant for MonitorSelector {
    type Discriminant = MonitorSelectorDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::All => Self::All,
            Self::Discriminant::Name => Self::Name("".to_string()),
            Self::Discriminant::Description => Self::Description("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::All => Self::All,
            Self::Discriminant::Name => Self::Name(str.to_string()),
            Self::Discriminant::Description => Self::Description(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::All => None,
            Self::Name(name) => Some(name.to_string()),
            Self::Description(desc) => Some(desc.to_string()),
        }
    }
}

impl FromStr for MonitorSelector {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(MonitorSelector::All),
            s => {
                if let Some(stripped) = s.strip_prefix("desc:") {
                    Ok(MonitorSelector::Description(stripped.to_string()))
                } else {
                    Ok(MonitorSelector::Name(s.to_string()))
                }
            }
        }
    }
}

impl Display for MonitorSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorSelector::All => write!(f, ""),
            MonitorSelector::Name(name) => write!(f, "{}", name),
            MonitorSelector::Description(desc) => write!(f, "desc:{}", desc),
        }
    }
}

impl EnumConfigForGtk for MonitorSelector {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.monitor_selector.all"),
            &t!("hyprland.monitor_selector.name"),
            &t!("hyprland.monitor_selector.description"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::All => None,
            Self::Name(_name) => Some(<(String,)>::to_gtk_box),
            Self::Description(_description) => Some(<(String,)>::to_gtk_box),
        }
    }
}

register_togtkbox!(MonitorSelector);
