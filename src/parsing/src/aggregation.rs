use crate::arithmetic::Arithmetic;
use crate::{parser::Lexeme, Rule};
use pest::iterators::Pair;
use std::fmt;

/// Represents the different types of aggregation operations that can be performed
/// on data sets (e.g., finding minimum, maximum, count, or sum of values).
#[derive(Debug, Clone, Eq, Hash, PartialEq, Copy)]
pub enum AggregationOperator {
    /// Find the minimum value in a dataset
    Min,
    /// Find the maximum value in a dataset
    Max,
    /// Count the number of items in a dataset
    Count,
    /// Calculate the sum of all values in a dataset
    Sum,
}

impl fmt::Display for AggregationOperator {
    /// Formats the aggregation operator as a lowercase string for display purposes.
    /// This is useful for generating human-readable output or SQL-like syntax.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AggregationOperator::Min => write!(f, "min"),
            AggregationOperator::Max => write!(f, "max"),
            AggregationOperator::Count => write!(f, "count"),
            AggregationOperator::Sum => write!(f, "sum"),
        }
    }
}

impl Lexeme for AggregationOperator {
    /// Parses a pest parsing rule into an AggregationOperator enum variant.
    ///
    /// # Arguments
    /// * `parsed_rule` - A pest Pair containing the parsed aggregation operator rule
    ///
    /// # Panics
    /// Panics if the parsed rule doesn't match any expected aggregation operator rules.
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        // Extract the inner rule that contains the specific operator type
        let operator = parsed_rule.into_inner().next().unwrap();

        // Match the rule type to the corresponding enum variant
        match operator.as_rule() {
            Rule::min => AggregationOperator::Min,
            Rule::max => AggregationOperator::Max,
            Rule::count => AggregationOperator::Count,
            Rule::sum => AggregationOperator::Sum,
            _ => unreachable!(), // Should never reach here if grammar is correct
        }
    }
}

/// Represents a complete aggregation expression consisting of an operator
/// and the arithmetic expression it operates on.
///
/// Examples: `sum(x + y)`, `max(price * quantity)`, `count(id)`
#[derive(Debug, Clone)]
pub struct Aggregation {
    /// The aggregation operation to perform (min, max, count, sum)
    operator: AggregationOperator,
    /// The arithmetic expression to aggregate over
    arithmetic: Arithmetic,
}

impl fmt::Display for Aggregation {
    /// Formats the aggregation as "operator(arithmetic_expression)".
    ///
    /// # Examples
    /// - `sum(x + y)`
    /// - `max(price * 0.8)`
    /// - `count(user_id)`
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.operator, self.arithmetic)
    }
}

impl Aggregation {
    /// Returns a vector of references to all variable names used in the arithmetic expression.
    /// This is useful for dependency analysis and query planning.
    ///
    /// # Returns
    /// A vector containing references to all variable names in the arithmetic expression
    pub fn vars(&self) -> Vec<&String> {
        self.arithmetic.vars()
    }

    /// Returns a reference to the arithmetic expression being aggregated.
    ///
    /// # Returns
    /// A reference to the internal `Arithmetic` expression
    pub fn arithmetic(&self) -> &Arithmetic {
        &self.arithmetic
    }

    /// Returns a reference to the aggregation operator.
    ///
    /// # Returns
    /// A reference to the `AggregationOperator` (min, max, count, or sum)
    pub fn operator(&self) -> &AggregationOperator {
        &self.operator
    }
}

impl Lexeme for Aggregation {
    /// Parses a pest parsing rule into a complete Aggregation struct.
    ///
    /// Expected rule structure: aggregation_operator followed by arithmetic_expression
    ///
    /// # Arguments
    /// * `parsed_rule` - A pest Pair containing the parsed aggregation rule
    ///
    /// # Returns
    /// A new `Aggregation` instance with the parsed operator and arithmetic expression
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let mut inner_rules = parsed_rule.into_inner();

        // Parse the aggregation operator (first inner rule)
        let operator = AggregationOperator::from_parsed_rule(inner_rules.next().unwrap());

        // Parse the arithmetic expression (second inner rule)
        let arithmetic = Arithmetic::from_parsed_rule(inner_rules.next().unwrap());

        Self {
            operator,
            arithmetic,
        }
    }
}
