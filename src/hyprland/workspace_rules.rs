use super::Orientation;
use crate::utils::{parse_bool, parse_int};
use std::{fmt::Display, str::FromStr};

#[derive(Default, Debug)]
pub struct WorkspaceRules {
    pub monitor: Option<String>,
    pub default: Option<bool>,
    pub gaps_in: Option<i32>,
    pub gaps_out: Option<i32>,
    pub border_size: Option<i32>,
    pub border: Option<bool>,
    pub shadow: Option<bool>,
    pub rounding: Option<bool>,
    pub decorate: Option<bool>,
    pub persistent: Option<bool>,
    pub on_created_empty: Option<String>,
    pub default_name: Option<String>,
    pub layoutopt_orientation: Option<Orientation>,
}

impl Display for WorkspaceRules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rules = Vec::new();

        if let Some(monitor) = &self.monitor {
            rules.push(format!("monitor:{}", monitor));
        }
        if let Some(default) = self.default {
            rules.push(format!("default:{}", default));
        }
        if let Some(gaps_in) = self.gaps_in {
            rules.push(format!("gapsin:{}", gaps_in));
        }
        if let Some(gaps_out) = self.gaps_out {
            rules.push(format!("gapsout:{}", gaps_out));
        }
        if let Some(border_size) = self.border_size {
            rules.push(format!("bordersize:{}", border_size));
        }
        if let Some(border) = self.border {
            rules.push(format!("border:{}", border));
        }
        if let Some(shadow) = self.shadow {
            rules.push(format!("shadow:{}", shadow));
        }
        if let Some(rounding) = self.rounding {
            rules.push(format!("rounding:{}", rounding));
        }
        if let Some(decorate) = self.decorate {
            rules.push(format!("decorate:{}", decorate));
        }
        if let Some(persistent) = self.persistent {
            rules.push(format!("persistent:{}", persistent));
        }
        if let Some(on_created_empty) = &self.on_created_empty {
            rules.push(format!("on-created-empty:{}", on_created_empty));
        }
        if let Some(default_name) = &self.default_name {
            rules.push(format!("defaultName:{}", default_name));
        }
        if let Some(layoutopt_orientation) = &self.layoutopt_orientation {
            rules.push(format!("layoutopt:orientation:{}", layoutopt_orientation));
        }

        write!(f, "{}", rules.join(", "))
    }
}

pub fn parse_workspace_rule(input: &str, rules: &mut WorkspaceRules) {
    let input = match input.strip_prefix("layoutopt:") {
        Some(value) => value,
        None => return,
    };
    let parts: Vec<&str> = input.splitn(2, ':').collect();
    if parts.len() != 2 {
        return;
    }

    let rule_name = parts[0].trim();
    let rule_value = parts[1].trim();

    match rule_name {
        "monitor" => rules.monitor = Some(rule_value.to_string()),
        "default" => rules.default = parse_bool(rule_value),
        "gapsin" => rules.gaps_in = parse_int(rule_value),
        "gapsout" => rules.gaps_out = parse_int(rule_value),
        "bordersize" => rules.border_size = parse_int(rule_value),
        "border" => rules.border = parse_bool(rule_value),
        "shadow" => rules.shadow = parse_bool(rule_value),
        "rounding" => rules.rounding = parse_bool(rule_value),
        "decorate" => rules.decorate = parse_bool(rule_value),
        "persistent" => rules.persistent = parse_bool(rule_value),
        "on-created-empty" => rules.on_created_empty = Some(rule_value.to_string()),
        "defaultName" => rules.default_name = Some(rule_value.to_string()),
        "orientation" => {
            rules.layoutopt_orientation =
                Some(Orientation::from_str(rule_value).expect("Invalid orientation"))
        }
        _ => {}
    }
}
