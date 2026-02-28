use crate::{gtk_converters::EnumConfigForGtk, register_togtkbox};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum ContentTypeInt {
    #[default]
    None,
    Photo,
    Video,
    Game,
}

impl TryFrom<u8> for ContentTypeInt {
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

impl From<ContentTypeInt> for u8 {
    fn from(value: ContentTypeInt) -> Self {
        match value {
            ContentTypeInt::None => 0,
            ContentTypeInt::Photo => 1,
            ContentTypeInt::Video => 2,
            ContentTypeInt::Game => 3,
        }
    }
}

impl FromStr for ContentTypeInt {
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
            "none" => Ok(ContentTypeInt::None),
            "photo" => Ok(ContentTypeInt::Photo),
            "video" => Ok(ContentTypeInt::Video),
            "game" => Ok(ContentTypeInt::Game),
            _ => Err(()),
        }
    }
}

impl Display for ContentTypeInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentTypeInt::None => write!(f, "0"),
            ContentTypeInt::Photo => write!(f, "1"),
            ContentTypeInt::Video => write!(f, "2"),
            ContentTypeInt::Game => write!(f, "3"),
        }
    }
}

impl EnumConfigForGtk for ContentTypeInt {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.content_type.none"),
            &t!("hyprland.content_type.photo"),
            &t!("hyprland.content_type.video"),
            &t!("hyprland.content_type.game"),
        ])
    }
}

register_togtkbox!(ContentTypeInt);
