use parsing::{arithmetic::ArithmeticOperator, compare::ComparisonOperator};
use reading::row::Array;
use planning::compare::ComparisonExprArgument;
use planning::arithmetic::ArithmeticArgument;
use planning::arithmetic::FactorArgument;
use planning::arguments::TransformationArgument;


pub fn compare_ints(x: i32, op: &ComparisonOperator, y: i32) -> bool {
    match op {
        ComparisonOperator::Equals => x == y,
        ComparisonOperator::NotEquals => x != y,
        ComparisonOperator::GreaterThan => x > y,
        ComparisonOperator::GreaterEqualThan => x >= y,
        ComparisonOperator::LessThan => x < y,
        ComparisonOperator::LessEqualThan => x <= y,
    }
}

pub fn arithmetic_ints(init: i32, rest: &[(&ArithmeticOperator, i32)]) -> i32 {
    let mut result = init;
    for (op, value) in rest {
        match op {
            ArithmeticOperator::Plus => result += value,
            ArithmeticOperator::Minus => result -= value,
            ArithmeticOperator::Multiply => result *= value,
            ArithmeticOperator::Divide => result /= value,
            ArithmeticOperator::Modulo => result %= value,
        }
    }
    result
}
    

/* ------------------------------ */
/* compare for rows */
/* ------------------------------ */
pub fn factor_row(v: &dyn Array, factor: &FactorArgument) -> i32 {
    match factor {
        FactorArgument::Var(transformation_arg) => {
            match transformation_arg {
                TransformationArgument::KV((true, id)) => v.column(*id),
                _ => panic!("factor_row: expected a kv argument"),
            }
        }
        FactorArgument::Const(constant) => constant.integer(),
    }
}

pub fn arithmetic_row(v: &dyn Array, arithmetic_expr: &ArithmeticArgument) -> i32 {
    let init = factor_row(v, arithmetic_expr.init());
    let rest = arithmetic_expr.rest().iter().map(|(op, factor)| {
        (op, factor_row(v, factor))
    }).collect::<Vec<_>>();

    arithmetic_ints(init, &rest)
}

        
pub fn compare_row(v: &dyn Array, compare_expr: &ComparisonExprArgument) -> bool {
    let left = arithmetic_row(v, compare_expr.left());
    let right = arithmetic_row(v, compare_expr.right());
    compare_ints(left, compare_expr.operator(), right)
}
    


/* ---------------------------------------------- */
/* compare for joins (fused into joins) */
/* ---------------------------------------------- */
pub fn jn_compare_extractor(k: Option<&dyn Array>, v1: Option<&dyn Array>, v2: Option<&dyn Array>, extracts: &(bool, bool, usize)) -> i32 {
    let (left_or_right, key_or_value, id) = extracts;
    if !key_or_value {
        // from key
        match k {
            Some(k) => k.column(*id),
            None => panic!("jn_compare_extractor: missing key array"),
        }
    } else {
        // from value
        match (left_or_right, v1, v2) {
            (false, Some(v1), _) => v1.column(*id),  // from left if v1 is provided
            (true, _, Some(v2)) => v2.column(*id),   // from right if v2 is provided
            _ => panic!("jn_compare_extractor: bad arguments"),
        }
    }
}

pub fn jn_compare(k: Option<&dyn Array>, v1: Option<&dyn Array>, v2: Option<&dyn Array>, compare_expr: &ComparisonExprArgument) -> bool {
    let left = jn_arithmetic(k, v1, v2, compare_expr.left());
    let right = jn_arithmetic(k, v1, v2, compare_expr.right());
    compare_ints(left, compare_expr.operator(), right)
}

pub fn jn_arithmetic(k: Option<&dyn Array>, v1: Option<&dyn Array>, v2: Option<&dyn Array>, arithmetic_expr: &ArithmeticArgument) -> i32 {
    let init = jn_factor(k, v1, v2, arithmetic_expr.init());
    let rest = arithmetic_expr.rest().iter().map(|(op, factor)| {
        (op, jn_factor(k, v1, v2, factor))
    }).collect::<Vec<_>>();

    arithmetic_ints(init, &rest)
}

pub fn jn_factor(k: Option<&dyn Array>, v1: Option<&dyn Array>, v2: Option<&dyn Array>, factor: &FactorArgument) -> i32 {
    match factor {
        FactorArgument::Var(transformation_arg) => {
            match transformation_arg {
                TransformationArgument::Jn(extracts) => jn_compare_extractor(k, v1, v2, extracts),
                _ => panic!("jn_factor: expected a jn argument"),
            }
        }
        FactorArgument::Const(constant) => constant.integer(),
    }
}
