use super::{WindowRule, WindowRuleParameter};
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
pub struct WindowRuleWithParameters {
    pub rule: WindowRule,
    pub parameters: Vec<WindowRuleParameter>,
}

impl FromStr for WindowRuleWithParameters {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if parts.is_empty() {
            return Err(());
        }

        let rule: WindowRule = parts[0].parse().unwrap_or_default();

        let parameters: Vec<WindowRuleParameter> = parts
            .iter()
            .skip(1)
            .filter_map(|s| s.parse::<WindowRuleParameter>().ok())
            .collect();

        Ok(WindowRuleWithParameters { rule, parameters })
    }
}

impl Display for WindowRuleWithParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}",
            self.rule,
            join_with_separator(&self.parameters, ", ")
        )
    }
}
impl ToGtkBox for WindowRuleWithParameters {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let window_rule_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        window_rule_box_box.append(&Label::new(Some(&t!("gtk_converters.rule"))));
        let window_rule_entry = create_entry();
        let window_rule_box = WindowRule::to_gtk_box(&window_rule_entry);
        window_rule_box_box.append(&window_rule_box);
        mother_box.append(&window_rule_box_box);

        let parameters_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        parameters_box_box.append(&Label::new(Some(&t!("gtk_converters.parameters"))));
        let parameters_entry = create_entry();
        let parameters_box = Vec::<WindowRuleParameter>::to_gtk_box(&parameters_entry, ',');
        parameters_box_box.append(&parameters_box);
        mother_box.append(&parameters_box_box);

        let window_rule_entry_clone = window_rule_entry.clone();
        let parameters_entry_clone = parameters_entry.clone();
        let update_ui = move |window_rule_with_parameters: WindowRuleWithParameters| {
            window_rule_entry_clone.set_text(&window_rule_with_parameters.rule.to_string());
            parameters_entry_clone.set_text(&join_with_separator(
                window_rule_with_parameters.parameters,
                ", ",
            ));
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        window_rule_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut window_rule_with_parameters: WindowRuleWithParameters =
                entry_clone.text().parse().unwrap_or_default();
            window_rule_with_parameters.rule = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&window_rule_with_parameters.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        parameters_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut window_rule_with_parameters: WindowRuleWithParameters =
                entry_clone.text().parse().unwrap_or_default();
            window_rule_with_parameters.parameters = entry
                .text()
                .split(',')
                .map(|s| s.trim().parse().unwrap_or_default())
                .collect();
            entry_clone.set_text(&window_rule_with_parameters.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let window_rule_with_parameters: WindowRuleWithParameters =
                entry.text().parse().unwrap_or_default();
            update_ui(window_rule_with_parameters);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(WindowRuleWithParameters);
