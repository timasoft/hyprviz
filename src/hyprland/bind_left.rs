use super::BindFlags;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindLeft {
    Bind(BindFlags),
    Unbind,
}

impl FromStr for BindLeft {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(flags) = s.strip_prefix("bind") {
            if flags.is_empty() {
                Ok(BindLeft::Bind(BindFlags::default()))
            } else {
                let flags = flags.parse().unwrap_or_default();
                Ok(BindLeft::Bind(flags))
            }
        } else if s == "unbind" {
            Ok(BindLeft::Unbind)
        } else {
            Err(())
        }
    }
}

impl Display for BindLeft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindLeft::Bind(flags) if flags == &BindFlags::default() => write!(f, "bind"),
            BindLeft::Bind(flags) => write!(f, "bind{}", flags),
            BindLeft::Unbind => write!(f, "unbind"),
        }
    }
}
