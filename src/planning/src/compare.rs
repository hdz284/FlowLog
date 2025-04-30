use std::fmt;
use parsing::compare::ComparisonOperator;
use catalog::compare::ComparisonExprPos;
use crate::{arguments::TransformationArgument, arithmetic::ArithmeticArgument};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ComparisonExprArgument {
    left: ArithmeticArgument,
    operator: ComparisonOperator,
    right: ArithmeticArgument,
}

impl ComparisonExprArgument {
    pub fn from_comparison_expr(compare_expr: &ComparisonExprPos, left_arguments: &Vec<TransformationArgument>, right_arguments: &Vec<TransformationArgument>) -> Self {
        let left = ArithmeticArgument::from_arithmetic(compare_expr.left(), left_arguments);
        let right = ArithmeticArgument::from_arithmetic(compare_expr.right(), right_arguments);
        let operator = compare_expr.operator().clone();

        Self { left, operator, right }
    }

    pub fn operator(&self) -> &ComparisonOperator {
        &self.operator
    }

    pub fn left(&self) -> &ArithmeticArgument {
        &self.left
    }

    pub fn right(&self) -> &ArithmeticArgument {
        &self.right
    }

    pub fn transformation_arguments(&self) -> Vec<&TransformationArgument> {
        let mut transformation_arguments = self.left.transformation_arguments();
        transformation_arguments.extend(self.right.transformation_arguments());
        transformation_arguments
    }

    pub fn jn_flip(&self) -> Self {
        Self {
            left: self.left.jn_flip(),
            operator: self.operator.clone(),
            right: self.right.jn_flip(),
        }
    }
}


impl fmt::Display for ComparisonExprArgument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.operator, self.right)
    }
}