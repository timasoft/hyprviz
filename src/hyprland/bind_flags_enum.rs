use rust_i18n::t;
use strum::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum BindFlagsEnum {
    Locked,
    Release,
    Click,
    Drag,
    LongPress,
    Repeat,
    NonConsuming,
    Mouse,
    Transparent,
    IgnoreMods,
    Separate,
    HasDescription,
    Bypass,
}

impl BindFlagsEnum {
    pub fn get_all() -> [BindFlagsEnum; 13] {
        [
            BindFlagsEnum::Locked,
            BindFlagsEnum::Release,
            BindFlagsEnum::Click,
            BindFlagsEnum::Drag,
            BindFlagsEnum::LongPress,
            BindFlagsEnum::Repeat,
            BindFlagsEnum::NonConsuming,
            BindFlagsEnum::Mouse,
            BindFlagsEnum::Transparent,
            BindFlagsEnum::IgnoreMods,
            BindFlagsEnum::Separate,
            BindFlagsEnum::HasDescription,
            BindFlagsEnum::Bypass,
        ]
    }

    pub fn to_fancy_string(&self) -> String {
        match self {
            BindFlagsEnum::Locked => t!("utils.locked").to_string(),
            BindFlagsEnum::Release => t!("utils.release").to_string(),
            BindFlagsEnum::Click => t!("utils.click").to_string(),
            BindFlagsEnum::Drag => t!("utils.drag").to_string(),
            BindFlagsEnum::LongPress => t!("utils.long_press").to_string(),
            BindFlagsEnum::Repeat => t!("utils.repeat").to_string(),
            BindFlagsEnum::NonConsuming => t!("utils.non_consuming").to_string(),
            BindFlagsEnum::Mouse => t!("utils.mouse").to_string(),
            BindFlagsEnum::Transparent => t!("utils.transparent").to_string(),
            BindFlagsEnum::IgnoreMods => t!("utils.ignore_mods").to_string(),
            BindFlagsEnum::Separate => t!("utils.separate").to_string(),
            BindFlagsEnum::HasDescription => t!("utils.has_description").to_string(),
            BindFlagsEnum::Bypass => t!("utils.bypass").to_string(),
        }
    }
}
