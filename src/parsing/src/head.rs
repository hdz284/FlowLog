use crate::{aggregation::Aggregation, arithmetic::Arithmetic, parser::Lexeme, Rule};
use pest::iterators::Pair;
use std::fmt;

/// Represents different types of arguments that can appear in a head expression.
/// Head arguments can be simple variables, arithmetic expressions, or aggregation functions.
///
/// # Examples
/// - Variable: `x`, `name`, `id`
/// - Arithmetic: `x + y`, `price * 0.8`, `count - 1`
/// - Aggregation: `sum(x)`, `max(price)`, `count(id)`
#[derive(Debug, Clone)]
pub enum HeadArg {
    /// A simple variable name (e.g., `x`, `name`, `user_id`)
    Var(String),
    /// An arithmetic expression (e.g., `x + y`, `price * tax_rate`)
    Arith(Arithmetic),
    /// An aggregation function (e.g., `sum(sales)`, `max(score)`)
    Aggregation(Aggregation),
}

impl HeadArg {
    /// Returns all variable names referenced in this head argument.
    ///
    /// # Returns
    /// A vector of references to variable names used in the argument
    ///
    /// # Examples
    /// - `Var("x")` returns `vec!["x"]`
    /// - `Arith(x + y)` returns `vec!["x", "y"]`
    /// - `Aggregation(sum(price))` returns `vec!["price"]`
    pub fn vars(&self) -> Vec<&String> {
        match self {
            Self::Var(var) => vec![var],
            Self::Arith(arith) => arith.vars(),
            Self::Aggregation(aggregation) => aggregation.vars(),
        }
    }
}

impl fmt::Display for HeadArg {
    /// Formats the head argument for display by delegating to the underlying type's Display implementation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(var) => write!(f, "{}", var),
            Self::Arith(arith) => write!(f, "{}", arith),
            Self::Aggregation(aggregation) => write!(f, "{}", aggregation),
        }
    }
}

impl Lexeme for HeadArg {
    /// Parses a pest rule into a HeadArg enum variant.
    ///
    /// The parsing logic handles the distinction between variables and arithmetic expressions
    /// by checking if an arithmetic expression is actually just a single variable.
    ///
    /// # Arguments
    /// * `parsed_rule` - A pest Pair containing the parsed head argument rule
    ///
    /// # Returns
    /// The appropriate HeadArg variant based on the parsed content
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        match parsed_rule.as_rule() {
            Rule::arithmics => {
                let arithmetic = Arithmetic::from_parsed_rule(parsed_rule);

                // Check if the arithmetic expression is actually just a single variable
                if arithmetic.is_var() {
                    // Extract the variable name and create a Var variant
                    Self::Var(arithmetic.init().vars()[0].to_string())
                } else {
                    // It's a complex arithmetic expression
                    Self::Arith(arithmetic)
                }
            }
            Rule::aggregate => {
                // Parse as an aggregation function
                Self::Aggregation(Aggregation::from_parsed_rule(parsed_rule))
            }
            _ => unreachable!(), // Should never reach here with correct grammar
        }
    }
}

/// Represents a head expression in a logic rule, consisting of a predicate name
/// and a list of arguments.
///
/// In logic programming, the head is the conclusion part of a rule.
///
/// # Examples
/// - `person(john, 25)` - predicate "person" with arguments "john" and "25"
/// - `salary(emp_id, sum(hours * rate))` - predicate with variable and aggregation
/// - `result(x + y)` - predicate with arithmetic expression
#[derive(Debug, Clone)]
pub struct Head {
    /// The name of the predicate (e.g., "person", "salary", "result")
    name: String,
    /// The list of arguments for this head expression
    head_arguments: Vec<HeadArg>,
}

impl fmt::Display for Head {
    /// Formats the head as "predicate_name(arg1, arg2, ...)".
    ///
    /// # Examples
    /// - `person(john, 25)`
    /// - `result(x + y, sum(z))`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let head_arguments = self
            .head_arguments
            .iter()
            .map(|head_arg| head_arg.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "{}({})", self.name, head_arguments)
    }
}

impl Head {
    /// Creates a new Head with the given predicate name and arguments.
    ///
    /// # Arguments
    /// * `name` - The predicate name
    /// * `head_arguments` - Vector of head arguments
    ///
    /// # Returns
    /// A new Head instance
    pub fn new(name: String, head_arguments: Vec<HeadArg>) -> Self {
        Self {
            name,
            head_arguments,
        }
    }

    /// Returns a reference to the predicate name.
    ///
    /// # Returns
    /// A reference to the predicate name string
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns a reference to the list of head arguments.
    ///
    /// # Returns
    /// A reference to the vector of HeadArg instances
    pub fn head_arguments(&self) -> &Vec<HeadArg> {
        &self.head_arguments
    }

    /// Returns the arity (number of arguments) of this head expression.
    ///
    /// # Returns
    /// The number of arguments in this head
    ///
    /// # Examples
    /// - `person(john, 25)` has arity 2
    /// - `result(x)` has arity 1
    /// - `empty()` has arity 0
    pub fn arity(&self) -> usize {
        self.head_arguments.len()
    }
}

impl Lexeme for Head {
    /// Parses a pest rule into a Head struct.
    ///
    /// Expected rule structure: predicate_name followed by zero or more head arguments.
    ///
    /// # Arguments
    /// * `parsed_rule` - A pest Pair containing the parsed head rule
    ///
    /// # Returns
    /// A new Head instance with the parsed predicate name and arguments
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let mut inner_rules = parsed_rule.into_inner();

        // First inner rule is the predicate name
        let name = inner_rules.next().unwrap().as_str().to_string();

        // Remaining inner rules are the head arguments
        let head_arguments = inner_rules
            .map(|head_arg| HeadArg::from_parsed_rule(head_arg))
            .collect::<Vec<HeadArg>>();

        Self::new(name, head_arguments)
    }
}
