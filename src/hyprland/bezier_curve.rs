use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub struct BezierCurve {
    pub name: String,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl FromStr for BezierCurve {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let values: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if values.len() < 5 {
            return Err(());
        }

        let name = values[0].clone();
        let x0 = values[1].parse::<f64>().unwrap_or(0.333);
        let y0 = values[2].parse::<f64>().unwrap_or(0.333);
        let x1 = values[3].parse::<f64>().unwrap_or(0.667);
        let y1 = values[4].parse::<f64>().unwrap_or(0.667);

        Ok(BezierCurve {
            name,
            x0,
            y0,
            x1,
            y1,
        })
    }
}

impl Display for BezierCurve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {:.3}, {:.3}, {:.3}, {:.3}",
            self.name, self.x0, self.y0, self.x1, self.y1
        )
    }
}

pub fn parse_bezier(input: &str) -> BezierCurve {
    BezierCurve::from_str(input).unwrap_or(BezierCurve {
        name: "default".to_string(),
        x0: 0.333,
        y0: 0.333,
        x1: 0.667,
        y1: 0.667,
    })
}
