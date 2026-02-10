use crate::{
    advanced_editors::{create_spin_button, create_switch},
    gtk_converters::{EnumConfigForGtk, ToGtkBoxWithSeparatorAndNamesBuilder},
    register_togtkbox,
    utils::{HasDiscriminant, MAX_SAFE_STEP_0_01_F64},
};
use gtk::{Box as GtkBox, Label, Orientation as GtkOrientation, StringList, prelude::*};
use rust_i18n::t;
use std::{cell::Cell, fmt::Display, rc::Rc, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(HyprOpacityDiscriminant))]
pub enum HyprOpacity {
    Overall(f64, bool),
    ActiveAndInactive(f64, bool, f64, bool),
    ActiveAndInactiveAndFullscreen(f64, bool, f64, bool, f64, bool),
}

impl HasDiscriminant for HyprOpacity {
    type Discriminant = HyprOpacityDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Overall => Self::Overall(1.0, false),
            Self::Discriminant::ActiveAndInactive => {
                Self::ActiveAndInactive(1.0, false, 1.0, false)
            }
            Self::Discriminant::ActiveAndInactiveAndFullscreen => {
                Self::ActiveAndInactiveAndFullscreen(1.0, false, 1.0, false, 1.0, false)
            }
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Overall => {
                let parts: Vec<&str> = str.split_whitespace().collect();
                if parts.len() >= 2 && parts[1].to_lowercase() == "override" {
                    let opacity = parts[0].parse::<f64>().unwrap_or(1.0);
                    Self::Overall(opacity, true)
                } else {
                    let opacity = parts
                        .first()
                        .unwrap_or(&"1.0")
                        .parse::<f64>()
                        .unwrap_or(1.0);
                    Self::Overall(opacity, false)
                }
            }
            Self::Discriminant::ActiveAndInactive => {
                let parts: Vec<&str> = str.split_whitespace().collect();
                match parts.len() {
                    0 => Self::ActiveAndInactive(1.0, false, 1.0, false),
                    1 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactive(opacity1, false, 1.0, false)
                    }
                    2 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactive(opacity1, false, opacity2, false)
                    }
                    3 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        let is_override2 = parts[2].to_lowercase() == "override";
                        Self::ActiveAndInactive(opacity1, false, opacity2, is_override2)
                    }
                    4 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactive(opacity1, true, opacity2, true)
                    }
                    _ => Self::ActiveAndInactive(1.0, false, 1.0, false),
                }
            }
            Self::Discriminant::ActiveAndInactiveAndFullscreen => {
                let parts: Vec<&str> = str.split_whitespace().collect();
                match parts.len() {
                    0 => Self::ActiveAndInactiveAndFullscreen(1.0, false, 1.0, false, 1.0, false),
                    1 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactiveAndFullscreen(
                            opacity1, false, 1.0, false, 1.0, false,
                        )
                    }
                    2 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactiveAndFullscreen(
                            opacity1, false, opacity2, false, 1.0, false,
                        )
                    }
                    3 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        let opacity3 = parts[2].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactiveAndFullscreen(
                            opacity1, false, opacity2, false, opacity3, false,
                        )
                    }
                    _ => {
                        let mut opacities = Vec::new();
                        let mut overrides = Vec::new();

                        for part in &parts {
                            if part.to_lowercase() == "override" {
                                if let Some(last) = overrides.last_mut() {
                                    *last = true;
                                }
                            } else if let Ok(opacity) = part.parse::<f64>() {
                                opacities.push(opacity);
                                overrides.push(false);
                            }
                        }

                        match (opacities.len(), overrides.len()) {
                            (3, 3) => Self::ActiveAndInactiveAndFullscreen(
                                opacities[0],
                                overrides[0],
                                opacities[1],
                                overrides[1],
                                opacities[2],
                                overrides[2],
                            ),
                            (3, 2) => Self::ActiveAndInactiveAndFullscreen(
                                opacities[0],
                                overrides[0],
                                opacities[1],
                                overrides[1],
                                opacities[2],
                                false,
                            ),
                            (2, 2) => Self::ActiveAndInactiveAndFullscreen(
                                opacities[0],
                                overrides[0],
                                opacities[1],
                                overrides[1],
                                1.0,
                                false,
                            ),
                            (1, 1) => Self::ActiveAndInactiveAndFullscreen(
                                opacities[0],
                                overrides[0],
                                1.0,
                                false,
                                1.0,
                                false,
                            ),
                            _ => Self::ActiveAndInactiveAndFullscreen(
                                1.0, false, 1.0, false, 1.0, false,
                            ),
                        }
                    }
                }
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            HyprOpacity::Overall(opacity, is_override) => {
                if *is_override {
                    Some(format!("{} override", opacity))
                } else {
                    Some(opacity.to_string())
                }
            }
            HyprOpacity::ActiveAndInactive(opacity1, is_override1, opacity2, is_override2) => {
                let mut parts = Vec::new();
                parts.push(opacity1.to_string());
                if *is_override1 {
                    parts.push("override".to_string());
                }
                parts.push(opacity2.to_string());
                if *is_override2 {
                    parts.push("override".to_string());
                }
                Some(parts.join(" "))
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                is_override1,
                opacity2,
                is_override2,
                opacity3,
                is_override3,
            ) => {
                let mut parts = Vec::new();
                parts.push(opacity1.to_string());
                if *is_override1 {
                    parts.push("override".to_string());
                }
                parts.push(opacity2.to_string());
                if *is_override2 {
                    parts.push("override".to_string());
                }
                parts.push(opacity3.to_string());
                if *is_override3 {
                    parts.push("override".to_string());
                }
                Some(parts.join(" "))
            }
        }
    }
}

impl Default for HyprOpacity {
    fn default() -> Self {
        HyprOpacity::Overall(1.0, false)
    }
}

impl FromStr for HyprOpacity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let parts: Vec<&str> = s.split(' ').collect();
        if parts.len() == 1 {
            // Overall
            let opacity = parts[0].parse::<f64>().unwrap_or_default();
            Ok(HyprOpacity::Overall(opacity, false))
        } else if parts.len() == 2 {
            // Active and Inactive or Active override
            let opacity1 = parts[0].parse::<f64>().unwrap_or_default();
            match parts[1].trim().to_lowercase().as_str() {
                "override" => Ok(HyprOpacity::Overall(opacity1, true)),
                opacity2 => Ok(HyprOpacity::ActiveAndInactive(
                    opacity1,
                    false,
                    opacity2.parse::<f64>().unwrap_or_default(),
                    false,
                )),
            }
        } else if parts.len() == 3 {
            // A - Active
            // I - Inactive
            // F - Fullscreen
            // o - Override
            // variants for 3 parts: AoI, AIo, AIF
            let opacity1 = parts[0].parse::<f64>().unwrap_or_default();
            match parts[1].trim().to_lowercase().as_str() {
                "override" => Ok(HyprOpacity::ActiveAndInactive(
                    opacity1,
                    true,
                    parts[2].parse::<f64>().unwrap_or_default(),
                    false,
                )),
                opacity2 => {
                    let opacity2 = opacity2.parse::<f64>().unwrap_or_default();
                    match parts[2].trim().to_lowercase().as_str() {
                        "override" => Ok(HyprOpacity::ActiveAndInactive(
                            opacity1, false, opacity2, true,
                        )),
                        opacity3 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1,
                            false,
                            opacity2,
                            false,
                            opacity3.parse::<f64>().unwrap_or_default(),
                            false,
                        )),
                    }
                }
            }
        } else if parts.len() == 4 {
            // A - Active
            // I - Inactive
            // F - Fullscreen
            // o - Override
            // variants for 4 parts: AoIo, AoIF, AIoF, AIFo
            let opacity1 = parts[0].parse::<f64>().unwrap_or_default();
            match parts[1].trim().to_lowercase().as_str() {
                "override" => match parts[3].trim().to_lowercase().as_str() {
                    "override" => Ok(HyprOpacity::ActiveAndInactive(
                        opacity1,
                        true,
                        parts[2].parse::<f64>().unwrap_or_default(),
                        false,
                    )),
                    opacity3 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                        opacity1,
                        true,
                        parts[2].parse::<f64>().unwrap_or_default(),
                        false,
                        opacity3.parse::<f64>().unwrap_or_default(),
                        false,
                    )),
                },
                opacity2 => {
                    let opacity2 = opacity2.parse::<f64>().unwrap_or_default();
                    match parts[2].trim().to_lowercase().as_str() {
                        "override" => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1,
                            false,
                            opacity2,
                            true,
                            parts[3].parse::<f64>().unwrap_or_default(),
                            false,
                        )),
                        opacity3 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1,
                            false,
                            opacity2,
                            false,
                            opacity3.parse::<f64>().unwrap_or_default(),
                            true,
                        )),
                    }
                }
            }
        } else if parts.len() == 5 {
            // A - Active
            // I - Inactive
            // F - Fullscreen
            // o - Override
            // variants for 5 parts: AoIoF, AoIFo, AIoFo
            let opacity1 = parts[0].parse::<f64>().unwrap_or_default();
            match parts[1].trim().to_lowercase().as_str() {
                "override" => match parts[3].trim().to_lowercase().as_str() {
                    "override" => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                        opacity1,
                        true,
                        parts[2].parse::<f64>().unwrap_or_default(),
                        true,
                        parts[4].parse::<f64>().unwrap_or_default(),
                        false,
                    )),
                    opacity3 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                        opacity1,
                        true,
                        parts[2].parse::<f64>().unwrap_or_default(),
                        false,
                        opacity3.parse::<f64>().unwrap_or_default(),
                        true,
                    )),
                },
                opacity2 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                    opacity1,
                    false,
                    opacity2.parse::<f64>().unwrap_or_default(),
                    true,
                    parts[3].parse::<f64>().unwrap_or_default(),
                    true,
                )),
            }
        } else if parts.len() == 6 {
            // A - Active
            // I - Inactive
            // F - Fullscreen
            // o - Override
            // variants for 6 parts: AoIoFo
            Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                parts[0].parse::<f64>().unwrap_or_default(),
                true,
                parts[2].parse::<f64>().unwrap_or_default(),
                true,
                parts[4].parse::<f64>().unwrap_or_default(),
                true,
            ))
        } else {
            Err(())
        }
    }
}

impl Display for HyprOpacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprOpacity::Overall(opacity, false) => write!(f, "{}", opacity),
            HyprOpacity::Overall(opacity, true) => write!(f, "{} override", opacity),
            HyprOpacity::ActiveAndInactive(opacity1, false, opacity2, false) => {
                write!(f, "{} {}", opacity1, opacity2)
            }
            HyprOpacity::ActiveAndInactive(opacity1, true, opacity2, false) => {
                write!(f, "{} override {}", opacity1, opacity2)
            }
            HyprOpacity::ActiveAndInactive(opacity1, false, opacity2, true) => {
                write!(f, "{} {} override", opacity1, opacity2)
            }
            HyprOpacity::ActiveAndInactive(opacity1, true, opacity2, true) => {
                write!(f, "{} override {} override", opacity1, opacity2)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                false,
                opacity2,
                false,
                opacity3,
                false,
            ) => {
                write!(f, "{} {} {}", opacity1, opacity2, opacity3)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                true,
                opacity2,
                false,
                opacity3,
                false,
            ) => {
                write!(f, "{} override {} {}", opacity1, opacity2, opacity3)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                false,
                opacity2,
                true,
                opacity3,
                false,
            ) => {
                write!(f, "{} {} override {}", opacity1, opacity2, opacity3)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                true,
                opacity2,
                true,
                opacity3,
                false,
            ) => {
                write!(
                    f,
                    "{} override {} override {}",
                    opacity1, opacity2, opacity3
                )
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                false,
                opacity2,
                false,
                opacity3,
                true,
            ) => {
                write!(f, "{} {} {} override", opacity1, opacity2, opacity3)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                true,
                opacity2,
                false,
                opacity3,
                true,
            ) => {
                write!(
                    f,
                    "{} override {} {} override",
                    opacity1, opacity2, opacity3
                )
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                false,
                opacity2,
                true,
                opacity3,
                true,
            ) => {
                write!(
                    f,
                    "{} {} override {} override",
                    opacity1, opacity2, opacity3
                )
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                true,
                opacity2,
                true,
                opacity3,
                true,
            ) => {
                write!(
                    f,
                    "{} override {} override {} override",
                    opacity1, opacity2, opacity3
                )
            }
        }
    }
}

impl EnumConfigForGtk for HyprOpacity {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.overall"),
            &t!("gtk_converters.active_and_inactive"),
            &t!("gtk_converters.active_and_inactive_and_fullscreen"),
        ])
    }

    const SEPARATOR: Option<char> = Some(' ');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            HyprOpacity::Overall(_opacity, _override) => Some(|entry, _separator, _labels, _| {
                let is_updating = Rc::new(Cell::new(false));
                let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                let opacity_spin_button = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                mother_box.append(&opacity_spin_button);

                let override_box = GtkBox::new(GtkOrientation::Vertical, 5);
                override_box.append(&Label::new(Some(&t!("gtk_converters.override"))));
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
                Some(|entry, _separator, _labels, _| {
                    let is_updating = Rc::new(Cell::new(false));
                    let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                    let opacity1_spin_button =
                        create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                    mother_box.append(&opacity1_spin_button);

                    let override1_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    override1_box.append(&Label::new(Some(&t!("gtk_converters.override"))));
                    let override1_switch = create_switch();
                    override1_box.append(&override1_switch);
                    mother_box.append(&override1_box);

                    let opacity2_spin_button =
                        create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                    mother_box.append(&opacity2_spin_button);

                    let override2_box = GtkBox::new(GtkOrientation::Vertical, 5);
                    override2_box.append(&Label::new(Some(&t!("gtk_converters.override"))));
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
            ) => Some(|entry, _separator, _labels, _| {
                let is_updating = Rc::new(Cell::new(false));
                let mother_box = GtkBox::new(GtkOrientation::Horizontal, 5);

                let opacity1_spin_button = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                mother_box.append(&opacity1_spin_button);

                let override1_box = GtkBox::new(GtkOrientation::Vertical, 5);
                override1_box.append(&Label::new(Some(&t!("gtk_converters.override"))));
                let override1_switch = create_switch();
                override1_box.append(&override1_switch);
                mother_box.append(&override1_box);

                let opacity2_spin_button = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                mother_box.append(&opacity2_spin_button);

                let override2_box = GtkBox::new(GtkOrientation::Vertical, 5);
                override2_box.append(&Label::new(Some(&t!("gtk_converters.override"))));
                let override2_switch = create_switch();
                override2_box.append(&override2_switch);
                mother_box.append(&override2_box);

                let opacity3_spin_button = create_spin_button(0.0, MAX_SAFE_STEP_0_01_F64, 0.01);
                mother_box.append(&opacity3_spin_button);

                let override3_box = GtkBox::new(GtkOrientation::Vertical, 5);
                override3_box.append(&Label::new(Some(&t!("gtk_converters.override"))));
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

register_togtkbox!(HyprOpacity);
