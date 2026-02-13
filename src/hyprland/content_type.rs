use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum ContentType {
    #[default]
    None,
    Photo,
    Video,
    Game,
}

impl FromStr for ContentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        match s {
            "none" => Ok(ContentType::None),
            "photo" => Ok(ContentType::Photo),
            "video" => Ok(ContentType::Video),
            "game" => Ok(ContentType::Game),
            _ => Err(()),
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::None => write!(f, "none"),
            ContentType::Photo => write!(f, "photo"),
            ContentType::Video => write!(f, "video"),
            ContentType::Game => write!(f, "game"),
        }
    }
}

impl EnumConfigForGtk for ContentType {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.content_type.none"),
            &t!("hyprland.content_type.photo"),
            &t!("hyprland.content_type.video"),
            &t!("hyprland.content_type.game"),
        ])
    }
}

register_togtkbox!(ContentType);
