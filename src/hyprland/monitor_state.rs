use super::{Cm, Position, Scale};
use std::fmt::Display;

pub struct MonitorState {
    pub resolution: String,
    pub position: Position,
    pub scale: Scale,
    pub mirror: Option<String>,
    pub bitdepth: Option<u8>,
    pub cm: Option<Cm>,
    pub sdrbrightness: Option<f64>,
    pub sdrsaturation: Option<f64>,
    pub vrr: Option<u8>,
    pub transform: Option<u8>,
}

impl Display for MonitorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let resolution = self.resolution.clone();
        let position = format!(", {}", self.position);
        let scale = format!(", {}", self.scale);
        let mirror = match &self.mirror {
            Some(mirror) => format!(", mirror, {}", mirror),
            None => "".to_string(),
        };
        let bitdepth = match &self.bitdepth {
            Some(bitdepth) => format!(", bitdepth, {}", bitdepth),
            None => "".to_string(),
        };
        let cm = match &self.cm {
            Some(cm) => format!(", cm, {}", cm),
            None => "".to_string(),
        };
        let sdrbrightness = match &self.sdrbrightness {
            Some(sdrbrightness) => format!(", sdrbrightness, {}", sdrbrightness),
            None => "".to_string(),
        };
        let sdrsaturation = match &self.sdrsaturation {
            Some(sdrsaturation) => format!(", sdrsaturation, {}", sdrsaturation),
            None => "".to_string(),
        };
        let vrr = match &self.vrr {
            Some(vrr) => format!(", vrr, {}", vrr),
            None => "".to_string(),
        };
        let transform = match &self.transform {
            Some(transform) => format!(", transform, {}", transform),
            None => "".to_string(),
        };

        write!(
            f,
            "{}{}{}{}{}{}{}{}{}{}",
            resolution,
            position,
            scale,
            mirror,
            bitdepth,
            cm,
            sdrbrightness,
            sdrsaturation,
            vrr,
            transform
        )
    }
}
