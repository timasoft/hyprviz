use crate::{
    gtk_converters::{FieldLabel, ToGtkBox, create_spin_button_builder},
    register_togtkbox,
    utils::MAX_SAFE_STEP_0_01_F64,
};
use gtk::{Box as GtkBox, Entry};
use std::{fmt::Display, num::ParseFloatError, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PosFloat0_01(f64);

impl FromStr for PosFloat0_01 {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PosFloat0_01(s.parse::<f64>()?))
    }
}

impl Display for PosFloat0_01 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl ToGtkBox for PosFloat0_01 {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(entry, &FieldLabel::Unnamed)
    }
}

register_togtkbox!(PosFloat0_01);
