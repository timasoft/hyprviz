use super::{HyprExpression, HyprVariable, Operator, PixelOrPercent};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HyprCoord {
    pub x: PixelOrPercent,
    pub y: PixelOrPercent,
    pub x_sub: u32,
    pub y_sub: u32,
    pub under_cursor: bool,
    pub on_screen: bool,
}

impl HyprCoord {
    pub fn to_expressions(self) -> (HyprExpression, HyprExpression) {
        let x_expr = Self::build_expression(self.x, self.x_sub, self.under_cursor, true);
        let y_expr = Self::build_expression(self.y, self.y_sub, self.under_cursor, false);

        (x_expr, y_expr)
    }

    fn build_expression(
        value: PixelOrPercent,
        sub: u32,
        under_cursor: bool,
        is_x: bool,
    ) -> HyprExpression {
        let base_var = if is_x {
            if under_cursor {
                HyprVariable::WindowW
            } else {
                HyprVariable::MonitorW
            }
        } else if under_cursor {
            HyprVariable::WindowH
        } else {
            HyprVariable::MonitorH
        };

        let cursor_var = if is_x {
            HyprVariable::CursorX
        } else {
            HyprVariable::CursorY
        };

        let base = match value {
            PixelOrPercent::Pixel(p) => HyprExpression::Uint(p.unsigned_abs()),
            PixelOrPercent::Percent(pct) => HyprExpression::Formula(
                Box::new(HyprExpression::Variable(base_var)),
                Operator::Multiply,
                Box::new(HyprExpression::Float(pct * 0.01)),
            ),
        };

        let with_sub = if sub > 0 {
            HyprExpression::Formula(
                Box::new(base),
                Operator::Subtract,
                Box::new(HyprExpression::Uint(sub)),
            )
        } else {
            base
        };

        if under_cursor {
            HyprExpression::Formula(
                Box::new(HyprExpression::Variable(cursor_var)),
                Operator::Subtract,
                Box::new(with_sub),
            )
        } else {
            with_sub
        }
    }
}

impl FromStr for HyprCoord {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let mut result = HyprCoord::default();

        let parts: Vec<&str> = s.split(' ').collect();

        if parts.is_empty() {
            return Err(());
        }

        let mut is_x = true;

        for part in parts {
            let part = part.trim();
            if part == "onscreen" {
                result.on_screen = true;
            } else if part == "undercursor" {
                result.under_cursor = true;
            } else {
                // parse "100", "100%", "100%-100"
                let (num_or_percent, sub) = part.split_once('-').unwrap_or((part, ""));
                let num_or_percent: PixelOrPercent =
                    PixelOrPercent::from_str(num_or_percent).unwrap_or_default();
                let sub: u32 = sub.parse().unwrap_or_default();
                if is_x {
                    result.x = num_or_percent;
                    result.x_sub = sub;
                    is_x = false;
                } else {
                    result.y = num_or_percent;
                    result.y_sub = sub;
                    break;
                }
            }
        }

        Ok(result)
    }
}
