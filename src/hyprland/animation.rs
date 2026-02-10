use super::{AnimationName, AnimationStyle};
use crate::utils::parse_bool;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: AnimationName,
    pub enabled: bool,
    pub speed: f64,
    pub curve: String,
    pub style: AnimationStyle,
}

impl FromStr for Animation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let values: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if values.is_empty() | values.first().is_none_or(|v| v.is_empty()) {
            return Err(());
        }

        let name = AnimationName::from_str(&values[0]).unwrap_or_default();
        let enabled = values.get(1).is_none_or(|v| parse_bool(v).unwrap_or(true));
        let speed = values
            .get(2)
            .map_or(10.0, |v| v.parse::<f64>().unwrap_or(10.0));
        let curve = values.get(3).map_or("default".to_string(), |v| v.clone());
        let style = values.get(4).map_or(AnimationStyle::None, |v| {
            AnimationStyle::from_str(v).unwrap_or_default()
        });

        Ok(Animation {
            name,
            enabled,
            speed,
            curve,
            style,
        })
    }
}

impl Display for Animation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {:.1}, {}",
            self.name,
            if self.enabled { 1 } else { 0 },
            self.speed,
            self.curve
        )?;
        if !matches!(self.style, AnimationStyle::None) {
            write!(f, ", {}", self.style)?;
        }
        Ok(())
    }
}

pub fn parse_animation(input: &str) -> Animation {
    Animation::from_str(input).unwrap_or(Animation {
        name: AnimationName::Global,
        enabled: true,
        speed: 10.0,
        curve: "default".to_string(),
        style: AnimationStyle::None,
    })
}
