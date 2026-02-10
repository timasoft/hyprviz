use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(EnumIter)]
pub enum Cm {
    Auto,
    Srgb,
    Dcip3,
    Dp3,
    Adobe,
    Wide,
    Edid,
    Hdr,
    Hdredid,
}

impl Cm {
    pub fn get_fancy_list() -> [&'static str; 9] {
        [
            "Auto",
            "SRGB",
            "CDI-P3",
            "DP3",
            "AdobeRGB",
            "WideGamut",
            "EDID",
            "HDR",
            "HDR-EDID",
        ]
    }

    pub fn from_id(id: u32) -> Self {
        match id {
            0 => Cm::Auto,
            1 => Cm::Srgb,
            2 => Cm::Dcip3,
            3 => Cm::Dp3,
            4 => Cm::Adobe,
            5 => Cm::Wide,
            6 => Cm::Edid,
            7 => Cm::Hdr,
            8 => Cm::Hdredid,
            _ => Cm::Auto,
        }
    }
}

impl Display for Cm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cm::Auto => write!(f, "auto"),
            Cm::Srgb => write!(f, "srgb"),
            Cm::Dcip3 => write!(f, "dcip3"),
            Cm::Dp3 => write!(f, "dp3"),
            Cm::Adobe => write!(f, "adobe"),
            Cm::Wide => write!(f, "wide"),
            Cm::Edid => write!(f, "edid"),
            Cm::Hdr => write!(f, "hdr"),
            Cm::Hdredid => write!(f, "hdredid"),
        }
    }
}

impl FromStr for Cm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "auto" => Ok(Cm::Auto),
            "srgb" => Ok(Cm::Srgb),
            "dcip3" => Ok(Cm::Dcip3),
            "dp3" => Ok(Cm::Dp3),
            "adobe" => Ok(Cm::Adobe),
            "wide" => Ok(Cm::Wide),
            "edid" => Ok(Cm::Edid),
            "hdr" => Ok(Cm::Hdr),
            "hdredid" => Ok(Cm::Hdredid),
            _ => Err(()),
        }
    }
}
