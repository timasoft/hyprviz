use super::{GestureAction, GestureDirection, Modifier};
use crate::{
    advanced_editors::{create_entry, create_spin_button, create_switch},
    gtk_converters::{ToGtkBox, ToGtkBoxWithSeparator},
    hyprland::modifier::parse_modifiers,
    register_togtkbox, register_togtkbox_with_separator,
    utils::{MAX_SAFE_STEP_0_01_F64, join_with_separator},
};
use gtk::{Box as GtkBox, Entry, Label, Orientation as GtkOrientation, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, collections::HashSet, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub struct Gesture {
    pub finger_count: u32,
    pub direction: GestureDirection,
    pub action: GestureAction,
    pub anim_speed: Option<f64>,
    pub mods: Option<HashSet<Modifier>>,
}

impl Default for Gesture {
    fn default() -> Self {
        Gesture {
            finger_count: 3,
            direction: GestureDirection::Swipe,
            action: GestureAction::default(),
            anim_speed: None,
            mods: None,
        }
    }
}

impl FromStr for Gesture {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();

        let finger_count = match parts.first().unwrap_or(&"").trim().parse::<u32>() {
            Ok(0) | Ok(1) | Ok(2) | Ok(3) => 3,
            Ok(finger_count) => finger_count,
            _ => 3,
        };

        let direction = parts
            .get(1)
            .unwrap_or(&"")
            .parse::<GestureDirection>()
            .unwrap_or_default();

        let mut action = GestureAction::default();

        let mut anim_speed = None;

        let mut mods = None;

        for (i, part) in parts.iter().enumerate().skip(2) {
            if let Some(stripped) = part.trim().strip_prefix("mod:") {
                mods = Some(parse_modifiers(stripped));
            } else if let Some(stripped) = part.trim().strip_prefix("scale:")
                && let Ok(speed) = stripped.parse::<f64>()
            {
                anim_speed = Some(speed);
            } else if let Ok(gesture_action) =
                GestureAction::from_str(&format!("{}, {}", part, parts.get(i + 1).unwrap_or(&"")))
            {
                action = gesture_action;
            }
        }

        Ok(Gesture {
            finger_count,
            direction,
            action,
            anim_speed,
            mods,
        })
    }
}

impl Display for Gesture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.anim_speed, &self.mods) {
            (Some(speed), Some(mods)) => write!(
                f,
                "{}, {}, mod:{}, scale:{}, {}",
                self.finger_count,
                self.direction,
                join_with_separator(mods, "_"),
                speed,
                self.action
            ),
            (Some(speed), None) => write!(
                f,
                "{}, {}, scale:{}, {}",
                self.finger_count, self.direction, speed, self.action
            ),
            (None, Some(mods)) => write!(
                f,
                "{}, {}, mod:{}, {}",
                self.finger_count,
                self.direction,
                join_with_separator(mods, "_"),
                self.action
            ),
            (None, None) => write!(
                f,
                "{}, {}, {}",
                self.finger_count, self.direction, self.action
            ),
        }
    }
}

impl ToGtkBox for Gesture {
    fn to_gtk_box(entry: &Entry) -> GtkBox {
        let is_updating = Rc::new(Cell::new(false));
        let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

        let finger_count_box = GtkBox::new(GtkOrientation::Vertical, 5);
        finger_count_box.append(&Label::new(Some(&t!("hyprland.gesture.finger_count"))));
        let finger_count_spin = create_spin_button(1.0, i32::MAX as f64, 1.0);
        finger_count_box.append(&finger_count_spin);
        mother_box.append(&finger_count_box);

        let direction_box = GtkBox::new(GtkOrientation::Vertical, 5);
        direction_box.append(&Label::new(Some(&t!("hyprland.gesture.direction"))));
        let direction_entry = create_entry();
        let direction_ui = GestureDirection::to_gtk_box(&direction_entry);
        direction_box.append(&direction_ui);
        mother_box.append(&direction_box);

        let action_box = GtkBox::new(GtkOrientation::Vertical, 5);
        action_box.append(&Label::new(Some(&t!("hyprland.gesture.action"))));
        let action_entry = create_entry();
        let action_ui = GestureAction::to_gtk_box(&action_entry);
        action_box.append(&action_ui);
        mother_box.append(&action_box);

        let anim_speed_mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let anim_speed_switch_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        anim_speed_switch_box.append(&Label::new(Some(&t!("hyprland.gesture.animation_speed"))));
        let anim_speed_switch = create_switch();
        anim_speed_switch_box.append(&anim_speed_switch);
        anim_speed_mother_box.append(&anim_speed_switch_box);

        let anim_speed_value_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        let anim_speed_spin = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
        anim_speed_spin.set_value(1.0);
        anim_speed_value_box.append(&anim_speed_spin);
        anim_speed_value_box.set_visible(false);
        anim_speed_mother_box.append(&anim_speed_value_box);
        mother_box.append(&anim_speed_mother_box);

        let mods_mother_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let mods_switch_box = GtkBox::new(GtkOrientation::Horizontal, 5);
        mods_switch_box.append(&Label::new(Some(&t!("hyprland.gesture.modifiers"))));
        let mods_switch = create_switch();
        mods_switch_box.append(&mods_switch);
        mods_mother_box.append(&mods_switch_box);

        let mods_value_box = GtkBox::new(GtkOrientation::Vertical, 5);
        let mods_entry = create_entry();
        let mods_ui = HashSet::<Modifier>::to_gtk_box(&mods_entry, '_');
        mods_value_box.append(&mods_ui);
        mods_value_box.set_visible(false);
        mods_mother_box.append(&mods_value_box);
        mother_box.append(&mods_mother_box);

        let finger_count_spin_clone = finger_count_spin.clone();
        let direction_entry_clone = direction_entry.clone();
        let action_entry_clone = action_entry.clone();
        let anim_speed_switch_clone = anim_speed_switch.clone();
        let anim_speed_spin_clone = anim_speed_spin.clone();
        let anim_speed_value_box_clone = anim_speed_value_box.clone();
        let mods_switch_clone = mods_switch.clone();
        let mods_entry_clone = mods_entry.clone();
        let mods_value_box_clone = mods_value_box.clone();
        let update_ui = move |gesture: Gesture| {
            finger_count_spin_clone.set_value(gesture.finger_count as f64);
            direction_entry_clone.set_text(&gesture.direction.to_string());
            action_entry_clone.set_text(&gesture.action.to_string());

            if let Some(speed) = gesture.anim_speed {
                anim_speed_switch_clone.set_active(true);
                anim_speed_value_box_clone.set_visible(true);
                anim_speed_spin_clone.set_value(speed);
            } else {
                anim_speed_switch_clone.set_active(false);
                anim_speed_value_box_clone.set_visible(false);
            }

            if let Some(mods) = gesture.mods {
                mods_switch_clone.set_active(true);
                mods_value_box_clone.set_visible(true);
                mods_entry_clone.set_text(&join_with_separator(&mods, "_"));
            } else {
                mods_switch_clone.set_active(false);
                mods_value_box_clone.set_visible(false);
                mods_entry_clone.set_text("");
            }
        };

        let initial_gesture: Gesture = entry.text().to_string().parse().unwrap_or_default();
        update_ui(initial_gesture);

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        finger_count_spin.connect_value_changed(move |spin| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut gesture: Gesture = entry_clone.text().parse().unwrap_or_default();
            gesture.finger_count = spin.value() as u32;
            entry_clone.set_text(&gesture.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        direction_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut gesture: Gesture = entry_clone.text().parse().unwrap_or_default();
            gesture.direction = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&gesture.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        action_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut gesture: Gesture = entry_clone.text().parse().unwrap_or_default();
            gesture.action = entry.text().parse().unwrap_or_default();
            entry_clone.set_text(&gesture.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        let anim_speed_spin_clone = anim_speed_spin.clone();
        let anim_speed_value_box_clone = anim_speed_value_box.clone();
        anim_speed_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut gesture: Gesture = entry_clone.text().parse().unwrap_or_default();
            if switch.state() {
                gesture.anim_speed = Some(anim_speed_spin_clone.value());
                anim_speed_value_box_clone.set_visible(true);
            } else {
                gesture.anim_speed = None;
                anim_speed_value_box_clone.set_visible(false);
            }
            entry_clone.set_text(&gesture.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        anim_speed_spin.connect_value_changed(move |spin| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut gesture: Gesture = entry_clone.text().parse().unwrap_or_default();
            gesture.anim_speed = Some(spin.value());
            entry_clone.set_text(&gesture.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        let mods_entry_clone = mods_entry.clone();
        let mods_value_box_clone = mods_value_box.clone();
        mods_switch.connect_state_notify(move |switch| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut gesture: Gesture = entry_clone.text().parse().unwrap_or_default();
            if switch.state() {
                let mods: HashSet<Modifier> = mods_entry_clone
                    .text()
                    .split('_')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse().unwrap_or_default())
                    .collect();
                gesture.mods = if mods.is_empty() { None } else { Some(mods) };
                mods_value_box_clone.set_visible(true);
            } else {
                gesture.mods = None;
                mods_value_box_clone.set_visible(false);
            }
            entry_clone.set_text(&gesture.to_string());
            is_updating_clone.set(false);
        });

        let entry_clone = entry.clone();
        let is_updating_clone = is_updating.clone();
        mods_entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let mut gesture: Gesture = entry_clone.text().parse().unwrap_or_default();
            let mods: HashSet<Modifier> = parse_modifiers(entry.text().as_str());
            gesture.mods = if mods.is_empty() { None } else { Some(mods) };
            entry_clone.set_text(&gesture.to_string());
            is_updating_clone.set(false);
        });

        let is_updating_clone = is_updating.clone();
        let update_ui_clone = update_ui.clone();
        entry.connect_changed(move |entry| {
            if is_updating_clone.get() {
                return;
            }
            is_updating_clone.set(true);
            let gesture: Gesture = entry.text().parse().unwrap_or_default();
            update_ui_clone(gesture);
            is_updating_clone.set(false);
        });

        mother_box
    }
}

register_togtkbox!(Gesture);
register_togtkbox_with_separator!(HashSet<Modifier>);
