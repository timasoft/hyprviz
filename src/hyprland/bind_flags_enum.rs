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
            BindFlagsEnum::Locked => t!("hyprland.bind_flags_enum.locked").to_string(),
            BindFlagsEnum::Release => t!("hyprland.bind_flags_enum.release").to_string(),
            BindFlagsEnum::Click => t!("hyprland.bind_flags_enum.click").to_string(),
            BindFlagsEnum::Drag => t!("hyprland.bind_flags_enum.drag").to_string(),
            BindFlagsEnum::LongPress => t!("hyprland.bind_flags_enum.long_press").to_string(),
            BindFlagsEnum::Repeat => t!("hyprland.bind_flags_enum.repeat").to_string(),
            BindFlagsEnum::NonConsuming => t!("hyprland.bind_flags_enum.non_consuming").to_string(),
            BindFlagsEnum::Mouse => t!("hyprland.bind_flags_enum.mouse").to_string(),
            BindFlagsEnum::Transparent => t!("hyprland.bind_flags_enum.transparent").to_string(),
            BindFlagsEnum::IgnoreMods => t!("hyprland.bind_flags_enum.ignore_mods").to_string(),
            BindFlagsEnum::Separate => t!("hyprland.bind_flags_enum.separate").to_string(),
            BindFlagsEnum::HasDescription => {
                t!("hyprland.bind_flags_enum.has_description").to_string()
            }
            BindFlagsEnum::Bypass => t!("hyprland.bind_flags_enum.bypass").to_string(),
        }
    }
}
