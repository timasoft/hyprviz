use crate::{
    advanced_editors::{
        create_button, create_dropdown, create_entry, create_spin_button, create_switch,
    },
    utils::{
        Angle, AnimationStyle, BorderColor, ChangeGroupActive, ContentType, CursorCorner,
        CycleNext, Direction, Dispatcher, DispatcherDiscriminant, DispatcherFullscreenState,
        FloatValue, FullscreenMode, FullscreenState, GroupLockAction, HasDiscriminant, HyprColor,
        HyprCoord, HyprGradient, HyprOpacity, HyprSize, IdOrName, IdleIngibitMode, KeyState,
        MAX_SAFE_STEP_0_01_F64, MIN_SAFE_STEP_0_01_F64, Modifier, MonitorTarget, MoveDirection,
        PixelOrPercent, RelativeId, ResizeParams, SetProp, SetPropToggleState, Side, SizeBound,
        SwapDirection, SwapNext, TagToggleState, ToggleState, WindowEvent, WindowGroupOption,
        WindowRule, WindowTarget, WorkspaceTarget, ZHeight, cow_to_static_str, join_with_separator,
    },
};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, StringList, prelude::*};
use rust_i18n::t;
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
    str::FromStr,
};
use strum::IntoEnumIterator;

const PLUG_SEPARATOR: char = '︲';

#[derive(Debug, Clone, Default)]
pub enum FieldLabel {
    Named(&'static str),
    #[default]
    Unnamed,
}

trait EnumConfigForGtk {
    fn separator() -> Option<char> {
        None
    }

    fn dropdown_items() -> StringList;
    #[allow(clippy::type_complexity)]
    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        None
    }
    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        None
    }
}

pub trait ToGtkBox {
    fn to_gtk_box(entry: &Entry) -> GtkBox;
}
pub struct ToGtkBoxImplementation(pub fn(&Entry) -> GtkBox);
inventory::collect!(ToGtkBoxImplementation);

trait ToOptionalGtkBox {
    fn to_gtk_box(entry: &Entry) -> GtkBox;
}
pub struct ToOptionalGtkBoxImplementation(pub fn(&Entry) -> GtkBox);
inventory::collect!(ToOptionalGtkBoxImplementation);

trait ToGtkBoxWithSeparator {
    fn to_gtk_box(entry: &Entry, separator: char) -> GtkBox;
}
pub struct ToGtkBoxWithSeparatorImplementation(pub fn(&Entry, char) -> GtkBox);
inventory::collect!(ToGtkBoxWithSeparatorImplementation);

trait ToGtkBoxWithSeparatorAndNames {
    fn to_gtk_box(entry: &Entry, separator: char, names: &[FieldLabel]) -> GtkBox;
}
pub struct ToGtkBoxWithSeparatorAndNamesImplementation(
    pub fn(&Entry, char, &[FieldLabel]) -> GtkBox,
);
inventory::collect!(ToGtkBoxWithSeparatorAndNamesImplementation);

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

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let dropdown = create_dropdown(&string_list);
        dropdown.set_selected(0);
        mother_box.append(&dropdown);

        let parameter_entry = create_entry();
        let parameter_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let mut variant_parameter_boxes = HashMap::new();
        if let Some(separator) = T::separator() {
            for (i, discriminant) in T::Discriminant::iter().enumerate() {
                if let Some(builder) = T::from_discriminant(discriminant).parameter_builder() {
                    let empty_vec = Vec::new();
                    let field_labels = T::field_labels().unwrap_or(Vec::new());
                    let labels = field_labels.get(i).unwrap_or(&empty_vec);
                    let variant_parameter_box = builder(&parameter_entry, separator, labels);
                    variant_parameter_box.set_visible(false);

                    parameter_box.append(&variant_parameter_box);
                    variant_parameter_boxes.insert(i, variant_parameter_box);
                }
            }
            mother_box.append(&parameter_box);
        }

        let dropdown_clone = dropdown.clone();
        let parameter_entry_clone = parameter_entry.clone();
        let get_value = move |variant_index: Option<usize>, parameter_text: Option<&str>| {
            let variant_index = match variant_index {
                Some(variant_index) => variant_index,
                None => dropdown_clone.selected() as usize,
            };
            let parameter_text = match parameter_text {
                Some(parameter_text) => parameter_text.to_string(),
                None => parameter_entry_clone.text().to_string(),
            };

            T::from_discriminant_and_str(
                T::Discriminant::iter()
                    .nth(variant_index)
                    .expect("variant_index out of bounds"),
                &parameter_text,
            )
        };

        let dropdown_clone = dropdown.clone();
        let parameter_entry_clone = parameter_entry.clone();
        let update_ui = move |value: T| {
            let variant_index = value.variant_index();
            dropdown_clone.set_selected(variant_index as u32);

            for variant_parameter_box in variant_parameter_boxes.values() {
                variant_parameter_box.set_visible(false);
            }

            if let Some(variant_parameter_box) = variant_parameter_boxes.get(&variant_index) {
                variant_parameter_box.set_visible(true);
            }

            let new_text = value.to_str_without_discriminant().unwrap_or_default();
            parameter_entry_clone.set_text(&new_text);
        };

        update_ui(get_value(None, None));

        let get_value_clone = get_value.clone();
        let update_ui_clone = update_ui.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        parameter_entry.connect_changed(move |parameter_entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let new_value = get_value_clone(None, Some(parameter_entry.text().as_str()));
            entry_clone.set_text(&new_value.to_string());
            update_ui_clone(new_value);
            is_updating_clone.set(false);
        });

        let get_value_clone = get_value.clone();
        let update_ui_clone = update_ui.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        dropdown.connect_selected_notify(move |dropdown| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let new_value = get_value_clone(Some(dropdown.selected() as usize), None);
            entry_clone.set_text(&new_value.to_string());
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

impl<T: ToGtkBox> ToOptionalGtkBox for Option<T> {
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

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let add_button = create_button(&t!("add"));

        let mother_box_clone = mother_box.clone();
        let add_button_clone = add_button.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        let rebuild_ui = move |text: &str| {
            while let Some(child) = mother_box_clone.first_child() {
                mother_box_clone.remove(&child);
            }

            let mut remove_buttons = Vec::new();

            let parts = text
                .split(separator)
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>();
            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                let part_box = GtkBox::new(GtkOrientation::Vertical, 5);

                let remove_button = create_button(&t!("remove"));
                part_box.append(&remove_button);
                remove_buttons.push(remove_button.clone());

                let sub_entry = create_entry();
                let sub_box = T::to_gtk_box(&sub_entry);
                sub_entry.set_text(part);
                part_box.append(&sub_box);

                let entry_clone_clone = entry_clone.clone();
                let is_updating_clone_clone = is_updating_clone.clone();
                sub_entry.connect_changed(move |entry| {
                    if is_updating_clone_clone.get() {
                        return;
                    }
                    is_updating_clone_clone.set(true);
                    let new_text = entry.text().to_string();
                    let entry_text = entry_clone_clone.text().to_string();
                    let mut parts_vec: Vec<String> = entry_text
                        .split(separator)
                        .filter(|p| !p.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    if i < parts_vec.len() {
                        parts_vec[i] = new_text;
                        let separator = match separator {
                            ';' => "; ".to_string(),
                            char => char.to_string(),
                        };
                        let updated_text = parts_vec.join(&separator);
                        entry_clone_clone.set_text(&updated_text);
                    }
                    is_updating_clone_clone.set(false);
                });

                mother_box_clone.append(&part_box);
            }
            mother_box_clone.append(&add_button_clone);
            remove_buttons
        };

        let entry_clone = entry.clone();
        add_button.connect_clicked(move |_| {
            let text = entry_clone.text().to_string();
            let mut parts = text.split(separator).collect::<Vec<_>>();
            let new_text = T::default().to_string();
            parts.push(&new_text);
            let separator = match separator {
                ';' => "; ".to_string(),
                char => char.to_string(),
            };
            let updated_text = parts.join(&separator);
            entry_clone.set_text(&updated_text);
        });

        let rebuild_ui_with_remove_buttons = move |entry: &Entry| {
            let remove_buttons = rebuild_ui(entry.text().as_str());
            for (i, remove_button) in remove_buttons.into_iter().enumerate() {
                let entry_clone = entry.clone();
                remove_button.connect_clicked(move |_| {
                    let text = entry_clone.text().to_string();
                    let mut parts = text
                        .split(separator)
                        .filter(|p| !p.is_empty())
                        .collect::<Vec<_>>();
                    parts.remove(i);
                    let separator = match separator {
                        ';' => "; ".to_string(),
                        char => char.to_string(),
                    };
                    let updated_text = parts.join(&separator);
                    entry_clone.set_text(&updated_text);
                });
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

impl<T: ToGtkBox + Default + Display + FromStr> ToGtkBoxWithSeparator for HashSet<T> {
    fn to_gtk_box(entry: &Entry, separator: char) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let add_button = create_button(&t!("add"));

        let mother_box_clone = mother_box.clone();
        let add_button_clone = add_button.clone();
        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        let rebuild_ui = move |text: &str| {
            while let Some(child) = mother_box_clone.first_child() {
                mother_box_clone.remove(&child);
            }

            let mut remove_buttons = Vec::new();

            let parts = text
                .split(separator)
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>();
            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                let part_box = GtkBox::new(GtkOrientation::Vertical, 5);

                let remove_button = create_button(&t!("remove"));
                part_box.append(&remove_button);
                remove_buttons.push(remove_button.clone());

                let sub_entry = create_entry();
                let sub_box = T::to_gtk_box(&sub_entry);
                part_box.append(&sub_box);

                let entry_clone_clone = entry_clone.clone();
                let is_updating_clone_clone = is_updating_clone.clone();
                sub_entry.connect_changed(move |entry| {
                    if is_updating_clone_clone.get() {
                        return;
                    }
                    is_updating_clone_clone.set(true);
                    let new_text = entry.text().to_string();
                    let entry_text = entry_clone_clone.text().to_string();
                    let mut parts_vec: Vec<String> = entry_text
                        .split(separator)
                        .filter(|p| !p.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    if i < parts_vec.len() {
                        parts_vec[i] = new_text;
                        let updated_text = parts_vec.join(&separator.to_string());
                        entry_clone_clone.set_text(&updated_text);
                    }
                    is_updating_clone_clone.set(false);
                });

                mother_box_clone.append(&part_box);
            }
            mother_box_clone.append(&add_button_clone);
            remove_buttons
        };

        let entry_clone = entry.clone();
        add_button.connect_clicked(move |_| {
            let text = entry_clone.text().to_string();
            let mut parts = text.split(separator).collect::<Vec<_>>();
            let new_text = T::default().to_string();
            parts.push(&new_text);
            let updated_text = parts.join(&separator.to_string());
            entry_clone.set_text(&updated_text);
        });

        let rebuild_ui_with_remove_buttons = move |entry: &Entry| {
            let remove_buttons = rebuild_ui(entry.text().as_str());
            for (i, remove_button) in remove_buttons.into_iter().enumerate() {
                let entry_clone = entry.clone();
                remove_button.connect_clicked(move |_| {
                    let text = entry_clone.text().to_string();
                    let mut parts = text
                        .split(separator)
                        .filter(|p| !p.is_empty())
                        .collect::<Vec<_>>();
                    parts.remove(i);
                    let updated_text = parts.join(&separator.to_string());
                    entry_clone.set_text(&updated_text);
                });
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
    fn to_gtk_box(entry: &Entry, _separator: char, names: &[FieldLabel]) -> GtkBox {
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
    fn to_gtk_box(entry: &Entry, separator: char, names: &[FieldLabel]) -> GtkBox {
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
            let parts: Vec<&str> = text.split(separator).collect();
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
    fn to_gtk_box(entry: &Entry, separator: char, names: &[FieldLabel]) -> GtkBox {
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
            let parts: Vec<&str> = text.split(separator).collect();
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
    fn to_gtk_box(entry: &Entry, separator: char, names: &[FieldLabel]) -> GtkBox {
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
            let parts: Vec<&str> = text.split(separator).collect();
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
            mother_box.append(&Label::new(Some(&t!("%"))));
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

impl EnumConfigForGtk for Direction {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("left"), &t!("right"), &t!("up"), &t!("down")])
    }
}

impl EnumConfigForGtk for MonitorTarget {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("direction"),
            &t!("id"),
            &t!("name"),
            &t!("current"),
            &t!("relative"),
        ])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            MonitorTarget::Direction(_direction) => Some(<(Direction,)>::to_gtk_box),
            MonitorTarget::Id(_id) => Some(<(u32,)>::to_gtk_box),
            MonitorTarget::Name(_name) => Some(<(String,)>::to_gtk_box),
            MonitorTarget::Current => None,
            MonitorTarget::Relative(_relative) => Some(<(i32,)>::to_gtk_box),
        }
    }
}

impl EnumConfigForGtk for PixelOrPercent {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("pixel"), &t!("percent")])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            PixelOrPercent::Pixel(_foo) => Some(<(i32,)>::to_gtk_box),
            PixelOrPercent::Percent(_foo) => Some(|entry, _, names| {
                create_spin_button_builder(0.0, 100.0, 0.1)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
        }
    }

    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        Some(vec![
            vec![FieldLabel::Unnamed],
            vec![FieldLabel::Named("%")],
        ])
    }
}

impl EnumConfigForGtk for ResizeParams {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("relative"), &t!("exact")])
    }

    fn separator() -> Option<char> {
        Some(' ')
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            ResizeParams::Relative(_x, _y) => Some(<(PixelOrPercent, PixelOrPercent)>::to_gtk_box),
            ResizeParams::Exact(_x, _y) => Some(<(PixelOrPercent, PixelOrPercent)>::to_gtk_box),
        }
    }
}

impl EnumConfigForGtk for FloatValue {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("relative"), &t!("exact")])
    }

    fn separator() -> Option<char> {
        Some(' ')
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            FloatValue::Relative(_f) => Some(|entry, _, names| {
                create_spin_button_builder(MIN_SAFE_STEP_0_01_F64, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            FloatValue::Exact(_f) => Some(|entry, _, names| {
                create_spin_button_builder(MIN_SAFE_STEP_0_01_F64, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
        }
    }
}

impl EnumConfigForGtk for ZHeight {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("top"), &t!("bottom")])
    }
}

impl EnumConfigForGtk for FullscreenMode {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("fullscreen"), &t!("maximize")])
    }
}

impl EnumConfigForGtk for RelativeId {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("absolute"), &t!("previous"), &t!("next")])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            RelativeId::Absolute(_i) => Some(<(u32,)>::to_gtk_box),
            RelativeId::Previous(_i) => Some(<(u32,)>::to_gtk_box),
            RelativeId::Next(_i) => Some(<(u32,)>::to_gtk_box),
        }
    }
}

impl EnumConfigForGtk for WorkspaceTarget {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("id"),
            &t!("relative"),
            &t!("on_monitor"),
            &t!("on_monitor_including_empty_workspace"),
            &t!("open"),
            &t!("name"),
            &t!("previous"),
            &t!("previous_per_monitor"),
            &t!("first_available_empty_workspace"),
            &t!("next_available_empty_workspace"),
            &t!("first_available_empty_workspace_on_monitor"),
            &t!("next_available_empty_workspace_on_monitor"),
            &t!("special"),
            &t!("special_with_name"),
        ])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            WorkspaceTarget::Id(_id) => Some(<(u32,)>::to_gtk_box),
            WorkspaceTarget::Relative(_relative) => Some(<(i32,)>::to_gtk_box),
            WorkspaceTarget::OnMonitor(_rel_id) => Some(<(RelativeId,)>::to_gtk_box),
            WorkspaceTarget::OnMonitorIncludingEmptyWorkspace(_rel_id) => {
                Some(<(RelativeId,)>::to_gtk_box)
            }
            WorkspaceTarget::Open(_rel_id) => Some(<(bool,)>::to_gtk_box),
            WorkspaceTarget::Name(_name) => Some(<(String,)>::to_gtk_box),
            WorkspaceTarget::Previous => None,
            WorkspaceTarget::PreviousPerMonitor => None,
            WorkspaceTarget::FirstAvailableEmptyWorkspace => None,
            WorkspaceTarget::NextAvailableEmptyWorkspace => None,
            WorkspaceTarget::FirstAvailableEmptyWorkspaceOnMonitor => None,
            WorkspaceTarget::NextAvailableEmptyWorkspaceOnMonitor => None,
            WorkspaceTarget::Special => None,
            WorkspaceTarget::SpecialWithName(_name) => Some(<(String,)>::to_gtk_box),
        }
    }
}

impl EnumConfigForGtk for WindowTarget {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("class"),
            &t!("initial_class"),
            &t!("title"),
            &t!("initial_title"),
            &t!("tag"),
            &t!("pid"),
            &t!("address"),
            &t!("active_window"),
            &t!("floating"),
            &t!("tiled"),
        ])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            WindowTarget::Class(_class) => Some(<(String,)>::to_gtk_box),
            WindowTarget::InitialClass(_class) => Some(<(String,)>::to_gtk_box),
            WindowTarget::Title(_title) => Some(<(String,)>::to_gtk_box),
            WindowTarget::InitialTitle(_title) => Some(<(String,)>::to_gtk_box),
            WindowTarget::Tag(_tag) => Some(<(String,)>::to_gtk_box),
            WindowTarget::Pid(_pid) => Some(<(String,)>::to_gtk_box),
            WindowTarget::Address(_address) => Some(<(String,)>::to_gtk_box),
            WindowTarget::ActiveWindow => None,
            WindowTarget::Floating => None,
            WindowTarget::Tiled => None,
        }
    }
}

impl EnumConfigForGtk for CursorCorner {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("top_left"),
            &t!("top_right"),
            &t!("bottom_left"),
            &t!("bottom_right"),
        ])
    }
}

impl EnumConfigForGtk for GroupLockAction {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("lock"), &t!("unlock"), &t!("toggle")])
    }
}

impl EnumConfigForGtk for ToggleState {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("on"), &t!("off"), &t!("toggle")])
    }
}

impl EnumConfigForGtk for FullscreenState {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("none"),
            &t!("maximize"),
            &t!("fullscreen"),
            &t!("maximize_and_fullscreen"),
        ])
    }
}

impl ToGtkBox for HyprCoord {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let x_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        x_box_box.append(&Label::new(Some("X")));
        let x_entry = create_entry();
        let x_box = PixelOrPercent::to_gtk_box(&x_entry);
        x_box_box.append(&x_box);
        mother_box.append(&x_box_box);

        let y_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        y_box_box.append(&Label::new(Some("Y")));
        let y_entry = create_entry();
        let y_box = PixelOrPercent::to_gtk_box(&y_entry);
        y_box_box.append(&y_box);
        mother_box.append(&y_box_box);

        let x_sub_box = GtkBox::new(GtkOrientation::Vertical, 5);
        x_sub_box.append(&Label::new(Some(&t!("subtrahend_of_x"))));
        let x_sub_spin_button = create_spin_button(0.0, i32::MAX as f64, 1.0);
        x_sub_box.append(&x_sub_spin_button);
        mother_box.append(&x_sub_box);

        let y_sub_box = GtkBox::new(GtkOrientation::Vertical, 5);
        y_sub_box.append(&Label::new(Some(&t!("subtrahend_of_y"))));
        let y_sub_spin_button = create_spin_button(0.0, i32::MAX as f64, 1.0);
        y_sub_box.append(&y_sub_spin_button);
        mother_box.append(&y_sub_box);

        let under_cursor_box = GtkBox::new(GtkOrientation::Vertical, 5);
        under_cursor_box.append(&Label::new(Some(&t!("under_cursor"))));
        let under_cursor_switch = create_switch();
        under_cursor_box.append(&under_cursor_switch);
        mother_box.append(&under_cursor_box);

        let on_screen_box = GtkBox::new(GtkOrientation::Vertical, 5);
        on_screen_box.append(&Label::new(Some(&t!("on_screen"))));
        let on_screen_switch = create_switch();
        on_screen_box.append(&on_screen_switch);
        mother_box.append(&on_screen_box);

        let x_entry_clone = x_entry.clone();
        let y_entry_clone = y_entry.clone();
        let x_sub_spin_button_clone = x_sub_spin_button.clone();
        let y_sub_spin_button_clone = y_sub_spin_button.clone();
        let under_cursor_switch_clone = under_cursor_switch.clone();
        let on_screen_switch_clone = on_screen_switch.clone();
        let update_ui = move |hypr_coord: HyprCoord| {
            x_entry_clone.set_text(&hypr_coord.x.to_string());
            y_entry_clone.set_text(&hypr_coord.y.to_string());
            x_sub_spin_button_clone.set_value(hypr_coord.x_sub as f64);
            y_sub_spin_button_clone.set_value(hypr_coord.y_sub as f64);
            under_cursor_switch_clone.set_active(hypr_coord.under_cursor);
            on_screen_switch_clone.set_active(hypr_coord.on_screen);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        x_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.x = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        y_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.y = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        x_sub_spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.x_sub = spin_button.value() as u32;
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        y_sub_spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.y_sub = spin_button.value() as u32;
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        under_cursor_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.under_cursor = switch.state();
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        on_screen_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_coord: HyprCoord = entry_clone.text().parse().unwrap_or_default();
            hypr_coord.on_screen = switch.state();
            entry_clone.set_text(&hypr_coord.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let hypr_coord = entry.text().parse().unwrap_or_default();
            update_ui(hypr_coord);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl ToGtkBox for HyprSize {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let width_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        width_box_box.append(&Label::new(Some(&t!("width"))));
        let width_entry = create_entry();
        let width_box = PixelOrPercent::to_gtk_box(&width_entry);
        width_box_box.append(&width_box);
        mother_box.append(&width_box_box);

        let height_box_box = GtkBox::new(GtkOrientation::Vertical, 5);
        height_box_box.append(&Label::new(Some(&t!("height"))));
        let height_entry = create_entry();
        let height_box = PixelOrPercent::to_gtk_box(&height_entry);
        height_box_box.append(&height_box);
        mother_box.append(&height_box_box);

        let size_bound_string_list = StringList::new(&[&t!("exact"), &t!("max"), &t!("min")]);

        let width_bound_box = GtkBox::new(GtkOrientation::Vertical, 5);
        width_bound_box.append(&Label::new(Some(&t!("width_bound"))));
        let width_bound_dropdown = create_dropdown(&size_bound_string_list);
        width_bound_box.append(&width_bound_dropdown);
        mother_box.append(&width_bound_box);

        let height_bound_box = GtkBox::new(GtkOrientation::Vertical, 5);
        height_bound_box.append(&Label::new(Some(&t!("height_bound"))));
        let height_bound_dropdown = create_dropdown(&size_bound_string_list);
        height_bound_box.append(&height_bound_dropdown);
        mother_box.append(&height_bound_box);

        let width_entry_clone = width_entry.clone();
        let height_entry_clone = height_entry.clone();
        let width_bound_dropdown_clone = width_bound_dropdown.clone();
        let height_bound_dropdown_clone = height_bound_dropdown.clone();
        let update_ui = move |hypr_size: HyprSize| {
            width_entry_clone.set_text(&hypr_size.width.to_string());
            height_entry_clone.set_text(&hypr_size.height.to_string());
            width_bound_dropdown_clone.set_selected(match hypr_size.width_bound {
                SizeBound::Exact => 0,
                SizeBound::Max => 1,
                SizeBound::Min => 2,
            });
            height_bound_dropdown_clone.set_selected(match hypr_size.height_bound {
                SizeBound::Exact => 0,
                SizeBound::Max => 1,
                SizeBound::Min => 2,
            });
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        width_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_size: HyprSize = entry_clone.text().parse().unwrap_or_default();
            hypr_size.width = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&hypr_size.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        height_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_size: HyprSize = entry_clone.text().parse().unwrap_or_default();
            hypr_size.height = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&hypr_size.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        width_bound_dropdown.connect_selected_notify(move |dropdown| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_size: HyprSize = entry_clone.text().parse().unwrap_or_default();
            hypr_size.width_bound = match dropdown.selected() {
                0 => SizeBound::Exact,
                1 => SizeBound::Max,
                2 => SizeBound::Min,
                _ => unreachable!(),
            };
            entry_clone.set_text(&hypr_size.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        height_bound_dropdown.connect_selected_notify(move |dropdown| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut hypr_size: HyprSize = entry_clone.text().parse().unwrap_or_default();
            hypr_size.height_bound = match dropdown.selected() {
                0 => SizeBound::Exact,
                1 => SizeBound::Max,
                2 => SizeBound::Min,
                _ => unreachable!(),
            };
            entry_clone.set_text(&hypr_size.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let hypr_size = entry.text().parse().unwrap_or_default();
            update_ui(hypr_size);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl EnumConfigForGtk for IdOrName {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("id"), &t!("name")])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            IdOrName::Id(_id) => Some(<(u32,)>::to_gtk_box),
            IdOrName::Name(_name) => Some(<(String,)>::to_gtk_box),
        }
    }
}

impl EnumConfigForGtk for WindowGroupOption {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("set"),
            &t!("set_always"),
            &t!("new"),
            &t!("lock"),
            &t!("lock_always"),
            &t!("barred"),
            &t!("deny"),
            &t!("invade"),
            &t!("override"),
            &t!("unset"),
        ])
    }
}

impl EnumConfigForGtk for WindowEvent {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("fullscreen"),
            &t!("maximize"),
            &t!("activate"),
            &t!("activatefocus"),
            &t!("fullscreenoutput"),
        ])
    }
}

impl EnumConfigForGtk for ContentType {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("none"), &t!("photo"), &t!("video"), &t!("game")])
    }
}

impl EnumConfigForGtk for HyprColor {
    fn dropdown_items() -> StringList {
        StringList::new(&["RGB", "RGBA"])
    }

    fn separator() -> Option<char> {
        Some(',')
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            HyprColor::Rgb(_r, _g, _b) => Some(<(u8, u8, u8)>::to_gtk_box),
            HyprColor::Rgba(_r, _g, _b, _a) => Some(<(u8, u8, u8, u8)>::to_gtk_box),
        }
    }

    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        Some(vec![
            vec![
                FieldLabel::Named(cow_to_static_str(t!("red"))),
                FieldLabel::Named(cow_to_static_str(t!("green"))),
                FieldLabel::Named(cow_to_static_str(t!("blue"))),
            ],
            vec![
                FieldLabel::Named(cow_to_static_str(t!("red"))),
                FieldLabel::Named(cow_to_static_str(t!("green"))),
                FieldLabel::Named(cow_to_static_str(t!("blue"))),
                FieldLabel::Named(cow_to_static_str(t!("alpha"))),
            ],
        ])
    }
}

impl ToGtkBox for Angle {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let degrees_spin_button = create_spin_button(0.0, 360.0, 1.0);
        mother_box.append(&degrees_spin_button);

        let degrees_spin_button_clone = degrees_spin_button.clone();
        let update_ui = move |angle: Angle| match angle {
            Angle::Degrees(degrees) => {
                degrees_spin_button_clone.set_value(degrees as f64);
            }
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        degrees_spin_button.connect_value_changed(move |spin_button| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let angle = Angle::Degrees(spin_button.value() as u16);
            entry_clone.set_text(&angle.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let angle = entry.text().parse().unwrap_or_default();
            update_ui(angle);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl ToGtkBox for BorderColor {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));

        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let border_color_string_list = StringList::new(&[
            &t!("active_border_color"),
            &t!("active_border_gradient"),
            &t!("active_and_inactive_border_color"),
            &t!("active_and_inactive_border_gradient"),
        ]);
        let border_color_dropdown = create_dropdown(&border_color_string_list);
        border_color_dropdown.set_selected(0);
        mother_box.append(&border_color_dropdown);

        let hypr_color_entry = create_entry();
        let hypr_color_box = HyprColor::to_gtk_box(&hypr_color_entry);
        hypr_color_box.set_visible(false);
        mother_box.append(&hypr_color_box);

        let second_hypr_color_entry = create_entry();
        let second_hypr_color_box = HyprColor::to_gtk_box(&second_hypr_color_entry);
        second_hypr_color_box.set_visible(false);
        mother_box.append(&second_hypr_color_box);

        let gradient_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let vec_hypr_color_entry = create_entry();
        let vec_hypr_color_box =
            Vec::<HyprColor>::to_gtk_box(&vec_hypr_color_entry, Self::SEPARATOR);
        gradient_box.append(&vec_hypr_color_box);
        let angle_entry = create_entry();
        let angle_box = Angle::to_gtk_box(&angle_entry);
        gradient_box.append(&angle_box);
        mother_box.append(&gradient_box);

        let second_gradient_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let second_vec_hypr_color_entry = create_entry();
        let second_vec_hypr_color_box =
            Vec::<HyprColor>::to_gtk_box(&second_vec_hypr_color_entry, Self::SEPARATOR);
        second_gradient_box.append(&second_vec_hypr_color_box);
        let opt_angle_entry = create_entry();
        let opt_angle_box = Option::<Angle>::to_gtk_box(&opt_angle_entry);
        second_gradient_box.append(&opt_angle_box);
        mother_box.append(&second_gradient_box);

        let border_color_dropdown_clone = border_color_dropdown.clone();
        let hypr_color_entry_clone = hypr_color_entry.clone();
        let hypr_color_box_clone = hypr_color_box.clone();
        let second_hypr_color_entry_clone = second_hypr_color_entry.clone();
        let second_hypr_color_box_clone = second_hypr_color_box.clone();
        let vec_hypr_color_entry_clone = vec_hypr_color_entry.clone();
        let vec_hypr_color_box_clone = vec_hypr_color_box.clone();
        let angle_entry_clone = angle_entry.clone();
        let angle_box_clone = angle_box.clone();
        let second_vec_hypr_color_entry_clone = second_vec_hypr_color_entry.clone();
        let second_vec_hypr_color_box_clone = second_vec_hypr_color_box.clone();
        let opt_angle_entry_clone = opt_angle_entry.clone();
        let opt_angle_box_clone = opt_angle_box.clone();
        let update_ui = move |border_color: BorderColor| match border_color {
            BorderColor::Color(color) => {
                border_color_dropdown_clone.set_selected(0);
                hypr_color_entry_clone.set_text(&color.to_string());

                hypr_color_box_clone.set_visible(true);
                second_hypr_color_box_clone.set_visible(false);
                vec_hypr_color_box_clone.set_visible(false);
                second_vec_hypr_color_box_clone.set_visible(false);
                angle_box_clone.set_visible(false);
                opt_angle_box_clone.set_visible(false);
            }
            BorderColor::Gradient(colors, angle) => {
                border_color_dropdown_clone.set_selected(1);
                vec_hypr_color_entry_clone
                    .set_text(&join_with_separator(colors, &Self::SEPARATOR.to_string()));
                angle_entry_clone.set_text(&angle.to_string());

                hypr_color_box_clone.set_visible(false);
                second_hypr_color_box_clone.set_visible(false);
                vec_hypr_color_box_clone.set_visible(true);
                second_vec_hypr_color_box_clone.set_visible(false);
                angle_box_clone.set_visible(true);
                opt_angle_box_clone.set_visible(false);
            }
            BorderColor::DoubleColor(color1, color2) => {
                border_color_dropdown_clone.set_selected(2);
                hypr_color_entry_clone.set_text(&color1.to_string());
                second_hypr_color_entry_clone.set_text(&color2.to_string());

                hypr_color_box_clone.set_visible(true);
                second_hypr_color_box_clone.set_visible(true);
                vec_hypr_color_box_clone.set_visible(false);
                second_vec_hypr_color_box_clone.set_visible(false);
                angle_box_clone.set_visible(false);
                opt_angle_box_clone.set_visible(false);
            }
            BorderColor::DoubleGradient(colors1, angle1, colors2, angle2) => {
                border_color_dropdown_clone.set_selected(3);
                vec_hypr_color_entry_clone
                    .set_text(&join_with_separator(colors1, &Self::SEPARATOR.to_string()));
                angle_entry_clone.set_text(&angle1.to_string());
                second_vec_hypr_color_entry_clone
                    .set_text(&join_with_separator(colors2, &Self::SEPARATOR.to_string()));
                opt_angle_entry_clone.set_text(&angle2.unwrap_or_default().to_string());

                hypr_color_box_clone.set_visible(false);
                second_hypr_color_box_clone.set_visible(false);
                vec_hypr_color_box_clone.set_visible(true);
                second_vec_hypr_color_box_clone.set_visible(true);
                angle_box_clone.set_visible(true);
                opt_angle_box_clone.set_visible(true);
            }
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let hypr_color_entry_clone = hypr_color_entry.clone();
        let second_hypr_color_entry_clone = second_hypr_color_entry.clone();
        let vec_hypr_color_entry_clone = vec_hypr_color_entry.clone();
        let second_vec_hypr_color_entry_clone = second_vec_hypr_color_entry.clone();
        let angle_entry_clone = angle_entry.clone();
        let opt_angle_entry_clone = opt_angle_entry.clone();
        let is_updating_clone = is_updating.clone();
        border_color_dropdown.connect_selected_notify(move |dropdown| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let border_color = match dropdown.selected() {
                0 => BorderColor::Color(hypr_color_entry_clone.text().parse().unwrap_or_default()),
                1 => BorderColor::Gradient(
                    vec_hypr_color_entry_clone
                        .text()
                        .split(Self::SEPARATOR)
                        .map(|s| s.parse().unwrap_or_default())
                        .collect(),
                    angle_entry_clone.text().parse().unwrap_or_default(),
                ),
                2 => BorderColor::DoubleColor(
                    hypr_color_entry_clone.text().parse().unwrap_or_default(),
                    second_hypr_color_entry_clone
                        .text()
                        .parse()
                        .unwrap_or_default(),
                ),
                3 => BorderColor::DoubleGradient(
                    vec_hypr_color_entry_clone
                        .text()
                        .split(Self::SEPARATOR)
                        .map(|s| s.parse().unwrap_or_default())
                        .collect(),
                    angle_entry_clone.text().parse().unwrap_or_default(),
                    second_vec_hypr_color_entry_clone
                        .text()
                        .split(Self::SEPARATOR)
                        .map(|s| s.parse().unwrap_or_default())
                        .collect(),
                    match opt_angle_entry_clone.text().as_str() {
                        "" => None,
                        s => Some(s.parse().unwrap_or_default()),
                    },
                ),
                _ => unreachable!(),
            };

            entry_clone.set_text(&border_color.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        hypr_color_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let color = entry.text().parse().unwrap_or_default();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            match border_color {
                BorderColor::Color(_) => {
                    entry_clone.set_text(&BorderColor::Color(color).to_string());
                }
                BorderColor::DoubleColor(_, color2) => {
                    entry_clone.set_text(&BorderColor::DoubleColor(color, color2).to_string());
                }
                _ => {}
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        second_hypr_color_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let color = entry.text().parse().unwrap_or_default();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            if let BorderColor::DoubleColor(color1, _) = border_color {
                entry_clone.set_text(&BorderColor::DoubleColor(color1, color).to_string());
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        vec_hypr_color_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let colors: Vec<HyprColor> = entry
                .text()
                .split(Self::SEPARATOR)
                .map(|s| s.parse().unwrap_or_default())
                .collect();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            match border_color {
                BorderColor::Gradient(_, angle) => {
                    entry_clone.set_text(&BorderColor::Gradient(colors, angle).to_string());
                }
                BorderColor::DoubleGradient(_, angle1, colors2, angle2) => {
                    entry_clone.set_text(
                        &BorderColor::DoubleGradient(colors, angle1, colors2, angle2).to_string(),
                    );
                }
                _ => {}
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        second_vec_hypr_color_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let colors: Vec<HyprColor> = entry
                .text()
                .split(Self::SEPARATOR)
                .map(|s| s.parse().unwrap_or_default())
                .collect();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            if let BorderColor::DoubleGradient(colors1, angle1, _, angle2) = border_color {
                entry_clone.set_text(
                    &BorderColor::DoubleGradient(colors1, angle1, colors, angle2).to_string(),
                );
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        angle_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let angle = entry.text().parse().unwrap_or_default();
            let border_color = entry_clone.text().parse().unwrap_or_default();

            match border_color {
                BorderColor::Gradient(colors, _) => {
                    entry_clone.set_text(&BorderColor::Gradient(colors, angle).to_string());
                }
                BorderColor::DoubleGradient(colors1, _, colors2, angle2) => {
                    entry_clone.set_text(
                        &BorderColor::DoubleGradient(colors1, angle, colors2, angle2).to_string(),
                    );
                }
                _ => {}
            }
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        opt_angle_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let angle = match entry.text().as_str() {
                "" => None,
                s => Some(s.parse().unwrap_or_default()),
            };
            let border_color = entry_clone.text().parse().unwrap_or_default();

            if let BorderColor::DoubleGradient(colors1, angle1, colors2, _) = border_color {
                entry_clone.set_text(
                    &BorderColor::DoubleGradient(colors1, angle1, colors2, angle).to_string(),
                );
            };
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

impl EnumConfigForGtk for IdleIngibitMode {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("none"), &t!("always"), &t!("focus"), &t!("fullscreen")])
    }
}

impl EnumConfigForGtk for HyprOpacity {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("overall"),
            &t!("active and inactive"),
            &t!("active and inactive and fullscreen"),
        ])
    }

    fn separator() -> Option<char> {
        Some(' ')
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            HyprOpacity::Overall(_opacity, _override) => Some(|entry, _separator, _labels| {
                let is_updating = Rc::new(Cell::new(false));
                let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                let opacity_spin_button = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                mother_box.append(&opacity_spin_button);

                let override_box = GtkBox::new(GtkOrientation::Vertical, 5);
                override_box.append(&Label::new(Some(&t!("override"))));
                let override_switch = create_switch();
                override_box.append(&override_switch);
                mother_box.append(&override_box);

                let opacity_spin_button_clone = opacity_spin_button.clone();
                let override_switch_clone = override_switch.clone();
                let update_ui = move |(opacity, override1): (f64, bool)| {
                    opacity_spin_button_clone.set_value(opacity);
                    override_switch_clone.set_state(override1);
                };

                let parse_value = |str: &str| {
                    let hypr_opacity = str.parse().unwrap_or_default();
                    match hypr_opacity {
                        HyprOpacity::Overall(opacity, override1) => (opacity, override1),
                        HyprOpacity::ActiveAndInactive(opacity, override1, _, _) => {
                            (opacity, override1)
                        }
                        HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity,
                            override1,
                            _,
                            _,
                            _,
                            _,
                        ) => (opacity, override1),
                    }
                };

                update_ui(parse_value(entry.text().as_str()));

                let override_switch_clone = override_switch.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                opacity_spin_button.connect_value_changed(move |spin_button| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let opacity = spin_button.value();
                    let override1 = override_switch_clone.state();
                    entry_clone.set_text(&HyprOpacity::Overall(opacity, override1).to_string());
                    is_updating_clone.set(false);
                });

                let opavity_spin_button_clone = opacity_spin_button.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                override_switch.connect_state_notify(move |switch| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let opacity = opavity_spin_button_clone.value();
                    let override1 = switch.state();
                    entry_clone.set_text(&HyprOpacity::Overall(opacity, override1).to_string());
                    is_updating_clone.set(false);
                });

                let is_updating_clone = is_updating.clone();
                entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    update_ui(parse_value(entry.text().as_str()));
                    is_updating_clone.set(false);
                });

                mother_box
            }),
            HyprOpacity::ActiveAndInactive(_opacity1, _override1, _opacity2, _override2) => {
                Some(|entry, _separator, _labels| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let opacity1_spin_button =
                        create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                    mother_box.append(&opacity1_spin_button);

                    let override1_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    override1_box.append(&Label::new(Some(&t!("override"))));
                    let override1_switch = create_switch();
                    override1_box.append(&override1_switch);
                    mother_box.append(&override1_box);

                    let opacity2_spin_button =
                        create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                    mother_box.append(&opacity2_spin_button);

                    let override2_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    override2_box.append(&Label::new(Some(&t!("override"))));
                    let override2_switch = create_switch();
                    override2_box.append(&override2_switch);
                    mother_box.append(&override2_box);

                    let opacity1_spin_button_clone = opacity1_spin_button.clone();
                    let override1_switch_clone = override1_switch.clone();
                    let opacity2_spin_button_clone = opacity2_spin_button.clone();
                    let override2_switch_clone = override2_switch.clone();
                    let update_ui = move |(opacity1, override1, opacity2, override2): (
                        f64,
                        bool,
                        f64,
                        bool,
                    )| {
                        opacity1_spin_button_clone.set_value(opacity1);
                        override1_switch_clone.set_state(override1);
                        opacity2_spin_button_clone.set_value(opacity2);
                        override2_switch_clone.set_state(override2);
                    };

                    let parse_value = |str: &str| {
                        let hypr_opacity = str.parse().unwrap_or_default();
                        match hypr_opacity {
                            HyprOpacity::ActiveAndInactive(
                                opacity1,
                                override1,
                                opacity2,
                                override2,
                            ) => (opacity1, override1, opacity2, override2),
                            HyprOpacity::Overall(opacity, override1) => {
                                (opacity, override1, 1.0, false)
                            }
                            HyprOpacity::ActiveAndInactiveAndFullscreen(
                                opacity1,
                                override1,
                                opacity2,
                                override2,
                                _,
                                _,
                            ) => (opacity1, override1, opacity2, override2),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let override1_switch_clone = override1_switch.clone();
                    let override2_switch_clone = override2_switch.clone();
                    let opacity2_spin_button_clone = opacity2_spin_button.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    opacity1_spin_button.connect_value_changed(move |spin_button| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let opacity1 = spin_button.value();
                        let override1 = override1_switch_clone.state();
                        let opacity2 = opacity2_spin_button_clone.value();
                        let override2 = override2_switch_clone.state();
                        entry_clone.set_text(
                            &HyprOpacity::ActiveAndInactive(
                                opacity1, override1, opacity2, override2,
                            )
                            .to_string(),
                        );
                        is_updating_clone.set(false);
                    });

                    let opacity1_spin_button_clone = opacity1_spin_button.clone();
                    let override2_switch_clone = override2_switch.clone();
                    let opacity2_spin_button_clone = opacity2_spin_button.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    override1_switch.connect_state_notify(move |switch| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let opacity1 = opacity1_spin_button_clone.value();
                        let override1 = switch.state();
                        let opacity2 = opacity2_spin_button_clone.value();
                        let override2 = override2_switch_clone.state();
                        entry_clone.set_text(
                            &HyprOpacity::ActiveAndInactive(
                                opacity1, override1, opacity2, override2,
                            )
                            .to_string(),
                        );
                        is_updating_clone.set(false);
                    });

                    let override1_switch_clone = override1_switch.clone();
                    let override2_switch_clone = override2_switch.clone();
                    let opacity1_spin_button_clone = opacity1_spin_button.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    opacity2_spin_button.connect_value_changed(move |spin_button| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let opacity1 = opacity1_spin_button_clone.value();
                        let override1 = override1_switch_clone.state();
                        let opacity2 = spin_button.value();
                        let override2 = override2_switch_clone.state();
                        entry_clone.set_text(
                            &HyprOpacity::ActiveAndInactive(
                                opacity1, override1, opacity2, override2,
                            )
                            .to_string(),
                        );
                        is_updating_clone.set(false);
                    });

                    let opacity1_spin_button_clone = opacity1_spin_button.clone();
                    let override1_switch_clone = override1_switch.clone();
                    let opacity2_spin_button_clone = opacity2_spin_button.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    override2_switch.connect_state_notify(move |switch| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let opacity1 = opacity1_spin_button_clone.value();
                        let override1 = override1_switch_clone.state();
                        let opacity2 = opacity2_spin_button_clone.value();
                        let override2 = switch.state();
                        entry_clone.set_text(
                            &HyprOpacity::ActiveAndInactive(
                                opacity1, override1, opacity2, override2,
                            )
                            .to_string(),
                        );
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        update_ui(parse_value(entry.text().as_str()));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                _opacity1,
                _override1,
                _opacity2,
                _override2,
                _opacity3,
                _override3,
            ) => Some(|entry, _separator, _labels| {
                let is_updating = Rc::new(Cell::new(false));
                let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                let opacity1_spin_button = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                mother_box.append(&opacity1_spin_button);

                let override1_box = GtkBox::new(GtkOrientation::Vertical, 5);
                override1_box.append(&Label::new(Some(&t!("override"))));
                let override1_switch = create_switch();
                override1_box.append(&override1_switch);
                mother_box.append(&override1_box);

                let opacity2_spin_button = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                mother_box.append(&opacity2_spin_button);

                let override2_box = GtkBox::new(GtkOrientation::Vertical, 5);
                override2_box.append(&Label::new(Some(&t!("override"))));
                let override2_switch = create_switch();
                override2_box.append(&override2_switch);
                mother_box.append(&override2_box);

                let opacity3_spin_button = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                mother_box.append(&opacity3_spin_button);

                let override3_box = GtkBox::new(GtkOrientation::Vertical, 5);
                override3_box.append(&Label::new(Some(&t!("override"))));
                let override3_switch = create_switch();
                override3_box.append(&override3_switch);
                mother_box.append(&override3_box);

                let opacity1_spin_button_clone = opacity1_spin_button.clone();
                let override1_switch_clone = override1_switch.clone();
                let opacity2_spin_button_clone = opacity2_spin_button.clone();
                let override2_switch_clone = override2_switch.clone();
                let opacity3_spin_button_clone = opacity3_spin_button.clone();
                let override3_switch_clone = override3_switch.clone();
                let update_ui = move |(
                    opacity1,
                    override1,
                    opacity2,
                    override2,
                    opacity3,
                    override3,
                ): (f64, bool, f64, bool, f64, bool)| {
                    opacity1_spin_button_clone.set_value(opacity1);
                    override1_switch_clone.set_state(override1);
                    opacity2_spin_button_clone.set_value(opacity2);
                    override2_switch_clone.set_state(override2);
                    opacity3_spin_button_clone.set_value(opacity3);
                    override3_switch_clone.set_state(override3);
                };

                let parse_value = |str: &str| {
                    let hypr_opacity = str.parse().unwrap_or_default();
                    match hypr_opacity {
                        HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1,
                            override1,
                            opacity2,
                            override2,
                            opacity3,
                            override3,
                        ) => (
                            opacity1, override1, opacity2, override2, opacity3, override3,
                        ),
                        HyprOpacity::ActiveAndInactive(
                            opacity1,
                            override1,
                            opacity2,
                            override2,
                        ) => (opacity1, override1, opacity2, override2, 1.0, false),
                        HyprOpacity::Overall(opacity, override1) => {
                            (opacity, override1, 1.0, false, 1.0, false)
                        }
                    }
                };

                update_ui(parse_value(entry.text().as_str()));

                let override1_switch_clone = override1_switch.clone();
                let override2_switch_clone = override2_switch.clone();
                let override3_switch_clone = override3_switch.clone();
                let opacity2_spin_button_clone = opacity2_spin_button.clone();
                let opacity3_spin_button_clone = opacity3_spin_button.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                opacity1_spin_button.connect_value_changed(move |spin_button| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let opacity1 = spin_button.value();
                    let override1 = override1_switch_clone.state();
                    let opacity2 = opacity2_spin_button_clone.value();
                    let override2 = override2_switch_clone.state();
                    let opacity3 = opacity3_spin_button_clone.value();
                    let override3 = override3_switch_clone.state();
                    entry_clone.set_text(
                        &HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1, override1, opacity2, override2, opacity3, override3,
                        )
                        .to_string(),
                    );
                    is_updating_clone.set(false);
                });

                let opacity1_spin_button_clone = opacity1_spin_button.clone();
                let override2_switch_clone = override2_switch.clone();
                let override3_switch_clone = override3_switch.clone();
                let opacity2_spin_button_clone = opacity2_spin_button.clone();
                let opacity3_spin_button_clone = opacity3_spin_button.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                override1_switch.connect_state_notify(move |switch| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let opacity1 = opacity1_spin_button_clone.value();
                    let override1 = switch.state();
                    let opacity2 = opacity2_spin_button_clone.value();
                    let override2 = override2_switch_clone.state();
                    let opacity3 = opacity3_spin_button_clone.value();
                    let override3 = override3_switch_clone.state();
                    entry_clone.set_text(
                        &HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1, override1, opacity2, override2, opacity3, override3,
                        )
                        .to_string(),
                    );
                    is_updating_clone.set(false);
                });

                let override1_switch_clone = override1_switch.clone();
                let override2_switch_clone = override2_switch.clone();
                let override3_switch_clone = override3_switch.clone();
                let opacity1_spin_button_clone = opacity1_spin_button.clone();
                let opacity3_spin_button_clone = opacity3_spin_button.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                opacity2_spin_button.connect_value_changed(move |spin_button| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let opacity1 = opacity1_spin_button_clone.value();
                    let override1 = override1_switch_clone.state();
                    let opacity2 = spin_button.value();
                    let override2 = override2_switch_clone.state();
                    let opacity3 = opacity3_spin_button_clone.value();
                    let override3 = override3_switch_clone.state();
                    entry_clone.set_text(
                        &HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1, override1, opacity2, override2, opacity3, override3,
                        )
                        .to_string(),
                    );
                    is_updating_clone.set(false);
                });

                let opacity1_spin_button_clone = opacity1_spin_button.clone();
                let override1_switch_clone = override1_switch.clone();
                let opacity2_spin_button_clone = opacity2_spin_button.clone();
                let override3_switch_clone = override3_switch.clone();
                let opacity3_spin_button_clone = opacity3_spin_button.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                override2_switch.connect_state_notify(move |switch| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let opacity1 = opacity1_spin_button_clone.value();
                    let override1 = override1_switch_clone.state();
                    let opacity2 = opacity2_spin_button_clone.value();
                    let override2 = switch.state();
                    let opacity3 = opacity3_spin_button_clone.value();
                    let override3 = override3_switch_clone.state();
                    entry_clone.set_text(
                        &HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1, override1, opacity2, override2, opacity3, override3,
                        )
                        .to_string(),
                    );
                    is_updating_clone.set(false);
                });

                let override1_switch_clone = override1_switch.clone();
                let override2_switch_clone = override2_switch.clone();
                let override3_switch_clone = override3_switch.clone();
                let opacity1_spin_button_clone = opacity1_spin_button.clone();
                let opacity2_spin_button_clone = opacity2_spin_button.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                opacity3_spin_button.connect_value_changed(move |spin_button| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let opacity1 = opacity1_spin_button_clone.value();
                    let override1 = override1_switch_clone.state();
                    let opacity2 = opacity2_spin_button_clone.value();
                    let override2 = override2_switch_clone.state();
                    let opacity3 = spin_button.value();
                    let override3 = override3_switch_clone.state();
                    entry_clone.set_text(
                        &HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1, override1, opacity2, override2, opacity3, override3,
                        )
                        .to_string(),
                    );
                    is_updating_clone.set(false);
                });

                let opacity1_spin_button_clone = opacity1_spin_button.clone();
                let override1_switch_clone = override1_switch.clone();
                let override2_switch_clone = override2_switch.clone();
                let opacity2_spin_button_clone = opacity2_spin_button.clone();
                let opacity3_spin_button_clone = opacity3_spin_button.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                override3_switch.connect_state_notify(move |switch| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let opacity1 = opacity1_spin_button_clone.value();
                    let override1 = override1_switch_clone.state();
                    let opacity2 = opacity2_spin_button_clone.value();
                    let override2 = override2_switch_clone.state();
                    let opacity3 = opacity3_spin_button_clone.value();
                    let override3 = switch.state();
                    entry_clone.set_text(
                        &HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1, override1, opacity2, override2, opacity3, override3,
                        )
                        .to_string(),
                    );
                    is_updating_clone.set(false);
                });

                let is_updating_clone = is_updating.clone();
                entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    update_ui(parse_value(entry.text().as_str()));
                    is_updating_clone.set(false);
                });

                mother_box
            }),
        }
    }
}

impl EnumConfigForGtk for Side {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("left"), &t!("right"), &t!("top"), &t!("bottom")])
    }
}

impl EnumConfigForGtk for AnimationStyle {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("none"),
            &t!("slide"),
            &t!("slide_with_side"),
            &t!("popin"),
            &t!("popin_with_percent"),
            &t!("gnomed"),
            &t!("slide_vert"),
            &t!("slide_vert_with_percent"),
            &t!("fade"),
            &t!("slide_fade"),
            &t!("slide_fade_with_percent"),
            &t!("slide_fade_vert"),
            &t!("slide_fade_vert_with_percent"),
            &t!("once"),
            &t!("loop"),
        ])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            AnimationStyle::None => None,
            AnimationStyle::Slide => None,
            AnimationStyle::SlideSide(_side) => Some(<(Side,)>::to_gtk_box),
            AnimationStyle::SlidePercent(_) => Some(|entry, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::Popin => None,
            AnimationStyle::PopinPercent(_percent) => Some(|entry, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::Gnomed => None,
            AnimationStyle::SlideVert => None,
            AnimationStyle::SlideVertPercent(_percent) => Some(|entry, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::Fade => None,
            AnimationStyle::SlideFade => None,
            AnimationStyle::SlideFadePercent(_percent) => Some(|entry, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::SlideFadeVert => None,
            AnimationStyle::SlideFadeVertPercent(_percent) => Some(|entry, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::Once => None,
            AnimationStyle::Loop => None,
        }
    }
}

impl EnumConfigForGtk for TagToggleState {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("set"), &t!("unset"), &t!("toggle")])
    }
}

impl EnumConfigForGtk for WindowRule {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("float"),
            &t!("tile"),
            &t!("fullscreen"),
            &t!("maximize"),
            &t!("persistent_size"),
            &t!("fullscreen_state"),
            &t!("move"),
            &t!("size"),
            &t!("center"),
            &t!("center_with_respect_to_monitor_reserved_area"),
            &t!("pseudo"),
            &t!("monitor"),
            &t!("workspace"),
            &t!("no_initial_focus"),
            &t!("pin"),
            &t!("unset"),
            &t!("no_max_size"),
            &t!("stay_focused"),
            &t!("group"),
            &t!("suppress_event"),
            &t!("content"),
            &t!("no_close_for"),
            &t!("animation"),
            &t!("border_color"),
            &t!("idle_ingibit"),
            &t!("opacity"),
            &t!("tag"),
            &t!("max_size"),
            &t!("min_size"),
        ])
    }

    fn separator() -> Option<char> {
        Some(' ')
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            WindowRule::Float => None,
            WindowRule::Tile => None,
            WindowRule::Fullscreen => None,
            WindowRule::Maximize => None,
            WindowRule::PersistentSize => None,
            WindowRule::FullscreenState(_fullscreen_state1, _fullscreen_state2) => {
                Some(<(FullscreenState, FullscreenState)>::to_gtk_box)
            }
            WindowRule::Move(_hypr_coord) => Some(<(HyprCoord,)>::to_gtk_box),
            WindowRule::Size(_hypr_size) => Some(<(HyprSize,)>::to_gtk_box),
            WindowRule::Center => None,
            WindowRule::CenterWithRespectToMonitorReservedArea => None,
            WindowRule::Pseudo => None,
            WindowRule::Monitor(_id_or_name) => Some(<(IdOrName,)>::to_gtk_box),
            WindowRule::Workspace(_workspace_target) => Some(<(WorkspaceTarget,)>::to_gtk_box),
            WindowRule::NoInitialFocus => None,
            WindowRule::Pin => None,
            WindowRule::Unset => None,
            WindowRule::NoMaxSize => None,
            WindowRule::StayFocused => None,
            WindowRule::Group(_window_group_option) => Some(|entry, separator, _names| {
                Vec::<WindowGroupOption>::to_gtk_box(entry, separator)
            }),
            WindowRule::SuppressEvent(_window_event) => Some(|entry, separator, _names| {
                HashSet::<WindowEvent>::to_gtk_box(entry, separator)
            }),
            WindowRule::Content(_content_type) => Some(<(ContentType,)>::to_gtk_box),
            WindowRule::NoCloseFor(_duration) => Some(<(u32,)>::to_gtk_box),
            WindowRule::Animation(_animation_style) => Some(<(AnimationStyle,)>::to_gtk_box),
            WindowRule::BorderColor(_border_color) => Some(<(BorderColor,)>::to_gtk_box),
            WindowRule::IdleIngibit(_idle_ingibit_mode) => Some(<(IdleIngibitMode,)>::to_gtk_box),
            WindowRule::Opacity(_hypr_opacity) => Some(<(HyprOpacity,)>::to_gtk_box),
            WindowRule::Tag(_tag_toggle_state, _tag) => Some(|entry, _separator, names| {
                <(TagToggleState, String)>::to_gtk_box(entry, PLUG_SEPARATOR, names)
            }),
            WindowRule::MaxSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            WindowRule::MinSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
        }
    }
}

impl EnumConfigForGtk for KeyState {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("down"), &t!("repeat"), &t!("up")])
    }
}

impl EnumConfigForGtk for DispatcherFullscreenState {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("current"),
            &t!("none"),
            &t!("maximize"),
            &t!("fullscreen"),
            &t!("maximize_and_fullscreen"),
        ])
    }
}

impl EnumConfigForGtk for MoveDirection {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("direction"),
            &t!("direction_silent"),
            &t!("monitor"),
            &t!("monitor_silent"),
        ])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            MoveDirection::Direction(_direction) => Some(<(Direction,)>::to_gtk_box),
            MoveDirection::DirectionSilent(_direction) => Some(<(Direction,)>::to_gtk_box),
            MoveDirection::Monitor(_monitor_target) => Some(<(MonitorTarget,)>::to_gtk_box),
            MoveDirection::MonitorSilent(_monitor_target) => Some(<(MonitorTarget,)>::to_gtk_box),
        }
    }
}

impl EnumConfigForGtk for SwapDirection {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("direction"), &t!("window")])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            SwapDirection::Direction(_direction) => Some(<(Direction,)>::to_gtk_box),
            SwapDirection::Window(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
        }
    }
}

impl ToGtkBox for CycleNext {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let is_prev_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_prev_box.append(&Label::new(Some(&t!("is_previous"))));
        let is_prev_switch = create_switch();
        is_prev_box.append(&is_prev_switch);
        mother_box.append(&is_prev_box);

        let is_tiled_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_tiled_box.append(&Label::new(Some(&t!("is_tiled"))));
        let is_tiled_switch = create_switch();
        is_tiled_box.append(&is_tiled_switch);
        mother_box.append(&is_tiled_box);

        let is_floating_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_floating_box.append(&Label::new(Some(&t!("is_floating"))));
        let is_floating_switch = create_switch();
        is_floating_box.append(&is_floating_switch);
        mother_box.append(&is_floating_box);

        let is_visible_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_visible_box.append(&Label::new(Some(&t!("is_visible"))));
        let is_visible_switch = create_switch();
        is_visible_box.append(&is_visible_switch);
        mother_box.append(&is_visible_box);

        let is_hist_box = GtkBox::new(GtkOrientation::Vertical, 5);
        is_hist_box.append(&Label::new(Some(&t!("is_hist"))));
        let is_hist_switch = create_switch();
        is_hist_box.append(&is_hist_switch);
        mother_box.append(&is_hist_box);

        let is_prev_switch_clone = is_prev_switch.clone();
        let is_tiled_switch_clone = is_tiled_switch.clone();
        let is_floating_switch_clone = is_floating_switch.clone();
        let is_visible_switch_clone = is_visible_switch.clone();
        let is_hist_switch_clone = is_hist_switch.clone();
        let update_ui = move |cycle_next: CycleNext| {
            is_prev_switch_clone.set_state(cycle_next.is_prev);
            is_tiled_switch_clone.set_state(cycle_next.is_tiled);
            is_floating_switch_clone.set_state(cycle_next.is_floating);
            is_visible_switch_clone.set_state(cycle_next.is_visible);
            is_hist_switch_clone.set_state(cycle_next.is_hist);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_prev_switch.connect_state_notify(move |is_prev_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_prev = is_prev_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_tiled_switch.connect_state_notify(move |is_tiled_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_tiled = is_tiled_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_floating_switch.connect_state_notify(move |is_floating_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_floating = is_floating_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_visible_switch.connect_state_notify(move |is_visible_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_visible = is_visible_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        is_hist_switch.connect_state_notify(move |is_hist_switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut cycle_next: CycleNext = entry_clone.text().parse().unwrap_or_default();
            cycle_next.is_hist = is_hist_switch.state();
            entry_clone.set_text(&cycle_next.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let cycle_next: CycleNext = entry.text().parse().unwrap_or_default();
            update_ui(cycle_next);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl EnumConfigForGtk for SwapNext {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("next"), &t!("prev")])
    }
}

impl EnumConfigForGtk for ChangeGroupActive {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("back"), &t!("forward"), &t!("index")])
    }

    fn separator() -> Option<char> {
        Some(PLUG_SEPARATOR)
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            ChangeGroupActive::Back => None,
            ChangeGroupActive::Forward => None,
            ChangeGroupActive::Index(_index) => Some(<(u32,)>::to_gtk_box),
        }
    }
}

impl EnumConfigForGtk for SetPropToggleState {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("on"), &t!("off"), &t!("toggle"), &t!("unset")])
    }
}

impl ToGtkBox for HyprGradient {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Vertical, 5);

        let colors_box = GtkBox::new(GtkOrientation::Vertical, 5);
        colors_box.append(&Label::new(Some(&t!("gradient_colors"))));

        let colors_entry = create_entry();
        let colors_separator = ',';
        let colors_ui_box = Vec::<HyprColor>::to_gtk_box(&colors_entry, colors_separator);
        colors_box.append(&colors_ui_box);
        mother_box.append(&colors_box);

        let angle_box = GtkBox::new(GtkOrientation::Vertical, 5);
        angle_box.append(&Label::new(Some(&t!("gradient_angle"))));

        let angle_entry = create_entry();
        let angle_ui_box = Option::<Angle>::to_gtk_box(&angle_entry);
        angle_box.append(&angle_ui_box);
        mother_box.append(&angle_box);

        let colors_entry_clone = colors_entry.clone();
        let angle_entry_clone = angle_entry.clone();
        let update_ui = move |gradient: HyprGradient| {
            let colors_text = join_with_separator(&gradient.colors, &colors_separator.to_string());
            colors_entry_clone.set_text(&colors_text);

            let angle_text = gradient.angle.map(|a| a.to_string()).unwrap_or_default();
            angle_entry_clone.set_text(&angle_text);
        };

        update_ui(entry.text().parse().unwrap_or_default());

        let entry_clone = entry.clone();
        let angle_entry_clone = angle_entry.clone();
        let is_updating_clone = is_updating.clone();
        colors_entry.connect_changed(move |colors_entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let colors: Vec<HyprColor> = colors_entry
                .text()
                .split(colors_separator)
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().unwrap_or_default())
                .collect();

            let angle_str = angle_entry_clone.text().to_string();
            let angle = match angle_str.as_str() {
                "" => None,
                _ => Some(angle_str.parse().unwrap_or_default()),
            };

            let gradient = HyprGradient { colors, angle };

            entry_clone.set_text(&gradient.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let colors_entry_clone = colors_entry.clone();
        let is_updating_clone = is_updating.clone();
        angle_entry.connect_changed(move |angle_entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let colors: Vec<HyprColor> = colors_entry_clone
                .text()
                .split(colors_separator)
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().unwrap_or_default())
                .collect();

            let angle = match angle_entry.text().as_str() {
                "" => None,
                s => Some(s.parse().unwrap_or_default()),
            };

            let gradient = HyprGradient { colors, angle };

            entry_clone.set_text(&gradient.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);

            let gradient = entry.text().parse().unwrap_or_default();
            update_ui(gradient);

            is_updating_clone.set(false);
        });

        mother_box
    }
}

impl EnumConfigForGtk for SetProp {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("alpha"),
            &t!("alpha_override"),
            &t!("alpha_inactive"),
            &t!("alpha_inactive_override"),
            &t!("alpha_fullscreen"),
            &t!("alpha_fullscreen_override"),
            &t!("animation_style"),
            &t!("active_border_color"),
            &t!("inactive_border_color"),
            &t!("animation"),
            &t!("border_color"),
            &t!("idle_ingibit"),
            &t!("opacity"),
            &t!("tag"),
            &t!("max_size"),
            &t!("min_size"),
            &t!("border_size"),
            &t!("rounding"),
            &t!("rounding_power"),
            &t!("allows_input"),
            &t!("dim_around"),
            &t!("decorate"),
            &t!("focus_on_activate"),
            &t!("keep_aspect_ratio"),
            &t!("nearest_neighbor"),
            &t!("no_anim"),
            &t!("no_blur"),
            &t!("no_border"),
            &t!("no_dim"),
            &t!("no_focus"),
            &t!("no_follow_mouse"),
            &t!("no_max_size"),
            &t!("no_rounding"),
            &t!("no_shadow"),
            &t!("no_shortcuts_inhibit"),
            &t!("opaque"),
            &t!("force_rgbx"),
            &t!("sync_fullscreen"),
            &t!("immediate"),
            &t!("xray"),
            &t!("render_unfocused"),
            &t!("scroll_mouse"),
            &t!("scroll_touchpad"),
            &t!("no_screenshare"),
            &t!("no_vrr"),
        ])
    }

    fn separator() -> Option<char> {
        Some(' ')
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            SetProp::Alpha(_alpha) => Some(|entry, _, names| {
                create_spin_button_builder(0.0, 1.0, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::AlphaOverride(_override) => Some(<(bool,)>::to_gtk_box),
            SetProp::AlphaInactive(_alpha) => Some(|entry, _, names| {
                create_spin_button_builder(0.0, 1.0, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::AlphaInactiveOverride(_override) => Some(<(bool,)>::to_gtk_box),
            SetProp::AlphaFullscreen(_alpha) => Some(|entry, _, names| {
                create_spin_button_builder(0.0, 1.0, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::AlphaFullscreenOverride(_override) => Some(<(bool,)>::to_gtk_box),
            SetProp::AnimationStyle(_style) => Some(<(String,)>::to_gtk_box),
            SetProp::ActiveBorderColor(_optional_gradient) => {
                Some(|entry, _, _| Option::<HyprGradient>::to_gtk_box(entry))
            }
            SetProp::InactiveBorderColor(_optional_gradient) => {
                Some(|entry, _, _| Option::<HyprGradient>::to_gtk_box(entry))
            }
            SetProp::Animation(_style) => Some(<(AnimationStyle,)>::to_gtk_box),
            SetProp::BorderColor(_color) => Some(<(BorderColor,)>::to_gtk_box),
            SetProp::IdleIngibit(_mode) => Some(<(IdleIngibitMode,)>::to_gtk_box),
            SetProp::Opacity(_opacity) => Some(<(HyprOpacity,)>::to_gtk_box),
            SetProp::Tag(_toggle_state, _tag) => Some(|entry, _, names| {
                <(TagToggleState, String)>::to_gtk_box(entry, PLUG_SEPARATOR, names)
            }),
            SetProp::MaxSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            SetProp::MinSize(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            SetProp::BorderSize(_size) => Some(<(u32,)>::to_gtk_box),
            SetProp::Rounding(_size) => Some(<(u32,)>::to_gtk_box),
            SetProp::RoundingPower(_power) => Some(|entry, _, names| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::AllowsInput(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::DimAround(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::Decorate(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::FocusOnActivate(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::KeepAspectRatio(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NearestNeighbor(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoAnim(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoBlur(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoBorder(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoDim(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoFocus(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoFollowMouse(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoMaxSize(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoRounding(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoShadow(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoShortcutsInhibit(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::Opaque(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::ForceRGBX(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::SyncFullscreen(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::Immediate(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::Xray(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::RenderUnfocused => None,
            SetProp::ScrollMouse(_scroll_factor) => Some(|entry, _, names| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::ScrollTouchpad(_scroll_factor) => Some(|entry, _, names| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            SetProp::NoScreenShare(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
            SetProp::NoVRR(_toggle_state) => Some(<(SetPropToggleState,)>::to_gtk_box),
        }
    }
}

impl EnumConfigForGtk for Modifier {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            "SHIFT", "CAPS", "CTRL", "ALT", "MOD2", "MOD3", "SUPER", "MOD5",
        ])
    }
}

impl EnumConfigForGtk for Dispatcher {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("exec"),
            &t!("execr"),
            &t!("pass"),
            &t!("send_shortcut"),
            &t!("send_key_state"),
            &t!("kill_active"),
            &t!("force_kill_active"),
            &t!("close_window"),
            &t!("kill_window"),
            &t!("signal"),
            &t!("signal_window"),
            &t!("workspace"),
            &t!("move_to_workspace"),
            &t!("move_to_workspace_silent"),
            &t!("toggle_floating"),
            &t!("set_floating"),
            &t!("set_tiled"),
            &t!("fullscreen"),
            &t!("fullscreen_state"),
            &t!("dpms"),
            &t!("pin"),
            &t!("move_focus"),
            &t!("move_window"),
            &t!("swap_window"),
            &t!("center_window"),
            &t!("resize_active"),
            &t!("move_active"),
            &t!("resize_window_pixel"),
            &t!("move_window_pixel"),
            &t!("cycle_next"),
            &t!("swap_next"),
            &t!("tag_window"),
            &t!("focus_window"),
            &t!("focus_monitor"),
            &t!("split_ratio"),
            &t!("move_cursor_to_corner"),
            &t!("move_cursor"),
            &t!("rename_workspace"),
            &t!("exit"),
            &t!("force_renderer_reload"),
            &t!("move_current_workspace_to_monitor"),
            &t!("focus_workspace_on_current_monitor"),
            &t!("move_workspace_to_monitor"),
            &t!("swap_active_workspaces"),
            &t!("bring_active_to_top"),
            &t!("alter_z_order"),
            &t!("toggle_special_workspace"),
            &t!("focus_urgent_or_last"),
            &t!("toggle_group"),
            &t!("change_group_active"),
            &t!("focus_current_or_last"),
            &t!("lock_groups"),
            &t!("lock_active_group"),
            &t!("move_into_group"),
            &t!("move_out_of_group"),
            &t!("move_window_or_group"),
            &t!("move_group_window"),
            &t!("deny_window_from_group"),
            &t!("set_ignore_group_lock"),
            &t!("global"),
            &t!("event"),
            &t!("set_prop"),
            &t!("toggle_swallow"),
        ])
    }

    fn separator() -> Option<char> {
        Some(' ')
    }

    fn parameter_builder(&self) -> Option<fn(&Entry, char, &[FieldLabel]) -> GtkBox> {
        match self {
            Self::Exec(_window_rules, _command) => Some(|entry, separator, _names| {
                let is_updating = Rc::new(Cell::new(false));
                let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                let window_rules_mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
                window_rules_mother_box.append(&Label::new(Some(&t!("window_rules"))));
                let window_rules_entry = create_entry();
                let window_rules_box = Vec::<WindowRule>::to_gtk_box(&window_rules_entry, ';');
                window_rules_mother_box.append(&window_rules_box);
                mother_box.append(&window_rules_mother_box);

                let command_entry = create_entry();
                mother_box.append(&command_entry);

                let window_rules_entry_clone = window_rules_entry.clone();
                let command_entry_clone = command_entry.clone();
                let update_ui = move |(window_rules, command): (Vec<WindowRule>, String)| {
                    window_rules_entry_clone.set_text(&join_with_separator(&window_rules, ";"));
                    command_entry_clone.set_text(&command);
                };

                let parse_value = |str: &str| {
                    let dispatcher =
                        Self::from_discriminant_and_str(DispatcherDiscriminant::Exec, str);
                    match dispatcher {
                        Dispatcher::Exec(window_rules, command) => (window_rules, command),
                        _ => (vec![], String::new()),
                    }
                };

                update_ui(parse_value(entry.text().as_str()));

                let command_entry_clone = command_entry.clone();
                let entry_clone = entry.clone();
                let is_updating_clone = is_updating.clone();
                window_rules_entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    entry_clone.set_text(&format!(
                        "[{}]{}{}",
                        entry.text(),
                        separator,
                        command_entry_clone.text()
                    ));
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
                    entry_clone.set_text(&format!(
                        "[{}]{}{}",
                        window_rules_entry_clone.text(),
                        separator,
                        entry.text()
                    ));
                    is_updating_clone.set(false);
                });

                let is_updating_clone = is_updating.clone();
                entry.connect_changed(move |entry| {
                    if is_updating_clone.get() {
                        return;
                    }
                    is_updating_clone.set(true);
                    let (window_rules, command) = parse_value(entry.text().as_str());
                    update_ui((window_rules, command));
                    is_updating_clone.set(false);
                });

                mother_box
            }),
            Self::Execr(_command) => Some(<(String,)>::to_gtk_box),
            Self::Pass(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
            Self::SendShortcut(_modifiers, _key, _window_target) => {
                Some(|entry, separator, _names| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let modifiers_entry = create_entry();
                    let modifiers_box = HashSet::<Modifier>::to_gtk_box(&modifiers_entry, '_');
                    mother_box.append(&modifiers_box);

                    let key_entry = create_entry();
                    mother_box.append(&key_entry);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box.append(&Label::new(Some(&t!("window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(modifiers, key, window_target): (
                        HashSet<Modifier>,
                        String,
                        Option<WindowTarget>,
                    )| {
                        modifiers_entry_clone.set_text(&join_with_separator(&modifiers, "_"));
                        key_entry_clone.set_text(&key);
                        let window_target_str = match window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::SendShortcut,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::SendShortcut(modifiers, key, window_target) => {
                                (modifiers, key, window_target)
                            }
                            _ => (HashSet::new(), String::new(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let key_entry_clone = key_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    modifiers_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => {
                                format!("{}{}{}", entry.text(), separator, key_entry_clone.text())
                            }
                            window_target_str => format!(
                                "{}{}{}{}{}",
                                entry.text(),
                                separator,
                                key_entry_clone.text(),
                                separator,
                                window_target_str
                            ),
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    key_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => {
                                format!(
                                    "{}{}{}",
                                    modifiers_entry_clone.text(),
                                    separator,
                                    entry.text()
                                )
                            }
                            window_target_str => format!(
                                "{}{}{}{}{}",
                                modifiers_entry_clone.text(),
                                separator,
                                entry.text(),
                                separator,
                                window_target_str
                            ),
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => {
                                format!(
                                    "{}{}{}",
                                    modifiers_entry_clone.text(),
                                    separator,
                                    key_entry_clone.text()
                                )
                            }
                            window_target_str => format!(
                                "{}{}{}{}{}",
                                modifiers_entry_clone.text(),
                                separator,
                                key_entry_clone.text(),
                                separator,
                                window_target_str
                            ),
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (modifiers, key, window_target) = parse_value(entry.text().as_str());
                        update_ui((modifiers, key, window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::SendKeyState(_modifiers, _key, _state, _window_target) => {
                Some(|entry, separator, _names| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let modifiers_entry = create_entry();
                    let modifiers_box = HashSet::<Modifier>::to_gtk_box(&modifiers_entry, '_');
                    mother_box.append(&modifiers_box);

                    let key_entry = create_entry();
                    mother_box.append(&key_entry);

                    let state_entry = create_entry();
                    let state_box = KeyState::to_gtk_box(&state_entry);
                    mother_box.append(&state_box);

                    let window_target_entry = create_entry();
                    let window_target_box = WindowTarget::to_gtk_box(&window_target_entry);
                    mother_box.append(&window_target_box);

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let state_entry_clone = state_entry.clone();
                    let window_target_entry_clone = window_target_entry.clone();
                    let update_ui = move |(modifiers, key, state, window_target): (
                        HashSet<Modifier>,
                        String,
                        KeyState,
                        WindowTarget,
                    )| {
                        modifiers_entry_clone.set_text(&join_with_separator(&modifiers, "_"));
                        key_entry_clone.set_text(&key);
                        state_entry_clone.set_text(&state.to_string());
                        window_target_entry_clone.set_text(&window_target.to_string());
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::SendKeyState,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::SendKeyState(modifiers, key, state, window_target) => {
                                (modifiers, key, state, window_target)
                            }
                            _ => (
                                HashSet::new(),
                                String::new(),
                                KeyState::default(),
                                WindowTarget::default(),
                            ),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let key_entry_clone = key_entry.clone();
                    let state_entry_clone = state_entry.clone();
                    let window_target_entry_clone = window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    modifiers_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        entry_clone.set_text(&format!(
                            "{}{}{}{}{}{}{}",
                            entry.text(),
                            separator,
                            key_entry_clone.text(),
                            separator,
                            state_entry_clone.text(),
                            separator,
                            window_target_entry_clone.text()
                        ));
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let state_entry_clone = state_entry.clone();
                    let window_target_entry_clone = window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    key_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        entry_clone.set_text(&format!(
                            "{}{}{}{}{}{}{}",
                            modifiers_entry_clone.text(),
                            separator,
                            entry.text(),
                            separator,
                            state_entry_clone.text(),
                            separator,
                            window_target_entry_clone.text()
                        ));
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let window_target_entry_clone = window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    state_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        entry_clone.set_text(&format!(
                            "{}{}{}{}{}{}{}",
                            modifiers_entry_clone.text(),
                            separator,
                            key_entry_clone.text(),
                            separator,
                            entry.text(),
                            separator,
                            window_target_entry_clone.text()
                        ));
                        is_updating_clone.set(false);
                    });

                    let modifiers_entry_clone = modifiers_entry.clone();
                    let key_entry_clone = key_entry.clone();
                    let state_entry_clone = state_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        entry_clone.set_text(&format!(
                            "{}{}{}{}{}{}{}",
                            modifiers_entry_clone.text(),
                            separator,
                            key_entry_clone.text(),
                            separator,
                            state_entry_clone.text(),
                            separator,
                            entry.text()
                        ));
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (modifiers, key, state, window_target) =
                            parse_value(entry.text().as_str());
                        update_ui((modifiers, key, state, window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::KillActive => None,
            Self::ForceKillActive => None,
            Self::CloseWindow(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
            Self::KillWindow(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
            Self::Signal(_signal) => Some(<(String,)>::to_gtk_box),
            Self::SignalWindow(_window_target, _signal) => {
                Some(<(WindowTarget, String)>::to_gtk_box)
            }
            Self::Workspace(_workspace_target) => Some(<(WorkspaceTarget,)>::to_gtk_box),
            Self::MoveToWorkspace(_workspace_target, _optional_window_target) => {
                Some(|entry, separator, _names| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let workspace_target_entry = create_entry();
                    let workspace_target_box = WorkspaceTarget::to_gtk_box(&workspace_target_entry);
                    mother_box.append(&workspace_target_box);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box.append(&Label::new(Some(&t!("window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let workspace_target_entry_clone = workspace_target_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(workspace_target, optional_window_target): (
                        WorkspaceTarget,
                        Option<WindowTarget>,
                    )| {
                        workspace_target_entry_clone.set_text(&workspace_target.to_string());
                        let window_target_str = match optional_window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::MoveToWorkspace,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::MoveToWorkspace(workspace_target, window_target) => {
                                (workspace_target, window_target)
                            }
                            _ => (WorkspaceTarget::default(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    workspace_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => entry.text().to_string(),
                            window_target => {
                                format!("{}{}{}", entry.text(), separator, window_target)
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let workspace_target_entry_clone = workspace_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => workspace_target_entry_clone.text().to_string(),
                            window_target => {
                                format!(
                                    "{}{}{}",
                                    workspace_target_entry_clone.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (workspace_target, optional_window_target) =
                            parse_value(entry.text().as_str());
                        update_ui((workspace_target, optional_window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::MoveToWorkspaceSilent(_workspace_target, _optional_window_target) => {
                Some(|entry, separator, _names| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let workspace_target_entry = create_entry();
                    let workspace_target_box = WorkspaceTarget::to_gtk_box(&workspace_target_entry);
                    mother_box.append(&workspace_target_box);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box.append(&Label::new(Some(&t!("window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let workspace_target_entry_clone = workspace_target_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(workspace_target, optional_window_target): (
                        WorkspaceTarget,
                        Option<WindowTarget>,
                    )| {
                        workspace_target_entry_clone.set_text(&workspace_target.to_string());
                        let window_target_str = match optional_window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::MoveToWorkspaceSilent,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::MoveToWorkspaceSilent(workspace_target, window_target) => {
                                (workspace_target, window_target)
                            }
                            _ => (WorkspaceTarget::default(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    workspace_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => entry.text().to_string(),
                            window_target => {
                                format!("{}{}{}", entry.text(), separator, window_target)
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let workspace_target_entry_clone = workspace_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => workspace_target_entry_clone.text().to_string(),
                            window_target => {
                                format!(
                                    "{}{}{}",
                                    workspace_target_entry_clone.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (workspace_target, optional_window_target) =
                            parse_value(entry.text().as_str());
                        update_ui((workspace_target, optional_window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::ToggleFloating(_optional_window_target)
            | Self::SetFloating(_optional_window_target)
            | Self::SetTiled(_optional_window_target) => {
                Some(|entry, _separator, _names| Option::<WindowTarget>::to_gtk_box(entry))
            }
            Self::Fullscreen(_fullscreen_mode) => Some(<(FullscreenMode,)>::to_gtk_box),
            Self::FullscreenState(_fullscreen_state1, _fullscreen_state2) => {
                Some(<(DispatcherFullscreenState, DispatcherFullscreenState)>::to_gtk_box)
            }
            Self::Dpms(_toggle_state, _optional_monitor_name) => {
                Some(|entry, separator, _names| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let toggle_state_entry = create_entry();
                    let toggle_state_box = ToggleState::to_gtk_box(&toggle_state_entry);
                    mother_box.append(&toggle_state_box);

                    let monitor_name_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    monitor_name_box.append(&Label::new(Some(&t!("monitor_name"))));
                    let optional_monitor_name_entry = create_entry();
                    let optional_monitor_name_box =
                        Option::<String>::to_gtk_box(&optional_monitor_name_entry);
                    monitor_name_box.append(&optional_monitor_name_box);
                    mother_box.append(&monitor_name_box);

                    let toggle_state_entry_clone = toggle_state_entry.clone();
                    let optional_monitor_name_entry_clone = optional_monitor_name_entry.clone();
                    let update_ui =
                        move |(toggle_state, monitor_name): (ToggleState, Option<String>)| {
                            toggle_state_entry_clone.set_text(&toggle_state.to_string());
                            let monitor_name_str = monitor_name.unwrap_or_default();
                            optional_monitor_name_entry_clone.set_text(&monitor_name_str);
                        };

                    let parse_value = |str: &str| {
                        let dispatcher =
                            Self::from_discriminant_and_str(DispatcherDiscriminant::Dpms, str);
                        match dispatcher {
                            Dispatcher::Dpms(toggle_state, monitor_name) => {
                                (toggle_state, monitor_name)
                            }
                            _ => (ToggleState::default(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let optional_monitor_name_entry_clone = optional_monitor_name_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    toggle_state_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_monitor_name_entry_clone.text().as_str() {
                            "" => entry.text().to_string(),
                            monitor_name => {
                                format!("{}{}{}", entry.text(), separator, monitor_name)
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let toggle_state_entry_clone = toggle_state_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_monitor_name_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => toggle_state_entry_clone.text().to_string(),
                            monitor_name => {
                                format!(
                                    "{}{}{}",
                                    toggle_state_entry_clone.text(),
                                    separator,
                                    monitor_name
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (toggle_state, monitor_name) = parse_value(entry.text().as_str());
                        update_ui((toggle_state, monitor_name));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::Pin(_optional_window_target) => {
                Some(|entry, _separator, _names| Option::<WindowTarget>::to_gtk_box(entry))
            }
            Self::MoveFocus(_direction) => Some(<(Direction,)>::to_gtk_box),
            Self::MoveWindow(_move_direction) => Some(<(MoveDirection,)>::to_gtk_box),
            Self::SwapWindow(_swap_direction) => Some(<(SwapDirection,)>::to_gtk_box),
            Self::CenterWindow(_respect_monitor_reserved_area) => Some(<(bool,)>::to_gtk_box),
            Self::ResizeActive(_resize_params) => Some(<(ResizeParams,)>::to_gtk_box),
            Self::MoveActive(_resize_params) => Some(<(ResizeParams,)>::to_gtk_box),
            Self::ResizeWindowPixel(_resize_params, _window_target)
            | Self::MoveWindowPixel(_resize_params, _window_target) => {
                Some(<(ResizeParams, WindowTarget)>::to_gtk_box)
            }
            Self::CycleNext(_cycle_next) => Some(<(CycleNext,)>::to_gtk_box),
            Self::SwapNext(_swap_next) => Some(<(SwapNext,)>::to_gtk_box),
            Self::TagWindow(_tag_toggle_state, _tag, _optional_window_target) => {
                Some(|entry, separator, _names| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let tag_toggle_state_entry = create_entry();
                    let tag_toggle_state_box = TagToggleState::to_gtk_box(&tag_toggle_state_entry);
                    mother_box.append(&tag_toggle_state_box);

                    let tag_entry = create_entry();
                    mother_box.append(&tag_entry);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box.append(&Label::new(Some(&t!("window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let tag_toggle_state_entry_clone = tag_toggle_state_entry.clone();
                    let tag_entry_clone = tag_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(tag_toggle_state, tag, optional_window_target): (
                        TagToggleState,
                        String,
                        Option<WindowTarget>,
                    )| {
                        tag_toggle_state_entry_clone.set_text(&tag_toggle_state.to_string());
                        tag_entry_clone.set_text(&tag);
                        let window_target_str = match optional_window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher =
                            Self::from_discriminant_and_str(DispatcherDiscriminant::TagWindow, str);
                        match dispatcher {
                            Dispatcher::TagWindow(tag_toggle_state, tag, window_target) => {
                                (tag_toggle_state, tag, window_target)
                            }
                            _ => (TagToggleState::default(), String::new(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let tag_entry_clone = tag_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    tag_toggle_state_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => {
                                format!("{}{}{}", entry.text(), separator, tag_entry_clone.text())
                            }
                            window_target => format!(
                                "{}{}{}{}{}",
                                entry.text(),
                                separator,
                                tag_entry_clone.text(),
                                separator,
                                window_target
                            ),
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let tag_toggle_state_entry_clone = tag_toggle_state_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    tag_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => {
                                format!(
                                    "{}{}{}",
                                    tag_toggle_state_entry_clone.text(),
                                    separator,
                                    entry.text()
                                )
                            }
                            window_target => {
                                format!(
                                    "{}{}{}{}{}",
                                    tag_toggle_state_entry_clone.text(),
                                    separator,
                                    entry.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let tag_toggle_state_entry_clone = tag_toggle_state_entry.clone();
                    let tag_entry_clone = tag_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => {
                                format!(
                                    "{}{}{}",
                                    tag_toggle_state_entry_clone.text(),
                                    separator,
                                    tag_entry_clone.text()
                                )
                            }
                            window_target => {
                                format!(
                                    "{}{}{}{}{}",
                                    tag_toggle_state_entry_clone.text(),
                                    separator,
                                    tag_entry_clone.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (tag_toggle_state, tag, optional_window_target) =
                            parse_value(entry.text().as_str());
                        update_ui((tag_toggle_state, tag, optional_window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::FocusWindow(_window_target) => Some(<(WindowTarget,)>::to_gtk_box),
            Self::FocusMonitor(_monitor_target) => Some(<(MonitorTarget,)>::to_gtk_box),
            Self::SplitRatio(_float_value) => Some(<(FloatValue,)>::to_gtk_box),
            Self::MoveCursorToCorner(_cursor_corner) => Some(<(CursorCorner,)>::to_gtk_box),
            Self::MoveCursor(_x, _y) => Some(<(u32, u32)>::to_gtk_box),
            Self::RenameWorkspace(_id, _new_name) => Some(<(u32, String)>::to_gtk_box),
            Self::Exit => None,
            Self::ForceRendererReload => None,
            Self::MoveCurrentWorkspaceToMonitor(_monitor_target) => {
                Some(<(MonitorTarget,)>::to_gtk_box)
            }
            Self::FocusWorkspaceOnCurrentMonitor(_workspace_target) => {
                Some(<(WorkspaceTarget,)>::to_gtk_box)
            }
            Self::MoveWorkspaceToMonitor(_workspace_target, _monitor_target) => {
                Some(<(WorkspaceTarget, MonitorTarget)>::to_gtk_box)
            }
            Self::SwapActiveWorkspaces(_monitor_target1, _monitor_target2) => {
                Some(<(MonitorTarget, MonitorTarget)>::to_gtk_box)
            }
            Self::BringActiveToTop => None,
            Self::AlterZOrder(_z_height, _optional_window_target) => {
                Some(|entry, separator, _names| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let z_height_entry = create_entry();
                    let z_height_box = ZHeight::to_gtk_box(&z_height_entry);
                    mother_box.append(&z_height_box);

                    let window_target_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    window_target_box.append(&Label::new(Some(&t!("window_target"))));
                    let optional_window_target_entry = create_entry();
                    let optional_window_target_box =
                        Option::<WindowTarget>::to_gtk_box(&optional_window_target_entry);
                    window_target_box.append(&optional_window_target_box);
                    mother_box.append(&window_target_box);

                    let z_height_entry_clone = z_height_entry.clone();
                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let update_ui = move |(z_height, optional_window_target): (
                        ZHeight,
                        Option<WindowTarget>,
                    )| {
                        z_height_entry_clone.set_text(&z_height.to_string());
                        let window_target_str = match optional_window_target {
                            Some(window_target) => window_target.to_string(),
                            None => String::new(),
                        };
                        optional_window_target_entry_clone.set_text(&window_target_str);
                    };

                    let parse_value = |str: &str| {
                        let dispatcher = Self::from_discriminant_and_str(
                            DispatcherDiscriminant::AlterZOrder,
                            str,
                        );
                        match dispatcher {
                            Dispatcher::AlterZOrder(z_height, optional_window_target) => {
                                (z_height, optional_window_target)
                            }
                            _ => (ZHeight::default(), None),
                        }
                    };

                    update_ui(parse_value(entry.text().as_str()));

                    let optional_window_target_entry_clone = optional_window_target_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    z_height_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match optional_window_target_entry_clone.text().as_str() {
                            "" => entry.text().to_string(),
                            window_target => {
                                format!("{}{}{}", entry.text(), separator, window_target)
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let z_height_entry_clone = z_height_entry.clone();
                    let entry_clone = entry.clone();
                    let is_updating_clone = is_updating.clone();
                    optional_window_target_entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let new_text = match entry.text().as_str() {
                            "" => z_height_entry_clone.text().to_string(),
                            window_target => {
                                format!(
                                    "{}{}{}",
                                    z_height_entry_clone.text(),
                                    separator,
                                    window_target
                                )
                            }
                        };
                        entry_clone.set_text(&new_text);
                        is_updating_clone.set(false);
                    });

                    let is_updating_clone = is_updating.clone();
                    entry.connect_changed(move |entry| {
                        if is_updating_clone.get() {
                            return;
                        }
                        is_updating_clone.set(true);
                        let (z_height, optional_window_target) = parse_value(entry.text().as_str());
                        update_ui((z_height, optional_window_target));
                        is_updating_clone.set(false);
                    });

                    mother_box
                })
            }
            Self::ToggleSpecialWorkspace(_optional_name) => {
                Some(|entry, _separator, _names| Option::<String>::to_gtk_box(entry))
            }
            Self::FocusUrgentOrLast => None,
            Self::ToggleGroup => None,
            Self::ChangeGroupActive(_change_group_active) => {
                Some(<(ChangeGroupActive,)>::to_gtk_box)
            }
            Self::FocusCurrentOrLast => None,
            Self::LockGroups(_group_lock_action) => Some(<(GroupLockAction,)>::to_gtk_box),
            Self::LockActiveGroup(_group_lock_action) => Some(<(GroupLockAction,)>::to_gtk_box),
            Self::MoveIntoGroup(_direction) => Some(<(Direction,)>::to_gtk_box),
            Self::MoveOutOfGroup(_optional_window_target) => {
                Some(|entry, _separator, _names| Option::<WindowTarget>::to_gtk_box(entry))
            }
            Self::MoveWindowOrGroup(_direction) => Some(<(Direction,)>::to_gtk_box),
            Self::MoveGroupWindow(_is_back) => Some(<(bool,)>::to_gtk_box),
            Self::DenyWindowFromGroup(_toggle_state) => Some(<(ToggleState,)>::to_gtk_box),
            Self::SetIgnoreGroupLock(_toggle_state) => Some(<(ToggleState,)>::to_gtk_box),
            Self::Global(_name) => Some(<(String,)>::to_gtk_box),
            Self::Event(_data) => Some(<(String,)>::to_gtk_box),
            Self::SetProp(_set_prop) => Some(<(SetProp,)>::to_gtk_box),
            Self::ToggleSwallow => None,
        }
    }

    fn field_labels() -> Option<Vec<Vec<FieldLabel>>> {
        Some(vec![
            // Exec(Vec<WindowRule>, String),
            vec![],
            // Execr(String),
            vec![FieldLabel::Unnamed],
            // Pass(WindowTarget),
            vec![FieldLabel::Unnamed],
            // SendShortcut(HashSet<Modifier>, String, Option<WindowTarget>),
            vec![],
            // SendKeyState(HashSet<Modifier>, String, KeyState, WindowTarget),
            vec![],
            // KillActive,
            vec![],
            // ForceKillActive,
            vec![],
            // CloseWindow(WindowTarget),
            vec![FieldLabel::Unnamed],
            // KillWindow(WindowTarget),
            vec![FieldLabel::Unnamed],
            // Signal(String),
            vec![FieldLabel::Unnamed],
            // SignalWindow(WindowTarget, String),
            vec![FieldLabel::Unnamed, FieldLabel::Unnamed],
            // Workspace(WorkspaceTarget),
            vec![FieldLabel::Unnamed],
            // MoveToWorkspace(WorkspaceTarget, Option<WindowTarget>),
            vec![],
            // MoveToWorkspaceSilent(WorkspaceTarget, Option<WindowTarget>),
            vec![],
            // ToggleFloating(Option<WindowTarget>),
            vec![],
            // SetFloating(Option<WindowTarget>),
            vec![],
            // SetTiled(Option<WindowTarget>),
            vec![],
            // Fullscreen(FullscreenMode),
            vec![FieldLabel::Named(cow_to_static_str(t!("fullscreen_mode")))],
            // FullscreenState(DispatcherFullscreenState, DispatcherFullscreenState),
            vec![
                FieldLabel::Named(cow_to_static_str(t!("internal_state"))),
                FieldLabel::Named(cow_to_static_str(t!("client_state"))),
            ],
            // other options does not need to be labelled
        ])
    }
}

// ToGtkBox
inventory::submit! {
    ToGtkBoxImplementation(<() as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<String as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<u8 as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<u32 as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<i32 as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<bool as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<Direction as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<MonitorTarget as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<PixelOrPercent as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<ResizeParams as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<FloatValue as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<ZHeight as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<FullscreenMode as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<RelativeId as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<WorkspaceTarget as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<WindowTarget as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<CursorCorner as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<GroupLockAction as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<ToggleState as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<FullscreenState as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<IdOrName as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<WindowGroupOption as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<WindowEvent as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<ContentType as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<HyprColor as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<IdleIngibitMode as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<HyprOpacity as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<Side as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<AnimationStyle as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<TagToggleState as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<WindowRule as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<KeyState as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<DispatcherFullscreenState as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<MoveDirection as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<SwapDirection as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<SwapNext as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<ChangeGroupActive as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<SetPropToggleState as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<SetProp as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<Modifier as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<Dispatcher as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<HyprCoord as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<HyprSize as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<Angle as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<BorderColor as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<CycleNext as ToGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxImplementation(<HyprGradient as ToGtkBox>::to_gtk_box)
}

// ToOptionalGtkBox
inventory::submit! {
    ToOptionalGtkBoxImplementation(<Option<Direction> as ToOptionalGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToOptionalGtkBoxImplementation(<Option<WindowTarget> as ToOptionalGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToOptionalGtkBoxImplementation(<Option<Angle> as ToOptionalGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToOptionalGtkBoxImplementation(<Option<HyprGradient> as ToOptionalGtkBox>::to_gtk_box)
}
inventory::submit! {
    ToOptionalGtkBoxImplementation(<Option<String> as ToOptionalGtkBox>::to_gtk_box)
}

// ToGtkBoxWithSeparator
inventory::submit! {
    ToGtkBoxWithSeparatorImplementation(<Vec<HyprColor> as ToGtkBoxWithSeparator>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorImplementation(<Vec<WindowRule> as ToGtkBoxWithSeparator>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorImplementation(<Vec<WindowGroupOption> as ToGtkBoxWithSeparator>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorImplementation(<HashSet<Modifier> as ToGtkBoxWithSeparator>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorImplementation(<HashSet<WindowEvent> as ToGtkBoxWithSeparator>::to_gtk_box)
}

// ToGtkBoxWithSeparatorAndNames
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(Direction,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(u32,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(i32,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(String,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(PixelOrPercent,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(PixelOrPercent, PixelOrPercent) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(FloatValue,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(WindowTarget, String) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(WorkspaceTarget, MonitorTarget) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(MonitorTarget, MonitorTarget) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(TagToggleState, String) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(u32, String) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(u32, u32) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(u8, u8, u8) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(u8, u8, u8, u8) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(FullscreenState, FullscreenState) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(DispatcherFullscreenState, DispatcherFullscreenState) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(ResizeParams, WindowTarget) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(ToggleState,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(GroupLockAction,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(ChangeGroupActive,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(CycleNext,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(SwapNext,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(SetPropToggleState,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
inventory::submit! {
    ToGtkBoxWithSeparatorAndNamesImplementation(<(KeyState,) as ToGtkBoxWithSeparatorAndNames>::to_gtk_box)
}
