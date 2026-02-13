use super::{
    WorkspaceRules, WorkspaceType, workspace_rules::parse_workspace_rule,
    workspace_type::parse_workspace_type,
};
use std::fmt::Display;

#[derive(Debug)]
pub struct Workspace {
    pub workspace_type: WorkspaceType,
    pub rules: WorkspaceRules,
}

impl Display for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let workspace_type = &self.workspace_type;
        let rules = &self.rules;
        if rules.monitor.is_some()
            || rules.default.is_some()
            || rules.gaps_in.is_some()
            || rules.gaps_out.is_some()
            || rules.border_size.is_some()
            || rules.border.is_some()
            || rules.shadow.is_some()
            || rules.rounding.is_some()
            || rules.decorate.is_some()
            || rules.persistent.is_some()
            || rules.on_created_empty.is_some()
            || rules.default_name.is_some()
            || rules.layoutopt_orientation.is_some()
        {
            write!(f, "{}, {}", workspace_type, rules)
        } else {
            write!(f, "{}", workspace_type)
        }
    }
}

pub fn parse_workspace(input: &str) -> Workspace {
    let values: Vec<String> = input.split(',').map(|s| s.trim().to_string()).collect();

    if values.is_empty() {
        return Workspace {
            workspace_type: WorkspaceType::Selector(Vec::new()),
            rules: WorkspaceRules::default(),
        };
    }

    let workspace_str = values.first().expect("Workspace type not found").as_str();
    let workspace_type = parse_workspace_type(workspace_str);
    let mut rules = WorkspaceRules::default();

    for rule_str in values.iter().skip(1) {
        parse_workspace_rule(rule_str, &mut rules);
    }

    Workspace {
        workspace_type,
        rules,
    }
}
