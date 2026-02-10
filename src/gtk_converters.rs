use crate::{
    advanced_editors::{
        create_button, create_dropdown, create_entry, create_spin_button, create_switch,
    },
    utils::HasDiscriminant,
};
use gtk::{
    Box as GtkBox, Entry, Label, Orientation as GtkOrientation, Stack, StringList, prelude::*,
};
use rust_i18n::t;
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    fmt::Display,
    hash::Hash,
    rc::Rc,
    str::FromStr,
};
use strum::IntoEnumIterator;

pub const PLUG_SEPARATOR: char = '︲';

#[derive(Debug, Clone, Default)]
pub enum FieldLabel {
    Named(&'static str),
    #[default]
    Unnamed,
}

pub type ToGtkBoxWithSeparatorAndNamesBuilder =
    fn(&Entry, char, &[FieldLabel], custom_split: Option<fn(&str) -> Vec<&str>>) -> GtkBox;

pub trait EnumConfigForGtk {
    const SEPARATOR: Option<char> = None;

    fn dropdown_items() -> StringList;

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        None
    }

    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        None
    }
}

pub trait ToGtkBox {
    fn to_gtk_box(entry: &Entry) -> GtkBox;
}
pub struct ToGtkBoxImplementation {
    pub name: &'static str,
    pub constructor: fn(&Entry) -> GtkBox,
}
inventory::collect!(ToGtkBoxImplementation);
#[macro_export]
macro_rules! register_togtkbox {
    ($($ty:ty),* $(,)?) => {
        $(
            inventory::submit! {
                $crate::gtk_converters::ToGtkBoxImplementation {
                    name: stringify!($ty),
                    constructor: <$ty as $crate::gtk_converters::ToGtkBox>::to_gtk_box,
                }
            }
        )*
    };
}

pub trait ToGtkBoxWithSeparator {
    fn to_gtk_box(entry: &Entry, separator: char) -> GtkBox;
}
pub struct ToGtkBoxWithSeparatorImplementation {
    pub name: &'static str,
    pub constructor: fn(&Entry, char) -> GtkBox,
}
inventory::collect!(ToGtkBoxWithSeparatorImplementation);
#[macro_export]
macro_rules! register_togtkbox_with_separator {
    ($($ty:ty),* $(,)?) => {
        $(
            inventory::submit! {
                $crate::gtk_converters::ToGtkBoxWithSeparatorImplementation {
                    name: stringify!($ty),
                    constructor: <$ty as $crate::gtk_converters::ToGtkBoxWithSeparator>::to_gtk_box,
                }
            }
        )*
    };
}

pub trait ToGtkBoxWithSeparatorAndNames {
    fn to_gtk_box(
        entry: &Entry,
        separator: char,
        names: &[FieldLabel],
        custom_split: Option<fn(&str) -> Vec<&str>>,
    ) -> GtkBox;
}
pub struct ToGtkBoxWithSeparatorAndNamesImplementation {
    pub name: &'static str,
    pub constructor: ToGtkBoxWithSeparatorAndNamesBuilder,
}
inventory::collect!(ToGtkBoxWithSeparatorAndNamesImplementation);
#[macro_export]
macro_rules! register_togtkbox_with_separator_names {
    ($($ty:ty),* $(,)?) => {
        $(
            inventory::submit! {
                $crate::gtk_converters::ToGtkBoxWithSeparatorAndNamesImplementation {
                    name: stringify!($ty),
                    constructor: <$ty as $crate::gtk_converters::ToGtkBoxWithSeparatorAndNames>::to_gtk_box,
                }
            }
        )*
    };
}

impl<T> ToGtkBox for T
where
    T: Display
        + FromStr
        + Default
        + PartialEq
        + Clone
        + HasDiscriminant
        + EnumConfigForGtk
        + 'static,
{
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let string_list = T::dropdown_items();
        let is_updating = Rc::new(Cell::new(false));
        let empty_vec = Vec::new();

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let dropdown = create_dropdown(&string_list);
        dropdown.set_selected(0);
        mother_box.append(&dropdown);

        let stack = Stack::new();
        let mut variant_entries = Vec::new();
        let field_labels = T::field_labels().unwrap_or_default();

        if let Some(separator) = T::SEPARATOR {
            for (i, discriminant) in T::Discriminant::iter().enumerate() {
                let param_entry = create_entry();
                variant_entries.push(param_entry.clone());

                if let Some(builder) = T::from_discriminant(discriminant).parameter_builder() {
                    let labels = field_labels.get(i).unwrap_or(&empty_vec);
                    let variant_box = builder(
                        &param_entry,
                        separator,
                        labels,
                        T::custom_split(discriminant),
                    );
                    stack.add_named(&variant_box, Some(&format!("v{}", i)));
                } else {
                    stack.add_named(
                        &GtkBox::new(GtkOrientation::Vertical, 0),
                        Some(&format!("v{}", i)),
                    );
                }
            }
            mother_box.append(&stack);
        }

        let dropdown_clone = dropdown.clone();
        let variant_entries_clone = variant_entries.clone();
        let get_value = move |variant_index: Option<usize>| -> T {
            let variant_index = variant_index.unwrap_or(dropdown_clone.selected() as usize);
            let param_text = variant_entries_clone
                .get(variant_index)
                .map(|e| e.text().to_string())
                .unwrap_or_default();

            T::from_discriminant_and_str(
                T::Discriminant::iter()
                    .nth(variant_index)
                    .expect("variant_index out of bounds"),
                &param_text,
            )
        };

        let dropdown_clone = dropdown.clone();
        let stack_clone = stack.clone();
        let variant_entries_clone = variant_entries.clone();
        let update_ui = move |value: T| {
            let variant_index = value.variant_index();
            let current_selection = dropdown_clone.selected() as usize;
            if current_selection != variant_index {
                dropdown_clone.set_selected(variant_index as u32);
            }

            if let Some(_separator) = T::SEPARATOR {
                stack_clone.set_visible_child_name(&format!("v{}", variant_index));
            }

            if let Some(param_entry) = variant_entries_clone.get(variant_index) {
                let new_text = value.to_str_without_discriminant().unwrap_or_default();
                if param_entry.text() != new_text {
                    param_entry.set_text(&new_text);
                }
            }
        };

        let initial_value: T = entry.text().to_string().parse().unwrap_or_default();
        update_ui(initial_value);

        for (i, param_entry) in variant_entries.into_iter().enumerate() {
            let get_value_clone = get_value.clone();
            let entry_clone = entry.clone();
            let is_updating_clone = is_updating.clone();
            param_entry.connect_changed(move |_| {
                if is_updating_clone.get() {
                    return;
                }
                is_updating_clone.set(true);
                let new_value = get_value_clone(Some(i));
                let new_text = new_value.to_string();
                if entry_clone.text() != new_text {
                    entry_clone.set_text(&new_text);
                }
                is_updating_clone.set(false);
            });
        }

        let get_value_clone = get_value.clone();
        let update_ui_clone = update_ui.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        dropdown.connect_selected_notify(move |dropdown| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let variant_index = dropdown.selected() as usize;
            let new_value = get_value_clone(Some(variant_index));
            let new_text = new_value.to_string();
            if entry_clone.text() != new_text {
                entry_clone.set_text(&new_text);
            }
            update_ui_clone(new_value);
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let new_value: T = entry.text().to_string().parse().unwrap_or_default();
            update_ui(new_value);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl<T: ToGtkBox> ToGtkBox for Option<T> {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let is_some_switch = create_switch();
        mother_box.append(&is_some_switch);

        let sub_box = T::to_gtk_box(entry);
        sub_box.set_visible(false);
        mother_box.append(&sub_box);

        let entry_clone = entry.clone();
        let sub_box_clone = sub_box.clone();
        let is_updating_clone = is_updating.clone();
        is_some_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            if switch.is_active() {
                sub_box_clone.set_visible(true);
                entry_clone.set_text("");
            } else {
                sub_box_clone.set_visible(false);
                entry_clone.set_text("");
            }
            is_updating_clone.set(false);
        });

        let is_some_switch_clone = is_some_switch.clone();
        let sub_box_clone = sub_box.clone();
        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            match entry.text().as_str() {
                "" => {
                    sub_box_clone.set_visible(false);
                    is_some_switch_clone.set_active(false);
                }
                _ => {
                    sub_box_clone.set_visible(true);
                    is_some_switch_clone.set_active(true);
                }
            }
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl<T: ToGtkBox + Default + Display> ToGtkBoxWithSeparator for Vec<T> {
    fn to_gtk_box(entry: &Entry, separator: char) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let join_separator = separator.to_string();

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let add_button = create_button(&t!("gtk_converters.add"));

        let mother_box_clone = mother_box.clone();
        let add_button_clone = add_button.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        let join_separator_clone = join_separator.clone();
        let rebuild_ui = move |text: &str| {
            while let Some(child) = mother_box_clone.first_child() {
                mother_box_clone.remove(&child);
            }

            let mut remove_buttons = Vec::new();
            let parts: Vec<&str> = text.split(separator).collect();

            for (i, part) in parts.iter().enumerate() {
                let part_box = GtkBox::new(GtkOrientation::Vertical, 5);

                let remove_button = create_button(&t!("gtk_converters.remove"));
                part_box.append(&remove_button);
                remove_buttons.push(remove_button.clone());

                let sub_entry = create_entry();
                let sub_box = T::to_gtk_box(&sub_entry);
                sub_entry.set_text(part.trim());
                part_box.append(&sub_box);

                let entry_clone_clone = entry_clone.clone();
                let is_updating_clone_clone = is_updating_clone.clone();
                let join_sep = join_separator_clone.clone();
                let index = i;

                sub_entry.connect_changed(move |sub_entry_widget| {
                    if is_updating_clone_clone.get() {
                        return;
                    }
                    is_updating_clone_clone.set(true);

                    let new_value = sub_entry_widget.text().trim().to_string();
                    let current_text = entry_clone_clone.text().to_string();

                    let mut parts_vec: Vec<String> = current_text
                        .split(separator)
                        .map(|s| s.to_string())
                        .collect();

                    if index < parts_vec.len() {
                        parts_vec[index] = new_value;
                        let updated_text = parts_vec.join(&join_sep);
                        entry_clone_clone.set_text(&updated_text);
                    }

                    is_updating_clone_clone.set(false);
                });

                mother_box_clone.append(&part_box);
            }

            mother_box_clone.append(&add_button_clone);
            remove_buttons
        };

        let entry_clone_add = entry.clone();
        let join_separator_add = join_separator.clone();
        add_button.connect_clicked(move |_| {
            let current_text = entry_clone_add.text().to_string();
            let parts: Vec<String> = current_text
                .split(separator)
                .map(|s| s.to_string())
                .collect();

            let mut new_parts = parts;
            new_parts.push(T::default().to_string());
            let updated_text = new_parts.join(&join_separator_add);
            entry_clone_add.set_text(&updated_text);
        });

        let rebuild_ui_with_remove_buttons = {
            let join_separator_rm = join_separator.clone();
            move |entry_widget: &Entry| {
                let remove_buttons = rebuild_ui(entry_widget.text().as_str());
                for (i, remove_button) in remove_buttons.into_iter().enumerate() {
                    let entry_clone_rm = entry_widget.clone();
                    let join_sep = join_separator_rm.clone();
                    let index = i;

                    remove_button.connect_clicked(move |_| {
                        let current_text = entry_clone_rm.text().to_string();
                        let mut parts_vec: Vec<String> = current_text
                            .split(separator)
                            .map(|s| s.to_string())
                            .collect();

                        if index < parts_vec.len() {
                            parts_vec.remove(index);
                            let updated_text = parts_vec.join(&join_sep);
                            entry_clone_rm.set_text(&updated_text);
                        }
                    });
                }
            }
        };

        rebuild_ui_with_remove_buttons(entry);

        let is_updating_main = is_updating.clone();
        entry.connect_changed(move |entry_widget| {
            if is_updating_main.get() {
                return;
            }
            is_updating_main.set(true);
            rebuild_ui_with_remove_buttons(entry_widget);
            is_updating_main.set(false);
        });

        mother_box
    }
}

impl<T: ToGtkBox + Default + Display + FromStr + Clone + Eq + Hash + 'static> ToGtkBoxWithSeparator
    for HashSet<T>
{
    fn to_gtk_box(entry: &Entry, separator: char) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let join_separator = separator.to_string();

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let add_button = create_button(&t!("gtk_converters.add"));

        let mother_box_clone = mother_box.clone();
        let add_button_clone = add_button.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        let join_separator_clone = join_separator.clone();
        let rebuild_ui = move |text: &str| {
            while let Some(child) = mother_box_clone.first_child() {
                mother_box_clone.remove(&child);
            }

            let mut remove_buttons = Vec::new();
            let parts: Vec<&str> = text
                .split(&join_separator_clone)
                .filter(|s| !s.is_empty())
                .collect();

            let mut displayed_values = HashSet::new();

            for part in parts {
                let value: T = part.parse().unwrap_or_default();
                if !displayed_values.insert(value.clone()) {
                    continue;
                }

                let item_box = GtkBox::new(GtkOrientation::Horizontal, 5);
                item_box.set_margin_start(5);
                item_box.set_margin_end(5);
                item_box.set_margin_top(2);
                item_box.set_margin_bottom(2);

                let remove_button = create_button(&t!("gtk_converters.remove"));
                item_box.append(&remove_button);

                let sub_entry = create_entry();
                let sub_box = T::to_gtk_box(&sub_entry);
                let initial_text = part.trim().to_string();
                sub_entry.set_text(&initial_text);
                item_box.append(&sub_box);

                let prev_text = Rc::new(RefCell::new(initial_text));

                let entry_clone_clone = entry_clone.clone();
                let is_updating_clone_clone = is_updating_clone.clone();
                let join_separator_clone_clone = join_separator_clone.clone();
                let prev_text_clone = prev_text.clone();
                sub_entry.connect_changed(move |sub_entry| {
                    if is_updating_clone_clone.get() {
                        return;
                    }
                    is_updating_clone_clone.set(true);

                    let new_text = sub_entry.text().trim().to_string();
                    let old_text = prev_text_clone.borrow().clone();
                    prev_text_clone.replace(new_text.clone());

                    let current_text = entry_clone_clone.text().to_string();
                    let mut current_set: HashSet<T> = current_text
                        .split(&join_separator_clone_clone)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.parse().unwrap_or_default())
                        .collect();

                    if !old_text.is_empty()
                        && let Ok(old_value) = old_text.parse::<T>()
                    {
                        current_set.remove(&old_value);
                    }

                    if !new_text.is_empty()
                        && let Ok(new_value) = new_text.parse::<T>()
                    {
                        current_set.insert(new_value);
                    }

                    let updated_text = current_set
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(&join_separator_clone_clone);
                    entry_clone_clone.set_text(&updated_text);

                    is_updating_clone_clone.set(false);
                });

                mother_box_clone.append(&item_box);
                remove_buttons.push((value, remove_button));
            }

            mother_box_clone.append(&add_button_clone);
            remove_buttons
        };

        let entry_clone = entry.clone();
        let join_separator_clone = join_separator.clone();
        add_button.connect_clicked(move |_| {
            let current_text = entry_clone.text().to_string();
            let mut set: HashSet<T> = current_text
                .split(&join_separator_clone)
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().unwrap_or_default())
                .collect();

            set.insert(T::default());

            let updated_text = set
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(&join_separator_clone);
            entry_clone.set_text(&updated_text);
        });

        let rebuild_ui_with_remove_buttons = {
            let join_separator_clone = join_separator.clone();
            move |entry: &Entry| {
                let remove_buttons = rebuild_ui(entry.text().as_str());
                for (value, remove_button) in remove_buttons {
                    let entry_clone = entry.clone();
                    let join_separator_clone_clone = join_separator_clone.clone();
                    let value_clone = value.clone();
                    remove_button.connect_clicked(move |_| {
                        let current_text = entry_clone.text().to_string();
                        let mut set: HashSet<T> = current_text
                            .split(&join_separator_clone_clone)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.parse().unwrap_or_default())
                            .collect();

                        set.remove(&value_clone);
                        let updated_text = set
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(&join_separator_clone_clone);
                        entry_clone.set_text(&updated_text);
                    });
                }
            }
        };

        rebuild_ui_with_remove_buttons(entry);

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            rebuild_ui_with_remove_buttons(entry);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl<T: ToGtkBox + Default + Display> ToGtkBoxWithSeparatorAndNames for (T,) {
    fn to_gtk_box(
        entry: &Entry,
        _separator: char,
        names: &[FieldLabel],
        _custom_split: Option<fn(&str) -> Vec<&str>>,
    ) -> GtkBox {
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        if let Some(FieldLabel::Named(name)) = names.first() {
            mother_box.append(&Label::new(Some(name)));
        }
        let t_box = T::to_gtk_box(entry);
        mother_box.append(&t_box);

        mother_box
    }
}

impl<T: ToGtkBox + Default + Display, N: ToGtkBox + Default + Display> ToGtkBoxWithSeparatorAndNames
    for (T, N)
{
    fn to_gtk_box(
        entry: &Entry,
        separator: char,
        names: &[FieldLabel],
        custom_split: Option<fn(&str) -> Vec<&str>>,
    ) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        if let Some(FieldLabel::Named(name)) = names.first() {
            mother_box.append(&Label::new(Some(name)));
        }
        let t_entry = create_entry();
        let t_box = T::to_gtk_box(&t_entry);
        mother_box.append(&t_box);

        if let Some(FieldLabel::Named(name)) = names.get(1) {
            mother_box.append(&Label::new(Some(name)));
        }
        let n_entry = create_entry();
        let n_box = N::to_gtk_box(&n_entry);
        mother_box.append(&n_box);

        let t_entry_clone = t_entry.clone();
        let n_entry_clone = n_entry.clone();
        let update_ui = move |text: &str| {
            let parts: Vec<&str> = match custom_split {
                Some(custom_split) => custom_split(text),
                None => text.split(separator).collect(),
            };
            if parts.len() >= 2 {
                t_entry_clone.set_text(parts[0]);
                n_entry_clone.set_text(parts[1]);
            } else if parts.len() == 1 {
                t_entry_clone.set_text(parts[0]);
                n_entry_clone.set_text("");
            }
        };
        update_ui(entry.text().as_str());

        let t_entry_clone = t_entry.clone();
        let n_entry_clone = n_entry.clone();
        let combine_values = move || {
            let t_val = t_entry_clone.text().to_string();
            let n_val = n_entry_clone.text().to_string();
            if separator == PLUG_SEPARATOR {
                format!("{}{}", t_val, n_val)
            } else {
                format!("{}{}{}", t_val, separator, n_val)
            }
        };

        let combine_values_clone = combine_values.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        t_entry.connect_changed(move |_| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&combine_values_clone());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        n_entry.connect_changed(move |_| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&combine_values());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            update_ui(entry.text().as_str());
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl<
    T: ToGtkBox + Default + Display,
    N: ToGtkBox + Default + Display,
    M: ToGtkBox + Default + Display,
> ToGtkBoxWithSeparatorAndNames for (T, N, M)
{
    fn to_gtk_box(
        entry: &Entry,
        separator: char,
        names: &[FieldLabel],
        custom_split: Option<fn(&str) -> Vec<&str>>,
    ) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        if let Some(FieldLabel::Named(name)) = names.first() {
            mother_box.append(&Label::new(Some(name)));
        }
        let t_entry = create_entry();
        let t_box = T::to_gtk_box(&t_entry);
        mother_box.append(&t_box);

        if let Some(FieldLabel::Named(name)) = names.get(1) {
            mother_box.append(&Label::new(Some(name)));
        }
        let n_entry = create_entry();
        let n_box = N::to_gtk_box(&n_entry);
        mother_box.append(&n_box);

        if let Some(FieldLabel::Named(name)) = names.get(2) {
            mother_box.append(&Label::new(Some(name)));
        }
        let m_entry = create_entry();
        let m_box = M::to_gtk_box(&m_entry);
        mother_box.append(&m_box);

        let t_entry_clone = t_entry.clone();
        let n_entry_clone = n_entry.clone();
        let m_entry_clone = m_entry.clone();
        let update_ui = move |text: &str| {
            let parts: Vec<&str> = match custom_split {
                Some(custom_split) => custom_split(text),
                None => text.split(separator).collect(),
            };
            if parts.len() >= 3 {
                t_entry_clone.set_text(parts[0]);
                n_entry_clone.set_text(parts[1]);
                m_entry_clone.set_text(parts[2]);
            } else if parts.len() == 2 {
                t_entry_clone.set_text(parts[0]);
                n_entry_clone.set_text(parts[1]);
                m_entry_clone.set_text("");
            } else if parts.len() == 1 {
                t_entry_clone.set_text(parts[0]);
                n_entry_clone.set_text("");
                m_entry_clone.set_text("");
            }
        };
        update_ui(entry.text().as_str());

        let t_entry_clone = t_entry.clone();
        let n_entry_clone = n_entry.clone();
        let m_entry_clone = m_entry.clone();
        let combine_values = move || {
            let t_val = t_entry_clone.text().to_string();
            let n_val = n_entry_clone.text().to_string();
            let m_val = m_entry_clone.text().to_string();
            if separator == PLUG_SEPARATOR {
                format!("{}{}{}", t_val, n_val, m_val)
            } else {
                format!("{}{}{}{}{}", t_val, separator, n_val, separator, m_val)
            }
        };

        let combine_values_clone = combine_values.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        t_entry.connect_changed(move |_| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&combine_values_clone());
            is_updating_clone.set(false);
        });

        let combine_values_clone = combine_values.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        n_entry.connect_changed(move |_| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&combine_values_clone());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        m_entry.connect_changed(move |_| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&combine_values());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            update_ui(entry.text().as_str());
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl<
    T: ToGtkBox + Default + Display,
    N: ToGtkBox + Default + Display,
    M: ToGtkBox + Default + Display,
    B: ToGtkBox + Default + Display,
> ToGtkBoxWithSeparatorAndNames for (T, N, M, B)
{
    fn to_gtk_box(
        entry: &Entry,
        separator: char,
        names: &[FieldLabel],
        custom_split: Option<fn(&str) -> Vec<&str>>,
    ) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        if let Some(FieldLabel::Named(name)) = names.first() {
            mother_box.append(&Label::new(Some(name)));
        }
        let t_entry = create_entry();
        let t_box = T::to_gtk_box(&t_entry);
        mother_box.append(&t_box);

        if let Some(FieldLabel::Named(name)) = names.get(1) {
            mother_box.append(&Label::new(Some(name)));
        }
        let n_entry = create_entry();
        let n_box = N::to_gtk_box(&n_entry);
        mother_box.append(&n_box);

        if let Some(FieldLabel::Named(name)) = names.get(2) {
            mother_box.append(&Label::new(Some(name)));
        }
        let m_entry = create_entry();
        let m_box = M::to_gtk_box(&m_entry);
        mother_box.append(&m_box);

        if let Some(FieldLabel::Named(name)) = names.get(3) {
            mother_box.append(&Label::new(Some(name)));
        }
        let b_entry = create_entry();
        let b_box = B::to_gtk_box(&b_entry);
        mother_box.append(&b_box);

        let t_entry_clone = t_entry.clone();
        let n_entry_clone = n_entry.clone();
        let m_entry_clone = m_entry.clone();
        let b_entry_clone = b_entry.clone();
        let update_ui = move |text: &str| {
            let parts: Vec<&str> = match custom_split {
                Some(custom_split) => custom_split(text),
                None => text.split(separator).collect(),
            };
            if parts.len() >= 4 {
                t_entry_clone.set_text(parts[0]);
                n_entry_clone.set_text(parts[1]);
                m_entry_clone.set_text(parts[2]);
                b_entry_clone.set_text(parts[3]);
            } else if parts.len() == 3 {
                t_entry_clone.set_text(parts[0]);
                n_entry_clone.set_text(parts[1]);
                m_entry_clone.set_text(parts[2]);
                b_entry_clone.set_text("");
            } else if parts.len() == 2 {
                t_entry_clone.set_text(parts[0]);
                n_entry_clone.set_text(parts[1]);
                m_entry_clone.set_text("");
                b_entry_clone.set_text("");
            } else if parts.len() == 1 {
                t_entry_clone.set_text(parts[0]);
                n_entry_clone.set_text("");
                m_entry_clone.set_text("");
                b_entry_clone.set_text("");
            }
        };
        update_ui(entry.text().as_str());

        let t_entry_clone = t_entry.clone();
        let n_entry_clone = n_entry.clone();
        let m_entry_clone = m_entry.clone();
        let b_entry_clone = b_entry.clone();
        let combine_values = move || {
            let t_val = t_entry_clone.text().to_string();
            let n_val = n_entry_clone.text().to_string();
            let m_val = m_entry_clone.text().to_string();
            let b_val = b_entry_clone.text().to_string();
            if separator == PLUG_SEPARATOR {
                format!("{}{}{}{}", t_val, n_val, m_val, b_val)
            } else {
                format!(
                    "{}{}{}{}{}{}{}",
                    t_val, separator, n_val, separator, m_val, separator, b_val
                )
            }
        };

        let combine_values_clone = combine_values.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        t_entry.connect_changed(move |_| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&combine_values_clone());
            is_updating_clone.set(false);
        });

        let combine_values_clone = combine_values.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        n_entry.connect_changed(move |_| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&combine_values_clone());
            is_updating_clone.set(false);
        });

        let combine_values_clone = combine_values.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        m_entry.connect_changed(move |_| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&combine_values_clone());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        b_entry.connect_changed(move |_| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&combine_values());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            update_ui(entry.text().as_str());
            is_updating_clone.set(false);
        });

        mother_box
    }
}

pub fn create_spin_button_builder(
    min: f64,
    max: f64,
    step: f64,
) -> impl Fn(&Entry, &FieldLabel) -> GtkBox {
    move |entry: &Entry, name: &FieldLabel| -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        if let FieldLabel::Named(name) = name
            && name != &"%"
        {
            mother_box.append(&Label::new(Some(name)));
        }
        let spin_button = create_spin_button(min, max, step);
        mother_box.append(&spin_button);
        if let FieldLabel::Named("%") = name {
            mother_box.append(&Label::new(Some("%")));
        }

        let spin_button_clone = spin_button.clone();
        let update_ui = move |value: f64| {
            spin_button_clone.set_value(value);
        };
        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let new_value = spin_button.value();
            entry_clone.set_text(&new_value.to_string());
            is_updating_clone.set(false);
        });

        entry.connect_changed(move |entry| {
            if is_updating.get() {
                return;
            }
            is_updating.set(true);
            update_ui(entry.text().parse().unwrap_or_default());
            is_updating.set(false);
        });

        mother_box
    }
}

impl ToGtkBox for () {
    fn to_gtk_box(_entry: &Entry) -> GtkBox {
        GtkBox::new(GtkOrientation::Horizontal, 5)
    }
}

impl ToGtkBox for String {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let new_entry = create_entry();
        mother_box.append(&new_entry);

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        new_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            entry_clone.set_text(&entry.text());
            is_updating_clone.set(false);
        });

        let new_entry_clone = new_entry.clone();
        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating.set(true);
            new_entry_clone.set_text(&entry.text());
            is_updating.set(false);
        });

        mother_box
    }
}

impl ToGtkBox for u8 {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let spin_button = create_spin_button(0.0, 255.0, 1.0);
        mother_box.append(&spin_button);

        let spin_button_clone = spin_button.clone();
        let update_ui = move |value: u8| {
            spin_button_clone.set_value(value as f64);
        };
        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let new_value = spin_button.value() as u8;
            entry_clone.set_text(&new_value.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating.set(true);
            update_ui(entry.text().parse().unwrap_or_default());
            is_updating.set(false);
        });

        mother_box
    }
}

impl ToGtkBox for u32 {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let spin_button = create_spin_button(0.0, i32::MAX as f64, 1.0);
        mother_box.append(&spin_button);

        let spin_button_clone = spin_button.clone();
        let update_ui = move |value: u32| {
            spin_button_clone.set_value(value as f64);
        };
        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let new_value = spin_button.value() as u32;
            entry_clone.set_text(&new_value.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            update_ui(entry.text().parse().unwrap_or_default());
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl ToGtkBox for i32 {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let spin_button = create_spin_button(i32::MIN as f64, i32::MAX as f64, 1.0);
        mother_box.append(&spin_button);

        let spin_button_clone = spin_button.clone();
        let update_ui = move |value: i32| {
            spin_button_clone.set_value(value as f64);
        };
        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let new_value = spin_button.value() as i32;
            entry_clone.set_text(&new_value.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            update_ui(entry.text().parse().unwrap_or_default());
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl ToGtkBox for bool {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let switch = create_switch();
        mother_box.append(&switch);

        let switch_clone = switch.clone();
        let update_ui = move |value: bool| {
            switch_clone.set_state(value);
        };
        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let new_value = switch.state();
            entry_clone.set_text(&new_value.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            update_ui(entry.text().parse().unwrap_or_default());
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!((), String, u8, u32, i32, bool,);
