use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNamesBuilder,
        create_spin_button_builder,
    },
    register_togtkbox,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(FloatValueDiscriminant))]
pub enum FontWeight {
    Thin,
    UltraLight,
    Light,
    SemiLight,
    Book,
    #[default]
    Normal,
    Medium,
    SemiBold,
    Bold,
    UltraBold,
    Heavy,
    UltraHeavy,
    Integer(u16),
}

impl HasDiscriminant for FontWeight {
    type Discriminant = FloatValueDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Thin => FontWeight::Thin,
            Self::Discriminant::UltraLight => FontWeight::UltraLight,
            Self::Discriminant::Light => FontWeight::Light,
            Self::Discriminant::SemiLight => FontWeight::SemiLight,
            Self::Discriminant::Book => FontWeight::Book,
            Self::Discriminant::Normal => FontWeight::Normal,
            Self::Discriminant::Medium => FontWeight::Medium,
            Self::Discriminant::SemiBold => FontWeight::SemiBold,
            Self::Discriminant::Bold => FontWeight::Bold,
            Self::Discriminant::UltraBold => FontWeight::UltraBold,
            Self::Discriminant::Heavy => FontWeight::Heavy,
            Self::Discriminant::UltraHeavy => FontWeight::UltraHeavy,
            Self::Discriminant::Integer => FontWeight::Integer(500),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Integer => FontWeight::Integer(str.parse::<u16>().unwrap_or(500)),
            discriminant => Self::from_discriminant(discriminant),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::Integer(i) => Some(i.to_string()),
            _ => None,
        }
    }
}

impl FromStr for FontWeight {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();

        if let Ok(i) = s.parse::<u16>() {
            if (100..=1000).contains(&i) {
                return Ok(FontWeight::Integer(i));
            } else {
                return Err(());
            }
        }

        match s.as_str() {
            "thin" => Ok(FontWeight::Thin),
            "ultralight" => Ok(FontWeight::UltraLight),
            "light" => Ok(FontWeight::Light),
            "semilight" => Ok(FontWeight::SemiLight),
            "book" => Ok(FontWeight::Book),
            "normal" => Ok(FontWeight::Normal),
            "medium" => Ok(FontWeight::Medium),
            "semibold" => Ok(FontWeight::SemiBold),
            "bold" => Ok(FontWeight::Bold),
            "ultrabold" => Ok(FontWeight::UltraBold),
            "heavy" => Ok(FontWeight::Heavy),
            "ultraheavy" => Ok(FontWeight::UltraHeavy),
            _ => Err(()),
        }
    }
}

impl Display for FontWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontWeight::Thin => write!(f, "thin"),
            FontWeight::UltraLight => write!(f, "ultralight"),
            FontWeight::Light => write!(f, "light"),
            FontWeight::SemiLight => write!(f, "semilight"),
            FontWeight::Book => write!(f, "book"),
            FontWeight::Normal => write!(f, "normal"),
            FontWeight::Medium => write!(f, "medium"),
            FontWeight::SemiBold => write!(f, "semibold"),
            FontWeight::Bold => write!(f, "bold"),
            FontWeight::UltraBold => write!(f, "ultrabold"),
            FontWeight::Heavy => write!(f, "heavy"),
            FontWeight::UltraHeavy => write!(f, "ultraheavy"),
            FontWeight::Integer(i) => write!(f, "{}", i),
        }
    }
}

impl EnumConfigForGtk for FontWeight {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.font_weight.thin"),
            &t!("hyprland.font_weight.ultralight"),
            &t!("hyprland.font_weight.light"),
            &t!("hyprland.font_weight.semilight"),
            &t!("hyprland.font_weight.book"),
            &t!("hyprland.font_weight.normal"),
            &t!("hyprland.font_weight.medium"),
            &t!("hyprland.font_weight.semibold"),
            &t!("hyprland.font_weight.bold"),
            &t!("hyprland.font_weight.ultrabold"),
            &t!("hyprland.font_weight.heavy"),
            &t!("hyprland.font_weight.ultraheavy"),
            &t!("hyprland.font_weight.integer"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            FontWeight::Integer(_u16) => Some(|entry, _, _names, _| {
                create_spin_button_builder(100.0, 1000.0, 1.0)(entry, &FieldLabel::Unnamed)
            }),
            _ => None,
        }
    }
}

register_togtkbox!(FontWeight);
