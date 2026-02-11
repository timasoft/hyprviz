use super::{WorkspaceSelector, workspace_selector::parse_single_selector};
use rust_i18n::t;
use std::fmt::Display;

#[derive(Debug)]
pub enum WorkspaceType {
    Named(String),
    Special(String),
    Numbered(u32),
    Selector(Vec<WorkspaceSelector>),
}

impl WorkspaceType {
    pub fn get_fancy_list() -> [String; 4] {
        [
            t!("hyprland.workspace_type.named").to_string(),
            t!("hyprland.workspace_type.special").to_string(),
            t!("hyprland.workspace_type.numbered").to_string(),
            t!("hyprland.workspace_type.selector").to_string(),
        ]
    }
}

impl Display for WorkspaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceType::Named(name) => write!(f, "name:{}", name),
            WorkspaceType::Special(name) => write!(f, "special:{}", name),
            WorkspaceType::Numbered(num) => write!(f, "{}", num),
            WorkspaceType::Selector(selector) => write!(
                f,
                "{}",
                selector
                    .iter()
                    .map(|selector| selector.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            ),
        }
    }
}

pub fn parse_workspace_selector(input: &str) -> Vec<WorkspaceSelector> {
    let mut selectors = Vec::new();
    let mut remaining = input.trim();

    while !remaining.is_empty() {
        let selector_result = parse_single_selector(remaining);

        match selector_result {
            Some((selector, rest)) => {
                selectors.push(selector);
                remaining = rest.trim();
            }
            None => {
                break;
            }
        }
    }

    selectors
}

pub fn parse_workspace_type(input: &str) -> WorkspaceType {
    if let Some(name) = input.strip_prefix("name:") {
        WorkspaceType::Named(name.to_string())
    } else if let Some(name) = input.strip_prefix("special:") {
        WorkspaceType::Special(name.to_string())
    } else if let Ok(num) = input.parse::<u32>() {
        WorkspaceType::Numbered(num)
    } else {
        WorkspaceType::Selector(parse_workspace_selector(input))
    }
}
