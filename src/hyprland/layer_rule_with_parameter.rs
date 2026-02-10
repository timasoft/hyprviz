use super::{LayerRule, NamespaceOrAddress};
use crate::{advanced_editors::create_entry, gtk_converters::ToGtkBox, register_togtkbox};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerRuleWithParameter {
    pub rule: LayerRule,
    pub namespace_or_address: NamespaceOrAddress,
}

impl FromStr for LayerRuleWithParameter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (rule_str, namespace_or_address_str) = s.split_once(',').unwrap_or((s, ""));

        let rule = rule_str.parse().unwrap_or_default();

        let namespace_or_address = namespace_or_address_str.parse().unwrap_or_default();

        Ok(LayerRuleWithParameter {
            rule,
            namespace_or_address,
        })
    }
}

impl Display for LayerRuleWithParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.rule, self.namespace_or_address)
    }
}

impl ToGtkBox for LayerRuleWithParameter {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let rule_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        rule_box_box.append(&Label::new(Some(&t!("gtk_converters.rule"))));
        let rule_entry = create_entry();
        let rule_box = LayerRule::to_gtk_box(&rule_entry);
        rule_box_box.append(&rule_box);
        mother_box.append(&rule_box_box);

        let namespace_or_address_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        namespace_or_address_box_box.append(&Label::new(Some(&t!(
            "gtk_converters.namespace_or_address"
        ))));
        let namespace_or_address_entry = create_entry();
        let namespace_or_address_box = NamespaceOrAddress::to_gtk_box(&namespace_or_address_entry);
        namespace_or_address_box_box.append(&namespace_or_address_box);
        mother_box.append(&namespace_or_address_box_box);

        let rule_entry_clone = rule_entry.clone();
        let namespace_or_address_entry_clone = namespace_or_address_entry.clone();
        let update_ui = move |layer_rule_with_parameter: LayerRuleWithParameter| {
            rule_entry_clone.set_text(&layer_rule_with_parameter.rule.to_string());
            namespace_or_address_entry_clone
                .set_text(&layer_rule_with_parameter.namespace_or_address.to_string());
        };

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        rule_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut layer_rule_with_parameter: LayerRuleWithParameter =
                entry_clone.text().parse().unwrap_or_default();
            layer_rule_with_parameter.rule = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&layer_rule_with_parameter.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        namespace_or_address_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut layer_rule_with_parameter: LayerRuleWithParameter =
                entry_clone.text().parse().unwrap_or_default();
            layer_rule_with_parameter.namespace_or_address =
                entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&layer_rule_with_parameter.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let layer_rule_with_parameter: LayerRuleWithParameter =
                entry.text().parse().unwrap_or_default();
            update_ui(layer_rule_with_parameter);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(LayerRuleWithParameter);
