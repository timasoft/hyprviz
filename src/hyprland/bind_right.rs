use super::{Dispatcher, Modifier, modifier::parse_modifiers};
use crate::utils::join_with_separator;
use rust_i18n::t;
use std::{collections::HashSet, error::Error, fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub struct BindRight {
    pub mods: HashSet<Modifier>,
    pub key: String,
    pub dispatcher: Dispatcher,
    pub description: Option<String>,
}

impl FromStr for BindRight {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if parts.is_empty() {
            return Err(t!("hyprland.bind_right.bind_parse_error", count = parts.len()).into());
        }

        let mods_str = &parts[0];
        let mods = if mods_str.is_empty() {
            HashSet::new()
        } else {
            parse_modifiers(mods_str)
        };

        let key = parts.get(1).unwrap_or(&String::new()).clone();

        let dispatcher_str = match parts.get(2) {
            Some(_) => parts[2..].join(", "),
            None => String::new(),
        };

        let dispatcher = dispatcher_str.parse().unwrap_or(Dispatcher::default());

        Ok(BindRight {
            mods,
            key,
            dispatcher,
            description: None,
        })
    }
}

impl Display for BindRight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = format!(
            "{}, {}{}, {}",
            join_with_separator(&self.mods, "_"),
            self.key,
            if let Some(desc) = &self.description {
                format!(", {}", desc)
            } else {
                "".to_string()
            },
            self.dispatcher,
        );
        write!(f, "{}", result)
    }
}

fn parse_bind_with_description(bind_right_str: &str) -> BindRight {
    let parts: Vec<String> = bind_right_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    if parts.is_empty() {
        return BindRight {
            mods: HashSet::new(),
            key: String::new(),
            dispatcher: Dispatcher::default(),
            description: Some(String::new()),
        };
    }

    let mods_str = &parts[0];
    let mods = if mods_str.is_empty() {
        HashSet::new()
    } else {
        parse_modifiers(mods_str)
    };

    let key = parts.get(1).unwrap_or(&String::new()).clone();

    let description = parts.get(2).unwrap_or(&String::new()).clone();

    let dispatcher_str = match parts.get(3) {
        Some(_) => parts[3..].join(", "),
        None => String::new(),
    };

    let dispatcher = dispatcher_str.parse().unwrap_or(Dispatcher::default());

    BindRight {
        mods,
        key,
        dispatcher,
        description: Some(description),
    }
}

pub fn parse_bind_right(has_description: bool, bind_right_str: &str) -> BindRight {
    if has_description {
        parse_bind_with_description(bind_right_str)
    } else {
        BindRight::from_str(bind_right_str).unwrap_or(BindRight {
            mods: HashSet::new(),
            key: String::new(),
            dispatcher: Dispatcher::default(),
            description: None,
        })
    }
}
