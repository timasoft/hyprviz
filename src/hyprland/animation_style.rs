use super::Side;
use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder, create_spin_button_builder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Default, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(AnimationStyleDiscriminant))]
pub enum AnimationStyle {
    #[default]
    None,
    Slide,
    SlideSide(Side),
    SlidePercent(f64),
    Popin,
    PopinPercent(f64),
    Gnomed,
    SlideVert,
    SlideVertPercent(f64),
    Fade,
    SlideFade,
    SlideFadePercent(f64),
    SlideFadeVert,
    SlideFadeVertPercent(f64),
    Once,
    Loop,
}

impl HasDiscriminant for AnimationStyle {
    type Discriminant = AnimationStyleDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::None => Self::None,
            Self::Discriminant::Slide => Self::Slide,
            Self::Discriminant::SlideSide => Self::SlideSide(Side::default()),
            Self::Discriminant::SlidePercent => Self::SlidePercent(0.0),
            Self::Discriminant::Popin => Self::Popin,
            Self::Discriminant::PopinPercent => Self::PopinPercent(0.0),
            Self::Discriminant::Gnomed => Self::Gnomed,
            Self::Discriminant::SlideVert => Self::SlideVert,
            Self::Discriminant::SlideVertPercent => Self::SlideVertPercent(0.0),
            Self::Discriminant::Fade => Self::Fade,
            Self::Discriminant::SlideFade => Self::SlideFade,
            Self::Discriminant::SlideFadePercent => Self::SlideFadePercent(0.0),
            Self::Discriminant::SlideFadeVert => Self::SlideFadeVert,
            Self::Discriminant::SlideFadeVertPercent => Self::SlideFadeVertPercent(0.0),
            Self::Discriminant::Once => Self::Once,
            Self::Discriminant::Loop => Self::Loop,
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        let s = str.trim();
        match discriminant {
            Self::Discriminant::None => Self::None,
            Self::Discriminant::Slide => Self::Slide,
            Self::Discriminant::SlideSide => Self::SlideSide(Side::from_str(s).unwrap_or_default()),
            Self::Discriminant::SlidePercent => {
                Self::SlidePercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::Popin => Self::Popin,
            Self::Discriminant::PopinPercent => {
                Self::PopinPercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::Gnomed => Self::Gnomed,
            Self::Discriminant::SlideVert => Self::SlideVert,
            Self::Discriminant::SlideVertPercent => {
                Self::SlideVertPercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::Fade => Self::Fade,
            Self::Discriminant::SlideFade => Self::SlideFade,
            Self::Discriminant::SlideFadePercent => {
                Self::SlideFadePercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::SlideFadeVert => Self::SlideFadeVert,
            Self::Discriminant::SlideFadeVertPercent => {
                Self::SlideFadeVertPercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::Once => Self::Once,
            Self::Discriminant::Loop => Self::Loop,
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            AnimationStyle::None => None,
            AnimationStyle::Slide => None,
            AnimationStyle::SlideSide(side) => Some(side.to_string()),
            AnimationStyle::SlidePercent(percent) => Some(percent.to_string()),
            AnimationStyle::Popin => None,
            AnimationStyle::PopinPercent(percent) => Some(percent.to_string()),
            AnimationStyle::Gnomed => None,
            AnimationStyle::SlideVert => None,
            AnimationStyle::SlideVertPercent(percent) => Some(percent.to_string()),
            AnimationStyle::Fade => None,
            AnimationStyle::SlideFade => None,
            AnimationStyle::SlideFadePercent(percent) => Some(percent.to_string()),
            AnimationStyle::SlideFadeVert => None,
            AnimationStyle::SlideFadeVertPercent(percent) => Some(percent.to_string()),
            AnimationStyle::Once => None,
            AnimationStyle::Loop => None,
        }
    }
}

impl FromStr for AnimationStyle {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        s.split_whitespace()
            .next()
            .map_or(Ok(AnimationStyle::None), |first| match first {
                "slide" => {
                    let remainder = s.strip_prefix("slide").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::Slide)
                    } else {
                        match Side::from_str(remainder) {
                            Ok(side) => Ok(AnimationStyle::SlideSide(side)),
                            Err(_) => match f64::from_str(remainder.trim_end_matches('%')) {
                                Ok(percent) => Ok(AnimationStyle::SlidePercent(percent)),
                                Err(_) => Ok(AnimationStyle::None),
                            },
                        }
                    }
                }
                "popin" => {
                    let remainder = s.strip_prefix("popin").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::Popin)
                    } else {
                        let percent = remainder.trim_end_matches('%');
                        match percent.parse::<f64>() {
                            Ok(percent) => Ok(AnimationStyle::PopinPercent(percent)),
                            Err(_) => Err(()),
                        }
                    }
                }
                "gnomed" => Ok(AnimationStyle::Gnomed),
                "slidevert" => {
                    let remainder = s.strip_prefix("slidevert").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::SlideVert)
                    } else {
                        let percent = remainder.trim_end_matches('%');
                        match percent.parse::<f64>() {
                            Ok(percent) => Ok(AnimationStyle::SlideVertPercent(percent)),
                            Err(_) => Err(()),
                        }
                    }
                }
                "fade" => Ok(AnimationStyle::Fade),
                "slidefade" => {
                    let remainder = s.strip_prefix("slidefade").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::SlideFade)
                    } else {
                        let percent = remainder.trim_end_matches('%');
                        match percent.parse::<f64>() {
                            Ok(percent) => Ok(AnimationStyle::SlideFadePercent(percent)),
                            Err(_) => Err(()),
                        }
                    }
                }
                "slidefadevert" => {
                    let remainder = s.strip_prefix("slidefadevert").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::SlideFadeVert)
                    } else {
                        let percent = remainder.trim_end_matches('%');
                        match percent.parse::<f64>() {
                            Ok(percent) => Ok(AnimationStyle::SlideFadeVertPercent(percent)),
                            Err(_) => Err(()),
                        }
                    }
                }
                "once" => Ok(AnimationStyle::Once),
                "loop" => Ok(AnimationStyle::Loop),
                _ => Err(()),
            })
    }
}

impl Display for AnimationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimationStyle::None => write!(f, ""),
            AnimationStyle::Slide => write!(f, "slide"),
            AnimationStyle::SlideSide(side) => write!(f, "slide {}", side),
            AnimationStyle::SlidePercent(percent) => write!(f, "slide {}%", percent),
            AnimationStyle::Popin => write!(f, "popin"),
            AnimationStyle::PopinPercent(percent) => write!(f, "popin {}%", percent),
            AnimationStyle::Gnomed => write!(f, "gnomed"),
            AnimationStyle::SlideVert => write!(f, "slidevert"),
            AnimationStyle::SlideVertPercent(percent) => write!(f, "slidevert {}%", percent),
            AnimationStyle::Fade => write!(f, "fade"),
            AnimationStyle::SlideFade => write!(f, "slidefade"),
            AnimationStyle::SlideFadePercent(percent) => write!(f, "slidefade {}%", percent),
            AnimationStyle::SlideFadeVert => write!(f, "slidefadevert"),
            AnimationStyle::SlideFadeVertPercent(percent) => {
                write!(f, "slidefadevert {}%", percent)
            }
            AnimationStyle::Once => write!(f, "once"),
            AnimationStyle::Loop => write!(f, "loop"),
        }
    }
}

impl EnumConfigForGtk for AnimationStyle {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("gtk_converters.none"),
            &t!("gtk_converters.slide"),
            &t!("gtk_converters.slide_with_side"),
            &t!("gtk_converters.popin"),
            &t!("gtk_converters.popin_with_percent"),
            &t!("gtk_converters.gnomed"),
            &t!("gtk_converters.slide_vert"),
            &t!("gtk_converters.slide_vert_with_percent"),
            &t!("gtk_converters.fade"),
            &t!("gtk_converters.slide_fade"),
            &t!("gtk_converters.slide_fade_with_percent"),
            &t!("gtk_converters.slide_fade_vert"),
            &t!("gtk_converters.slide_fade_vert_with_percent"),
            &t!("gtk_converters.once"),
            &t!("gtk_converters.loop"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            AnimationStyle::None => None,
            AnimationStyle::Slide => None,
            AnimationStyle::SlideSide(_side) => Some(<(Side,)>::to_gtk_box),
            AnimationStyle::SlidePercent(_) => Some(|entry, _, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::Popin => None,
            AnimationStyle::PopinPercent(_percent) => Some(|entry, _, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::Gnomed => None,
            AnimationStyle::SlideVert => None,
            AnimationStyle::SlideVertPercent(_percent) => Some(|entry, _, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::Fade => None,
            AnimationStyle::SlideFade => None,
            AnimationStyle::SlideFadePercent(_percent) => Some(|entry, _, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::SlideFadeVert => None,
            AnimationStyle::SlideFadeVertPercent(_percent) => Some(|entry, _, _, _| {
                create_spin_button_builder(0.0, 100.0, 0.1)(entry, &FieldLabel::Named("%"))
            }),
            AnimationStyle::Once => None,
            AnimationStyle::Loop => None,
        }
    }
}

register_togtkbox!(AnimationStyle);
register_togtkbox_with_separator_names!((Side,));
