use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

use crate::hyprland::{AnimationStyle, Side};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum AnimationName {
    #[default]
    Global,
    Windows,
    WindowsIn,
    WindowsOut,
    WindowsMove,
    Layers,
    LayersIn,
    LayersOut,
    Fade,
    FadeIn,
    FadeOut,
    FadeSwitch,
    FadeShadow,
    FadeDim,
    FadeLayers,
    FadeLayersIn,
    FadeLayersOut,
    FadePopups,
    FadePopupsIn,
    FadePopupsOut,
    FadeDpms,
    Border,
    BorderAngle,
    Workspaces,
    WorkspacesIn,
    WorkspacesOut,
    SpecialWorkspace,
    SpecialWorkspaceIn,
    SpecialWorkspaceOut,
    ZoomFactor,
    MonitorAdded,
}

impl AnimationName {
    pub fn get_list() -> [&'static str; 31] {
        [
            "global",
            "windows",
            "windowsIn",
            "windowsOut",
            "windowsMove",
            "layers",
            "layersIn",
            "layersOut",
            "fade",
            "fadeIn",
            "fadeOut",
            "fadeSwitch",
            "fadeShadow",
            "fadeDim",
            "fadeLayers",
            "fadeLayersIn",
            "fadeLayersOut",
            "fadePopups",
            "fadePopupsIn",
            "fadePopupsOut",
            "fadeDpms",
            "border",
            "borderangle",
            "workspaces",
            "workspacesIn",
            "workspacesOut",
            "specialWorkspace",
            "specialWorkspaceIn",
            "specialWorkspaceOut",
            "zoomFactor",
            "monitorAdded",
        ]
    }

    pub fn get_fancy_list() -> [String; 31] {
        [
            t!("utils.global").to_string(),
            t!("utils.windows").to_string(),
            t!("utils.windows_in").to_string(),
            t!("utils.windows_out").to_string(),
            t!("utils.windows_move").to_string(),
            t!("utils.layers").to_string(),
            t!("utils.layers_in").to_string(),
            t!("utils.layers_out").to_string(),
            t!("utils.fade").to_string(),
            t!("utils.fade_in").to_string(),
            t!("utils.fade_out").to_string(),
            t!("utils.fade_switch").to_string(),
            t!("utils.fade_shadow").to_string(),
            t!("utils.fade_dim").to_string(),
            t!("utils.fade_layers").to_string(),
            t!("utils.fade_layers_in").to_string(),
            t!("utils.fade_layers_out").to_string(),
            t!("utils.fade_popups").to_string(),
            t!("utils.fade_popups_in").to_string(),
            t!("utils.fade_popups_out").to_string(),
            t!("utils.fade_dpms").to_string(),
            t!("utils.border").to_string(),
            t!("utils.borderangle").to_string(),
            t!("utils.workspaces").to_string(),
            t!("utils.workspaces_in").to_string(),
            t!("utils.workspaces_out").to_string(),
            t!("utils.special_workspace").to_string(),
            t!("utils.special_workspace_in").to_string(),
            t!("utils.special_workspace_out").to_string(),
            t!("utils.zoom_factor").to_string(),
            t!("utils.monitor_added").to_string(),
        ]
    }

    pub fn get_fancy_available_styles(&self) -> Option<Vec<String>> {
        match self {
            AnimationName::Windows | AnimationName::WindowsIn | AnimationName::WindowsOut => {
                Some(vec![
                    t!("utils.none").to_string(),
                    t!("utils.slide").to_string(),
                    t!("utils.slide_with_side").to_string(),
                    t!("utils.popin").to_string(),
                    t!("utils.popin_with_percent").to_string(),
                    t!("utils.gnomed").to_string(),
                ])
            }
            AnimationName::Layers | AnimationName::LayersIn | AnimationName::LayersOut => {
                Some(vec![
                    t!("utils.none").to_string(),
                    t!("utils.slide").to_string(),
                    t!("utils.slide_with_side").to_string(),
                    t!("utils.popin").to_string(),
                    t!("utils.fade").to_string(),
                ])
            }
            AnimationName::BorderAngle => Some(vec![
                t!("utils.none").to_string(),
                t!("utils.once").to_string(),
                t!("utils.loop").to_string(),
            ]),
            AnimationName::Workspaces
            | AnimationName::WorkspacesIn
            | AnimationName::WorkspacesOut
            | AnimationName::SpecialWorkspace
            | AnimationName::SpecialWorkspaceIn
            | AnimationName::SpecialWorkspaceOut => Some(vec![
                t!("utils.none").to_string(),
                t!("utils.slide").to_string(),
                t!("utils.slide_with_percent").to_string(),
                t!("utils.slidevert").to_string(),
                t!("utils.slidevert_with_percent").to_string(),
                t!("utils.fade").to_string(),
                t!("utils.slidefade").to_string(),
                t!("utils.slidefade_with_percent").to_string(),
                t!("utils.slidefadevert").to_string(),
                t!("utils.slidefade_with_percent").to_string(),
            ]),
            _ => None,
        }
    }

    pub fn get_available_styles(&self) -> Option<Vec<AnimationStyle>> {
        match self {
            AnimationName::Windows | AnimationName::WindowsIn | AnimationName::WindowsOut => {
                Some(vec![
                    AnimationStyle::None,
                    AnimationStyle::Slide,
                    AnimationStyle::SlideSide(Side::Left),
                    AnimationStyle::Popin,
                    AnimationStyle::PopinPercent(50.0),
                    AnimationStyle::Gnomed,
                ])
            }
            AnimationName::Layers | AnimationName::LayersIn | AnimationName::LayersOut => {
                Some(vec![
                    AnimationStyle::None,
                    AnimationStyle::Slide,
                    AnimationStyle::SlideSide(Side::Left),
                    AnimationStyle::Popin,
                    AnimationStyle::Fade,
                ])
            }
            AnimationName::BorderAngle => Some(vec![
                AnimationStyle::None,
                AnimationStyle::Once,
                AnimationStyle::Loop,
            ]),
            AnimationName::Workspaces
            | AnimationName::WorkspacesIn
            | AnimationName::WorkspacesOut
            | AnimationName::SpecialWorkspace
            | AnimationName::SpecialWorkspaceIn
            | AnimationName::SpecialWorkspaceOut => Some(vec![
                AnimationStyle::None,
                AnimationStyle::Slide,
                AnimationStyle::SlidePercent(50.0),
                AnimationStyle::SlideVert,
                AnimationStyle::SlideVertPercent(50.0),
                AnimationStyle::Fade,
                AnimationStyle::SlideFade,
                AnimationStyle::SlideFadePercent(50.0),
                AnimationStyle::SlideFadeVert,
                AnimationStyle::SlideFadePercent(50.0),
            ]),
            _ => None,
        }
    }

    pub fn get_id_of_style(&self, style: AnimationStyle) -> Option<usize> {
        match self {
            AnimationName::Windows | AnimationName::WindowsIn | AnimationName::WindowsOut => {
                match style {
                    AnimationStyle::None => Some(0),
                    AnimationStyle::Slide => Some(1),
                    AnimationStyle::SlideSide(_) => Some(2),
                    AnimationStyle::Popin => Some(3),
                    AnimationStyle::PopinPercent(_) => Some(4),
                    AnimationStyle::Gnomed => Some(5),
                    _ => None,
                }
            }
            AnimationName::Layers | AnimationName::LayersIn | AnimationName::LayersOut => {
                match style {
                    AnimationStyle::None => Some(0),
                    AnimationStyle::Slide => Some(1),
                    AnimationStyle::SlideSide(_) => Some(2),
                    AnimationStyle::Popin => Some(3),
                    AnimationStyle::Fade => Some(4),
                    _ => None,
                }
            }
            AnimationName::BorderAngle => match style {
                AnimationStyle::None => Some(0),
                AnimationStyle::Once => Some(1),
                AnimationStyle::Loop => Some(2),
                _ => None,
            },
            AnimationName::Workspaces
            | AnimationName::WorkspacesIn
            | AnimationName::WorkspacesOut
            | AnimationName::SpecialWorkspace
            | AnimationName::SpecialWorkspaceIn
            | AnimationName::SpecialWorkspaceOut => match style {
                AnimationStyle::None => Some(0),
                AnimationStyle::Slide => Some(1),
                AnimationStyle::SlidePercent(_) => Some(2),
                AnimationStyle::SlideVert => Some(3),
                AnimationStyle::SlideVertPercent(_) => Some(4),
                AnimationStyle::Fade => Some(5),
                AnimationStyle::SlideFade => Some(6),
                AnimationStyle::SlideFadePercent(_) => Some(7),
                AnimationStyle::SlideFadeVert => Some(8),
                AnimationStyle::SlideFadeVertPercent(_) => Some(9),
                _ => None,
            },
            _ => None,
        }
    }
}

impl FromStr for AnimationName {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "global" => Ok(AnimationName::Global),
            "windows" => Ok(AnimationName::Windows),
            "windowsIn" => Ok(AnimationName::WindowsIn),
            "windowsOut" => Ok(AnimationName::WindowsOut),
            "windowsMove" => Ok(AnimationName::WindowsMove),
            "layers" => Ok(AnimationName::Layers),
            "layersIn" => Ok(AnimationName::LayersIn),
            "layersOut" => Ok(AnimationName::LayersOut),
            "fade" => Ok(AnimationName::Fade),
            "fadeIn" => Ok(AnimationName::FadeIn),
            "fadeOut" => Ok(AnimationName::FadeOut),
            "fadeSwitch" => Ok(AnimationName::FadeSwitch),
            "fadeShadow" => Ok(AnimationName::FadeShadow),
            "fadeDim" => Ok(AnimationName::FadeDim),
            "fadeLayers" => Ok(AnimationName::FadeLayers),
            "fadeLayersIn" => Ok(AnimationName::FadeLayersIn),
            "fadeLayersOut" => Ok(AnimationName::FadeLayersOut),
            "fadePopups" => Ok(AnimationName::FadePopups),
            "fadePopupsIn" => Ok(AnimationName::FadePopupsIn),
            "fadePopupsOut" => Ok(AnimationName::FadePopupsOut),
            "fadeDpms" => Ok(AnimationName::FadeDpms),
            "border" => Ok(AnimationName::Border),
            "borderangle" => Ok(AnimationName::BorderAngle),
            "workspaces" => Ok(AnimationName::Workspaces),
            "workspacesIn" => Ok(AnimationName::WorkspacesIn),
            "workspacesOut" => Ok(AnimationName::WorkspacesOut),
            "specialWorkspace" => Ok(AnimationName::SpecialWorkspace),
            "specialWorkspaceIn" => Ok(AnimationName::SpecialWorkspaceIn),
            "specialWorkspaceOut" => Ok(AnimationName::SpecialWorkspaceOut),
            "zoomFactor" => Ok(AnimationName::ZoomFactor),
            "monitorAdded" => Ok(AnimationName::MonitorAdded),
            _ => Ok(AnimationName::Global),
        }
    }
}

impl Display for AnimationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimationName::Global => write!(f, "global"),
            AnimationName::Windows => write!(f, "windows"),
            AnimationName::WindowsIn => write!(f, "windowsIn"),
            AnimationName::WindowsOut => write!(f, "windowsOut"),
            AnimationName::WindowsMove => write!(f, "windowsMove"),
            AnimationName::Layers => write!(f, "layers"),
            AnimationName::LayersIn => write!(f, "layersIn"),
            AnimationName::LayersOut => write!(f, "layersOut"),
            AnimationName::Fade => write!(f, "fade"),
            AnimationName::FadeIn => write!(f, "fadeIn"),
            AnimationName::FadeOut => write!(f, "fadeOut"),
            AnimationName::FadeSwitch => write!(f, "fadeSwitch"),
            AnimationName::FadeShadow => write!(f, "fadeShadow"),
            AnimationName::FadeDim => write!(f, "fadeDim"),
            AnimationName::FadeLayers => write!(f, "fadeLayers"),
            AnimationName::FadeLayersIn => write!(f, "fadeLayersIn"),
            AnimationName::FadeLayersOut => write!(f, "fadeLayersOut"),
            AnimationName::FadePopups => write!(f, "fadePopups"),
            AnimationName::FadePopupsIn => write!(f, "fadePopupsIn"),
            AnimationName::FadePopupsOut => write!(f, "fadePopupsOut"),
            AnimationName::FadeDpms => write!(f, "fadeDpms"),
            AnimationName::Border => write!(f, "border"),
            AnimationName::BorderAngle => write!(f, "borderangle"),
            AnimationName::Workspaces => write!(f, "workspaces"),
            AnimationName::WorkspacesIn => write!(f, "workspacesIn"),
            AnimationName::WorkspacesOut => write!(f, "workspacesOut"),
            AnimationName::SpecialWorkspace => write!(f, "specialWorkspace"),
            AnimationName::SpecialWorkspaceIn => write!(f, "specialWorkspaceIn"),
            AnimationName::SpecialWorkspaceOut => write!(f, "specialWorkspaceOut"),
            AnimationName::ZoomFactor => write!(f, "zoomFactor"),
            AnimationName::MonitorAdded => write!(f, "monitorAdded"),
        }
    }
}
