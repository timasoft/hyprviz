use super::{HyprExpression, HyprVariable, Operator, PixelOrPercent, SizeBound};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HyprSize {
    pub width: PixelOrPercent,
    pub height: PixelOrPercent,
    pub width_bound: SizeBound,
    pub height_bound: SizeBound,
}

impl HyprSize {
    pub fn to_expressions(self) -> (HyprExpression, HyprExpression) {
        let width_expr = Self::build_expression(self.width, HyprVariable::MonitorW);
        let height_expr = Self::build_expression(self.height, HyprVariable::MonitorH);

        (width_expr, height_expr)
    }

    fn build_expression(value: PixelOrPercent, monitor_var: HyprVariable) -> HyprExpression {
        match value {
            PixelOrPercent::Pixel(p) => HyprExpression::Uint(p.unsigned_abs()),
            PixelOrPercent::Percent(pct) => HyprExpression::Formula(
                Box::new(HyprExpression::Variable(monitor_var)),
                Operator::Multiply,
                Box::new(HyprExpression::Float(pct * 0.01)),
            ),
        }
    }
}

impl FromStr for HyprSize {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let mut result = HyprSize {
            width: PixelOrPercent::Percent(50.0),
            height: PixelOrPercent::Percent(50.0),
            width_bound: SizeBound::Exact,
            height_bound: SizeBound::Exact,
        };

        let parts: Vec<&str> = s.split(' ').collect();

        let width = parts.first().unwrap_or(&"");
        let height = parts.get(1).unwrap_or(&"");

        if let Some(stripped) = width.strip_prefix("<") {
            result.width_bound = SizeBound::Max;
            result.width = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else if let Some(stripped) = width.strip_prefix(">") {
            result.width_bound = SizeBound::Min;
            result.width = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else {
            result.width = PixelOrPercent::from_str(width).unwrap_or_default();
        }

        if let Some(stripped) = height.strip_prefix("<") {
            result.height_bound = SizeBound::Max;
            result.height = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else if let Some(stripped) = height.strip_prefix(">") {
            result.height_bound = SizeBound::Min;
            result.height = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else {
            result.height = PixelOrPercent::from_str(height).unwrap_or_default();
        }

        Ok(result)
    }
}
