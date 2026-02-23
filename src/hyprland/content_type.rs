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

impl TryFrom<u8> for ContentType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Photo),
            2 => Ok(Self::Video),
            3 => Ok(Self::Game),
            _ => Err(()),
        }
    }
}

impl From<ContentType> for u8 {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::None => 0,
            ContentType::Photo => 1,
            ContentType::Video => 2,
            ContentType::Game => 3,
        }
    }
}

impl FromStr for ContentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Ok(num) = s.parse::<u8>()
            && let Ok(content_type) = num.try_into()
        {
            return Ok(content_type);
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
