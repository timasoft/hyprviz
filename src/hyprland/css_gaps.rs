use crate::{
    gtk_converters::{
        EnumConfigForGtk, ToGtkBoxWithSeparatorAndNames, ToGtkBoxWithSeparatorAndNamesBuilder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::HasDiscriminant,
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(CssGapsDiscriminant))]
pub enum CssGaps {
    All(u32),
    VerticalHorizontal(u32, u32),
    TopSidesBottom(u32, u32, u32),
    TopRightBottomLeft(u32, u32, u32, u32),
}

impl HasDiscriminant for CssGaps {
    type Discriminant = CssGapsDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            CssGapsDiscriminant::All => CssGaps::All(0),
            CssGapsDiscriminant::VerticalHorizontal => CssGaps::VerticalHorizontal(0, 0),
            CssGapsDiscriminant::TopSidesBottom => CssGaps::TopSidesBottom(0, 0, 0),
            CssGapsDiscriminant::TopRightBottomLeft => CssGaps::TopRightBottomLeft(0, 0, 0, 0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        let parts: Vec<u32> = str
            .split(',')
            .map(|s| s.trim().parse::<u32>().unwrap_or_default())
            .collect();

        match discriminant {
            CssGapsDiscriminant::All => CssGaps::All(parts.first().copied().unwrap_or_default()),
            CssGapsDiscriminant::VerticalHorizontal => CssGaps::VerticalHorizontal(
                parts.first().copied().unwrap_or_default(),
                parts.get(1).copied().unwrap_or_default(),
            ),
            CssGapsDiscriminant::TopSidesBottom => CssGaps::TopSidesBottom(
                parts.first().copied().unwrap_or_default(),
                parts.get(1).copied().unwrap_or_default(),
                parts.get(2).copied().unwrap_or_default(),
            ),
            CssGapsDiscriminant::TopRightBottomLeft => CssGaps::TopRightBottomLeft(
                parts.first().copied().unwrap_or_default(),
                parts.get(1).copied().unwrap_or_default(),
                parts.get(2).copied().unwrap_or_default(),
                parts.get(3).copied().unwrap_or_default(),
            ),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            CssGaps::All(gap) => Some(gap.to_string()),
            CssGaps::VerticalHorizontal(gap1, gap2) => Some(format!("{gap1},{gap2}")),
            CssGaps::TopSidesBottom(gap1, gap2, gap3) => Some(format!("{gap1},{gap2},{gap3}")),
            CssGaps::TopRightBottomLeft(gap1, gap2, gap3, gap4) => {
                Some(format!("{gap1},{gap2},{gap3},{gap4}"))
            }
        }
    }
}

impl Default for CssGaps {
    fn default() -> Self {
        CssGaps::All(0)
    }
}

impl FromStr for CssGaps {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(',');
        let mut gaps = Vec::new();
        for part in parts {
            gaps.push(part.trim().parse::<u32>().unwrap_or_default());
        }
        match gaps.len() {
            0 => Err(()),
            1 => Ok(CssGaps::All(gaps[0])),
            2 => Ok(CssGaps::VerticalHorizontal(gaps[0], gaps[1])),
            3 => Ok(CssGaps::TopSidesBottom(gaps[0], gaps[1], gaps[2])),
            _ => Ok(CssGaps::TopRightBottomLeft(
                gaps[0], gaps[1], gaps[2], gaps[3],
            )),
        }
    }
}

impl Display for CssGaps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CssGaps::All(gap) => write!(f, "{gap}"),
            CssGaps::VerticalHorizontal(gap1, gap2) => write!(f, "{gap1},{gap2}"),
            CssGaps::TopSidesBottom(gap1, gap2, gap3) => write!(f, "{gap1},{gap2},{gap3}"),
            CssGaps::TopRightBottomLeft(gap1, gap2, gap3, gap4) => {
                write!(f, "{gap1},{gap2},{gap3},{gap4}")
            }
        }
    }
}

impl EnumConfigForGtk for CssGaps {
    fn dropdown_items() -> gtk::StringList {
        StringList::new(&[
            &t!("hyprland.css_gaps.all"),
            &t!("hyprland.css_gaps.vertical_horizontal"),
            &t!("hyprland.css_gaps.top_sides_bottom"),
            &t!("hyprland.css_gaps.top_right_bottom_left"),
        ])
    }

    const SEPARATOR: Option<char> = Some(',');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            CssGaps::All(_gap) => Some(<(u32,)>::to_gtk_box),
            CssGaps::VerticalHorizontal(_gap1, _gap2) => Some(<(u32, u32)>::to_gtk_box),
            CssGaps::TopSidesBottom(_gap1, _gap2, _gap3) => Some(<(u32, u32, u32)>::to_gtk_box),
            CssGaps::TopRightBottomLeft(_gap1, _gap2, _gap3, _gap4) => {
                Some(<(u32, u32, u32, u32)>::to_gtk_box)
            }
        }
    }
}

register_togtkbox!(CssGaps);
register_togtkbox_with_separator_names!((u32, u32, u32), (u32, u32, u32, u32));
