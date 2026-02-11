use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, ToGtkBoxWithSeparatorAndNamesBuilder,
        create_spin_button_builder,
    },
    register_togtkbox,
    utils::{HasDiscriminant, MAX_SAFE_STEP_0_01_F64, MIN_SAFE_STEP_0_01_F64},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(FloatValueDiscriminant))]
pub enum FloatValue {
    Relative(f64),
    Exact(f64),
}

impl HasDiscriminant for FloatValue {
    type Discriminant = FloatValueDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Relative => Self::Relative(0.0),
            Self::Discriminant::Exact => Self::Exact(0.0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Relative => match Self::from_str(str) {
                Ok(FloatValue::Relative(f)) => FloatValue::Relative(f),
                Ok(FloatValue::Exact(f)) => FloatValue::Relative(f),
                Err(_) => FloatValue::Relative(0.0),
            },
            Self::Discriminant::Exact => match Self::from_str(str) {
                Ok(FloatValue::Relative(f)) => FloatValue::Exact(f),
                Ok(FloatValue::Exact(f)) => FloatValue::Exact(f),
                Err(_) => FloatValue::Exact(0.0),
            },
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            FloatValue::Relative(f) => Some(format!("{:+}", f)),
            FloatValue::Exact(f) => Some(format!("{}", f)),
        }
    }
}

impl Default for FloatValue {
    fn default() -> Self {
        FloatValue::Relative(0.0)
    }
}

impl FromStr for FloatValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Some(s) = s.strip_prefix("exact ") {
            let float = s.parse::<f64>().unwrap_or(0.0);
            Ok(FloatValue::Exact(float.abs()))
        } else {
            let float = s.parse::<f64>().unwrap_or(0.0);
            Ok(FloatValue::Relative(float))
        }
    }
}

impl Display for FloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloatValue::Relative(float) => write!(f, "{:+}", float),
            FloatValue::Exact(float) => write!(f, "exact {}", float),
        }
    }
}

impl EnumConfigForGtk for FloatValue {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("hyprland.float_value.relative"), &t!("hyprland.float_value.exact")])
    }

    const SEPARATOR: Option<char> = Some(' ');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            FloatValue::Relative(_f) => Some(|entry, _, names, _| {
                create_spin_button_builder(MIN_SAFE_STEP_0_01_F64, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            FloatValue::Exact(_f) => Some(|entry, _, names, _| {
                create_spin_button_builder(MIN_SAFE_STEP_0_01_F64, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
        }
    }
}

register_togtkbox!(FloatValue);
