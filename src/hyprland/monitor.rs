use super::{Cm, MonitorSelector, MonitorState, Position, Scale};
use std::str::FromStr;

pub enum Monitor {
    Enabled(MonitorState),
    Disabled,
    AddReserved(i64, i64, i64, i64),
}

pub fn parse_monitor(input: &str) -> (MonitorSelector, Monitor) {
    let values = input
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    let monitor_selector =
        MonitorSelector::from_str(values.first().unwrap_or(&"".to_string()).as_str())
            .unwrap_or_default();

    let state = values.get(1).unwrap_or(&"preferred".to_string()).to_owned();

    match state.as_str() {
        "disable" => (monitor_selector, Monitor::Disabled),
        "addreserved" => (
            monitor_selector,
            Monitor::AddReserved(
                values
                    .get(2)
                    .unwrap_or(&"0".to_string())
                    .parse::<i64>()
                    .unwrap_or(0),
                values
                    .get(3)
                    .unwrap_or(&"0".to_string())
                    .parse::<i64>()
                    .unwrap_or(0),
                values
                    .get(4)
                    .unwrap_or(&"0".to_string())
                    .parse::<i64>()
                    .unwrap_or(0),
                values
                    .get(5)
                    .unwrap_or(&"0".to_string())
                    .parse::<i64>()
                    .unwrap_or(0),
            ),
        ),
        resolution => {
            let mut monitor_state = MonitorState {
                resolution: resolution.to_string(),
                position: {
                    Position::from_str(values.get(2).unwrap_or(&"auto".to_string()).as_str())
                        .unwrap_or(Position::Auto)
                },
                scale: {
                    Scale::from_str(values.get(3).unwrap_or(&"auto".to_string()).as_str())
                        .unwrap_or(Scale::Auto)
                },
                mirror: None,
                bitdepth: None,
                cm: None,
                sdrbrightness: None,
                sdrsaturation: None,
                vrr: None,
                transform: None,
            };

            for i in 4..values.len() {
                match values.get(i).unwrap_or(&"".to_string()).as_str() {
                    "mirror" => {
                        monitor_state.mirror =
                            Some(values.get(i + 1).unwrap_or(&"".to_string()).to_owned());
                    }
                    "bitdepth" => {
                        monitor_state.bitdepth = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"10".to_string())
                                .parse::<u8>()
                                .unwrap_or(0),
                        );
                    }
                    "cm" => {
                        monitor_state.cm = Some(
                            match values
                                .get(i + 1)
                                .unwrap_or(&"auto".to_string())
                                .to_owned()
                                .as_str()
                            {
                                "auto" => Cm::Auto,
                                "srgb" => Cm::Srgb,
                                "dcip3" => Cm::Dcip3,
                                "dp3" => Cm::Dp3,
                                "adobe" => Cm::Adobe,
                                "wide" => Cm::Wide,
                                "edid" => Cm::Edid,
                                "hdr" => Cm::Hdr,
                                "hdredid" => Cm::Hdredid,
                                _ => Cm::Auto,
                            },
                        );
                    }
                    "sdrbrightness" => {
                        monitor_state.sdrbrightness = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"1.0".to_string())
                                .parse::<f64>()
                                .unwrap_or(1.0),
                        )
                    }
                    "sdrsaturation" => {
                        monitor_state.sdrsaturation = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"1.0".to_string())
                                .parse::<f64>()
                                .unwrap_or(1.0),
                        );
                    }
                    "vrr" => {
                        monitor_state.vrr = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"0".to_string())
                                .parse::<u8>()
                                .unwrap_or(0),
                        );
                    }
                    "transform" => {
                        monitor_state.transform = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"0".to_string())
                                .parse::<u8>()
                                .unwrap_or(0),
                        );
                    }
                    _ => {}
                }
            }

            (monitor_selector, Monitor::Enabled(monitor_state))
        }
    }
}
