use super::WorkspaceSelectorWindowCountFlags;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorkspaceSelectorWindowCountDiscriminant))]
pub enum WorkspaceSelectorWindowCount {
    Range {
        flags: WorkspaceSelectorWindowCountFlags,
        range_start: u32,
        range_end: u32,
    },
    Single {
        flags: WorkspaceSelectorWindowCountFlags,
        count: u32,
    },
}

impl HasDiscriminant for WorkspaceSelectorWindowCount {
    type Discriminant = WorkspaceSelectorWindowCountDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Range => Self::Range {
                flags: WorkspaceSelectorWindowCountFlags::default(),
                range_start: 0,
                range_end: 0,
            },
            Self::Discriminant::Single => Self::Single {
                flags: WorkspaceSelectorWindowCountFlags::default(),
                count: 0,
            },
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Range => match Self::from_str(str).unwrap_or_default() {
                Self::Range {
                    flags,
                    range_start,
                    range_end,
                } => Self::Range {
                    flags,
                    range_start,
                    range_end,
                },
                Self::Single { flags, count } => Self::Range {
                    flags,
                    range_start: count,
                    range_end: count,
                },
            },
            Self::Discriminant::Single => match Self::from_str(str).unwrap_or_default() {
                Self::Range {
                    flags,
                    range_start,
                    range_end: _,
                } => Self::Single {
                    flags,
                    count: range_start,
                },
                Self::Single { flags, count } => Self::Single { flags, count },
            },
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::Range {
                flags,
                range_start,
                range_end,
            } => Some(format!("{}{}-{}", flags, range_start, range_end)),
            Self::Single { flags, count } => Some(format!("{}{}", flags, count)),
        }
    }

    fn custom_split(discriminant: Self::Discriminant) -> Option<fn(&str) -> Vec<&str>> {
        match discriminant {
            Self::Discriminant::Range => Some(|s: &str| {
                let s = s.trim();
                let (flags_str, count_str) = if let Some(pos) = s.find(|c: char| !c.is_alphabetic())
                {
                    (&s[..pos], &s[pos..])
                } else {
                    (s, "")
                };
                let count_str = count_str.trim().trim_matches('-');
                let (start_str, end_str) =
                    count_str.split_once('-').unwrap_or((count_str, count_str));

                vec![flags_str, start_str, end_str]
            }),
            Self::Discriminant::Single => Some(|s: &str| {
                let s = s.trim();
                let (flags_str, count_str) = if let Some(pos) = s.find(|c: char| !c.is_alphabetic())
                {
                    (&s[..pos], &s[pos..])
                } else {
                    (s, "")
                };
                let count_str = count_str.trim().trim_matches('-');

                vec![flags_str, count_str]
            }),
        }
    }
}

impl Default for WorkspaceSelectorWindowCount {
    fn default() -> Self {
        WorkspaceSelectorWindowCount::Single {
            flags: WorkspaceSelectorWindowCountFlags::default(),
            count: 0,
        }
    }
}

impl FromStr for WorkspaceSelectorWindowCount {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let (flags_str, count_str) = if let Some(pos) = s.find(|c: char| !c.is_alphabetic()) {
            (&s[..pos], &s[pos..])
        } else {
            (s, "")
        };

        let flags = WorkspaceSelectorWindowCountFlags::from_str(flags_str).unwrap_or_default();

        let count_str = count_str.trim().trim_matches('-');

        if count_str.contains('-') {
            if let Some((start_str, end_str)) = count_str.split_once('-')
                && let (Ok(start), Ok(end)) = (
                    start_str.trim().parse::<u32>(),
                    end_str.trim().parse::<u32>(),
                )
            {
                Ok(WorkspaceSelectorWindowCount::Range {
                    flags,
                    range_start: start,
                    range_end: end,
                })
            } else {
                Err(())
            }
        } else if !count_str.is_empty()
            && let Ok(count) = count_str.parse::<u32>()
        {
            Ok(WorkspaceSelectorWindowCount::Single { flags, count })
        } else {
            Err(())
        }
    }
}

impl Display for WorkspaceSelectorWindowCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSelectorWindowCount::Range {
                flags,
                range_start,
                range_end,
            } => {
                write!(f, "{}{}-{}", flags, range_start, range_end)
            }
            WorkspaceSelectorWindowCount::Single { flags, count } => {
                write!(f, "{}{}", flags, count)
            }
        }
    }
}

impl EnumConfigForGtk for WorkspaceSelectorWindowCount {
    fn dropdown_items() -> StringList {
        StringList::new(&[&t!("gtk_converters.range"), &t!("gtk_converters.single")])
    }

    const SEPARATOR: Option<char> = Some('-');

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            Self::Range {
                flags: _,
                range_start: _,
                range_end: _,
            } => Some(<(WorkspaceSelectorWindowCountFlags, u32, u32)>::to_gtk_box),
            Self::Single { flags: _, count: _ } => {
                Some(<(WorkspaceSelectorWindowCountFlags, u32)>::to_gtk_box)
            }
        }
    }
}

register_togtkbox!(WorkspaceSelectorWindowCount);
register_togtkbox_with_separator_names!(
    (WorkspaceSelectorWindowCountFlags, u32, u32),
    (WorkspaceSelectorWindowCountFlags, u32)
);
