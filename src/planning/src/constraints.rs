use std::fmt;
use std::sync::Arc;
use parsing::rule::Const;
// use crate::collections::{Collection, CollectionSignature};
// use crate::flow::TransformationFlow;
use crate::arguments::TransformationArgument;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct BaseConstraints {
    constant_eq_constraints: Arc<Vec<(TransformationArgument, Const)>>,
    variable_eq_constraints: Arc<Vec<(TransformationArgument, TransformationArgument)>>,
}

impl BaseConstraints {
    pub fn new(constant_eq_constraints: Vec<(TransformationArgument, Const)>, variable_eq_constraints: Vec<(TransformationArgument, TransformationArgument)>) -> Self {
        Self {
            constant_eq_constraints: Arc::new(constant_eq_constraints),
            variable_eq_constraints: Arc::new(variable_eq_constraints),
        }
    }

    pub fn constant_eq_constraints(&self) -> &Arc<Vec<(TransformationArgument, Const)>> {
        &self.constant_eq_constraints
    }

    pub fn variable_eq_constraints(&self) -> &Arc<Vec<(TransformationArgument, TransformationArgument)>> {
        &self.variable_eq_constraints
    }

    pub fn is_empty(&self) -> bool {
        self.constant_eq_constraints.is_empty() && self.variable_eq_constraints.is_empty()
    }
}

impl fmt::Display for BaseConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut constraints = Vec::new();

        // format constant constraints like `x = 3`
        for (arg, constant) in self.constant_eq_constraints.iter() {
            constraints.push(format!("{} = {}", arg, constant));
        }

        // format variable equality constraints like `y = x`
        for (arg1, arg2) in self.variable_eq_constraints.iter() {
            constraints.push(format!("{} = {}", arg1, arg2));
        }

        // join all constraints with ", " and write them on a single line
        write!(f, "{}", constraints.join(", "))
    }
}
