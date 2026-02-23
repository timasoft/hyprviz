use super::{HyprVariable, Operator};
use crate::{
    gtk_converters::{
        EnumConfigForGtk, FieldLabel, PLUG_SEPARATOR, ToGtkBoxWithSeparatorAndNames,
        ToGtkBoxWithSeparatorAndNamesBuilder, create_spin_button_builder,
    },
    register_togtkbox, register_togtkbox_with_separator_names,
    utils::{HasDiscriminant, MAX_SAFE_STEP_0_01_F64, strip_outer_parens},
};
use gtk::StringList;
use rust_i18n::t;
use std::{fmt::Display, str::FromStr};
use strum::{EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(HyprExpressionDiscriminant))]
pub enum HyprExpression {
    Float(f64),
    Uint(u32),
    Variable(HyprVariable),
    Formula(Box<HyprExpression>, Operator, Box<HyprExpression>),
}

impl HasDiscriminant for HyprExpression {
    type Discriminant = HyprExpressionDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            HyprExpressionDiscriminant::Float => HyprExpression::Float(0.0),
            HyprExpressionDiscriminant::Uint => HyprExpression::Uint(0),
            HyprExpressionDiscriminant::Variable => {
                HyprExpression::Variable(HyprVariable::default())
            }
            HyprExpressionDiscriminant::Formula => {
                HyprExpression::Formula(Box::default(), Operator::default(), Box::default())
            }
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            HyprExpressionDiscriminant::Float => {
                HyprExpression::Float(str.parse::<f64>().unwrap_or(0.0))
            }
            HyprExpressionDiscriminant::Uint => {
                HyprExpression::Uint(str.parse::<u32>().unwrap_or(0))
            }
            HyprExpressionDiscriminant::Variable => {
                HyprExpression::Variable(str.parse::<HyprVariable>().unwrap_or_default())
            }
            HyprExpressionDiscriminant::Formula => {
                match HyprExpression::from_str(str).unwrap_or_default() {
                    HyprExpression::Formula(left, op, right) => {
                        HyprExpression::Formula(left, op, right)
                    }
                    HyprExpression::Uint(v) => HyprExpression::Formula(
                        Box::new(HyprExpression::Uint(v)),
                        Operator::default(),
                        Box::default(),
                    ),
                    HyprExpression::Float(v) => HyprExpression::Formula(
                        Box::new(HyprExpression::Float(v)),
                        Operator::default(),
                        Box::default(),
                    ),
                    HyprExpression::Variable(v) => HyprExpression::Formula(
                        Box::new(HyprExpression::Variable(v)),
                        Operator::default(),
                        Box::default(),
                    ),
                }
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            HyprExpression::Float(v) => Some(v.to_string()),
            HyprExpression::Uint(v) => Some(v.to_string()),
            HyprExpression::Variable(v) => Some(v.to_string()),
            HyprExpression::Formula(left, op, right) => Some(format!("({left}{op}{right})")),
        }
    }

    fn custom_split(discriminant: Self::Discriminant) -> Option<fn(&str) -> Vec<&str>> {
        match discriminant {
            HyprExpressionDiscriminant::Formula => Some(|s| {
                let parse_str = strip_outer_parens(s).unwrap_or(s);

                let mut depth = 0;
                let mut split_pos = None;

                for (i, c) in parse_str.chars().enumerate() {
                    match c {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        '+' | '-' if depth == 0 => {
                            split_pos = Some(i);
                        }
                        _ => {}
                    }
                }

                if split_pos.is_none() {
                    depth = 0;
                    for (i, c) in parse_str.chars().enumerate() {
                        match c {
                            '(' => depth += 1,
                            ')' => depth -= 1,
                            '*' | '/' if depth == 0 => {
                                split_pos = Some(i);
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(pos) = split_pos {
                    let (left, right) = parse_str.split_at(pos);
                    let op_char = &parse_str[pos..pos + 1];
                    let right = &right[1..];
                    vec![left.trim(), op_char, right.trim()]
                } else {
                    vec![parse_str.trim(), "", ""]
                }
            }),
            _ => None,
        }
    }
}

impl Default for HyprExpression {
    fn default() -> Self {
        HyprExpression::Uint(0)
    }
}

impl FromStr for HyprExpression {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let try_parse_base = |s: &str| -> Option<HyprExpression> {
            if let Ok(v) = s.parse::<u32>() {
                return Some(HyprExpression::Uint(v));
            }
            if let Ok(v) = s.parse::<f64>() {
                return Some(HyprExpression::Float(v));
            }
            if let Ok(v) = s.parse::<HyprVariable>() {
                return Some(HyprExpression::Variable(v));
            }
            None
        };

        if let Some(expr) = try_parse_base(s) {
            return Ok(expr);
        }

        if let Some(stripped) = strip_outer_parens(s)
            && let Some(expr) = try_parse_base(stripped)
        {
            return Ok(expr);
        }

        let parse_str = strip_outer_parens(s).unwrap_or(s);

        let find_split = |s: &str, ops: &[char]| -> Option<(usize, char)> {
            let mut depth: i32 = 0;
            let mut last_pos = None;
            let mut last_op = None;

            for (byte_idx, c) in s.char_indices() {
                match c {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ if depth == 0 && ops.contains(&c) => {
                        last_pos = Some(byte_idx);
                        last_op = Some(c);
                    }
                    _ => {}
                }

                if depth < 0 {
                    return None;
                }
            }

            if depth != 0 {
                return None;
            }

            match (last_pos, last_op) {
                (Some(pos), Some(op)) => Some((pos, op)),
                _ => None,
            }
        };

        let (split_pos, op_char) = if let Some(res) = find_split(parse_str, &['+', '-']) {
            res
        } else if let Some(res) = find_split(parse_str, &['*', '/']) {
            res
        } else {
            return Err("Failed to parse expression");
        };

        let (left_str, right_part) = parse_str.split_at(split_pos);

        let op_len = op_char.len_utf8();
        let right_str = &right_part[op_len..];

        let left = left_str.trim();
        let right = right_str.trim();

        if left.is_empty() || right.is_empty() {
            return Err("Missing operand");
        }

        let left_expr = HyprExpression::from_str(left)?;
        let right_expr = HyprExpression::from_str(right)?;

        let operator = op_char
            .to_string()
            .parse::<Operator>()
            .map_err(|_| "Invalid operator")?;

        Ok(HyprExpression::Formula(
            Box::new(left_expr),
            operator,
            Box::new(right_expr),
        ))
    }
}

impl Display for HyprExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprExpression::Float(v) => write!(f, "{v}"),
            HyprExpression::Uint(v) => write!(f, "{v}"),
            HyprExpression::Variable(v) => write!(f, "{v}"),
            HyprExpression::Formula(left, op, right) => {
                write!(f, "({left}{op}{right})")
            }
        }
    }
}

impl EnumConfigForGtk for HyprExpression {
    fn dropdown_items() -> StringList {
        StringList::new(&[
            &t!("hyprland.hypr_expression.float"),
            &t!("hyprland.hypr_expression.uint"),
            &t!("hyprland.hypr_expression.variable"),
            &t!("hyprland.hypr_expression.formula"),
        ])
    }

    const SEPARATOR: Option<char> = Some(PLUG_SEPARATOR);

    fn parameter_builder(&self) -> Option<ToGtkBoxWithSeparatorAndNamesBuilder> {
        match self {
            HyprExpression::Float(_float) => Some(|entry, _, names, _| {
                create_spin_button_builder(0.0, MAX_SAFE_STEP_0_01_F64, 0.01)(
                    entry,
                    names.first().unwrap_or(&FieldLabel::Unnamed),
                )
            }),
            HyprExpression::Uint(_uint) => Some(<(u32,)>::to_gtk_box),
            HyprExpression::Variable(_variable) => Some(<(HyprVariable,)>::to_gtk_box),
            HyprExpression::Formula(_left, _op, _right) => {
                Some(<(HyprExpression, Operator, HyprExpression)>::to_gtk_box)
            }
        }
    }
}

register_togtkbox!(HyprExpression);
register_togtkbox_with_separator_names!(
    (HyprVariable,),
    (HyprExpression, Operator, HyprExpression)
);
