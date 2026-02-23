use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum Operator {
    #[default]
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl FromStr for Operator {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "+" => Ok(Operator::Add),
            "-" => Ok(Operator::Subtract),
            "*" => Ok(Operator::Multiply),
            "/" => Ok(Operator::Divide),
            _ => Err(()),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Subtract => write!(f, "-"),
            Operator::Multiply => write!(f, "*"),
            Operator::Divide => write!(f, "/"),
        }
    }
}

impl EnumConfigForGtk for Operator {
    fn dropdown_items() -> StringList {
        StringList::new(&["+", "-", "*", "/"])
    }
}

register_togtkbox!(Operator);
