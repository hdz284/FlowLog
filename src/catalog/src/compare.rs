use std::fmt;
use parsing::compare::{ComparisonExpr, ComparisonOperator};
use crate::{arithmetic::ArithmeticPos, atoms::AtomArgumentSignature};


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ComparisonExprPos {
    left: ArithmeticPos,
    operator: ComparisonOperator,
    right: ArithmeticPos,
}

impl ComparisonExprPos {
    pub fn from_comparison_expr(compare_expr: &ComparisonExpr, left_var_signatures: &Vec<AtomArgumentSignature>, right_var_signatures: &Vec<AtomArgumentSignature>) -> Self {
        let left = ArithmeticPos::from_arithmetic(compare_expr.left(), left_var_signatures);
        let right = ArithmeticPos::from_arithmetic(compare_expr.right(), right_var_signatures);
        let operator = compare_expr.operator().clone();

        Self { left, operator, right }
    }

    pub fn operator(&self) -> &ComparisonOperator {
        &self.operator
    }

    pub fn left(&self) -> &ArithmeticPos {
        &self.left
    }

    pub fn right(&self) -> &ArithmeticPos {
        &self.right
    }

    pub fn signatures(&self) -> Vec<&AtomArgumentSignature> {
        let mut signatures = self.left.signatures();
        signatures.extend(self.right.signatures());
        signatures
    }
}


impl fmt::Display for ComparisonExprPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{} {} {}]", self.left, self.operator, self.right)
    }
}