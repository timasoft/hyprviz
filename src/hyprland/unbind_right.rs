use super::{Modifier, modifier::parse_modifiers};
use crate::utils::join_with_separator;
use rust_i18n::t;
use std::{collections::HashSet, error::Error, fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub struct UnbindRight {
    pub mods: HashSet<Modifier>,
    pub key: String,
}

impl FromStr for UnbindRight {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if parts.is_empty() {
            return Err(t!("utils.unbind_parse_error", count = parts.len()).into());
        }

        let mods_str = &parts[0];
        let mods = if mods_str.is_empty() {
            HashSet::new()
        } else {
            parse_modifiers(mods_str)
        };

        let key = parts.get(1).unwrap_or(&String::new()).clone();

        Ok(UnbindRight { mods, key })
    }
}

impl Display for UnbindRight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = format!("{}, {}", join_with_separator(&self.mods, "_"), self.key,);
        write!(f, "{}", result)
    }
}
