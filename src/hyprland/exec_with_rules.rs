use super::WindowRule;
use crate::{
    advanced_editors::create_entry,
    gtk_converters::{ToGtkBox, ToGtkBoxWithSeparator},
    register_togtkbox,
    utils::join_with_separator,
};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExecWithRules {
    pub rules: Vec<WindowRule>,
    pub command: String,
}

impl FromStr for ExecWithRules {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let str = s.trim_start();

        if str.starts_with('[') {
            let mut rules = Vec::new();
            let mut rule = String::new();
            let mut in_brackets = false;
            let mut command = String::new();

            for c in str.chars() {
                if c == '[' {
                    in_brackets = true;
                } else if c == ']' {
                    if !rule.trim().is_empty() {
                        rules.push(rule.parse().unwrap_or_default());
                        rule.clear();
                    }
                    in_brackets = false;
                } else if c == ';' && in_brackets {
                    if !rule.trim().is_empty() {
                        rules.push(rule.parse().unwrap_or_default());
                        rule.clear();
                    }
                } else if in_brackets {
                    rule.push(c);
                } else {
                    command.push(c);
                }
            }

            Ok(Self {
                rules,
                command: command.trim_start().to_string(),
            })
        } else {
            Ok(Self {
                rules: Vec::new(),
                command: str.to_string(),
            })
        }
    }
}

impl Display for ExecWithRules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rules.is_empty() {
            true => write!(f, "{}", self.command),
            false => write!(
                f,
                "[{}] {}",
                join_with_separator(&self.rules, "; "),
                self.command
            ),
        }
    }
}

impl ToGtkBox for ExecWithRules {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let window_rules_mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        window_rules_mother_box.append(&Label::new(Some(&t!(
            "hyprland.exec_with_rules.window_rules"
        ))));
        let window_rules_entry = create_entry();
        let window_rules_box = Vec::<WindowRule>::to_gtk_box(&window_rules_entry, ';');
        window_rules_mother_box.append(&window_rules_box);
        mother_box.append(&window_rules_mother_box);

        let command_box = GtkBox::new(GtkOrientation::Vertical, 5);
        command_box.append(&Label::new(Some(&t!("hyprland.exec_with_rules.command"))));
        let command_entry = create_entry();
        command_box.append(&command_entry);
        mother_box.append(&command_box);

        let window_rules_entry_clone = window_rules_entry.clone();
        let command_entry_clone = command_entry.clone();
        let update_ui = move |exec_with_rules: ExecWithRules| {
            window_rules_entry_clone.set_text(&join_with_separator(&exec_with_rules.rules, ";"));
            command_entry_clone.set_text(&exec_with_rules.command);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let command_entry_clone = command_entry.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        window_rules_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            match entry.text().is_empty() {
                true => entry_clone.set_text(&command_entry_clone.text()),
                false => entry_clone.set_text(&format!(
                    "[{}] {}",
                    entry.text(),
                    command_entry_clone.text()
                )),
            }

            is_updating_clone.set(false);
        });

        let window_rules_entry_clone = window_rules_entry.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        command_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            match window_rules_entry_clone.text().is_empty() {
                true => entry_clone.set_text(&entry.text()),
                false => entry_clone.set_text(&format!(
                    "[{}] {}",
                    window_rules_entry_clone.text(),
                    entry.text()
                )),
            }

            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let exec_with_rules: ExecWithRules = entry.text().parse().unwrap_or_default();
            update_ui(exec_with_rules);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(ExecWithRules);
