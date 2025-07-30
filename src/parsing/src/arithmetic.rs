use crate::{parser::Lexeme, rule::Const, Rule};
use pest::iterators::Pair;
use std::collections::HashSet;
use std::fmt;

/** arithmic ops **/
// arithmic_op = { plus | minus | times | divide }
// plus = { "+" }
// minus = { "-" }
// times = { "*" }
// divide = { "/" }
// modulo = { "%" }
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ArithmeticOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
}

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &ArithmeticOperator::Plus => {
                write!(f, "+")
            }
            &ArithmeticOperator::Minus => {
                write!(f, "-")
            }
            &ArithmeticOperator::Multiply => {
                write!(f, "*")
            }
            &ArithmeticOperator::Divide => {
                write!(f, "/")
            }
            &ArithmeticOperator::Modulo => {
                write!(f, "%")
            }
        }
    }
}

impl Lexeme for ArithmeticOperator {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let operator = parsed_rule.into_inner().next().unwrap();
        match operator.as_rule() {
            Rule::plus => ArithmeticOperator::Plus,
            Rule::minus => ArithmeticOperator::Minus,
            Rule::times => ArithmeticOperator::Multiply,
            Rule::divide => ArithmeticOperator::Divide,
            Rule::modulo => ArithmeticOperator::Modulo,
            _ => unreachable!(),
        }
    }
}

// factor = { variable | constant }
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Factor {
    Var(String),
    Const(Const),
}

impl Factor {
    pub fn is_var(&self) -> bool {
        match self {
            Self::Var(_) => true,
            _ => false,
        }
    }

    pub fn vars_set(&self) -> HashSet<&String> {
        self.vars().into_iter().collect()
    }

    pub fn vars(&self) -> Vec<&String> {
        match self {
            Self::Var(var) => vec![var],
            _ => vec![],
        }
    }
}

impl fmt::Display for Factor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Factor::Var(var) => write!(f, "{}", var),
            Factor::Const(constant) => write!(f, "{}", constant),
        }
    }
}

impl Lexeme for Factor {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let inner = parsed_rule.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::variable => Self::Var(inner.as_str().to_string()), // to_string() copies the string
            Rule::constant => Self::Const(Const::from_parsed_rule(inner)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Arithmetic {
    init: Factor,
    rest: Vec<(ArithmeticOperator, Factor)>,
}

impl Arithmetic {
    pub fn new(init: Factor, rest: Vec<(ArithmeticOperator, Factor)>) -> Self {
        Self { init, rest }
    }

    pub fn init(&self) -> &Factor {
        &self.init
    }

    pub fn rest(&self) -> &Vec<(ArithmeticOperator, Factor)> {
        &self.rest
    }

    pub fn vars_set(&self) -> HashSet<&String> {
        self.vars().into_iter().collect()
    }

    pub fn vars(&self) -> Vec<&String> {
        let mut vec = self.init.vars();
        for (_, factor) in &self.rest {
            vec.extend(factor.vars_set().into_iter());
        }
        vec
    }

    // if it is a simple variable
    pub fn is_var(&self) -> bool {
        self.init.is_var() && self.rest.is_empty()
    }

    /// Returns a list of variable names in order of appearance (duplicates preserved).
    /// Ignores constants.
    pub fn variables_list_in_order(&self) -> Vec<String> {
        let mut vars = Vec::new();

        if let Factor::Var(v) = &self.init {
            vars.push(v.clone());
        }

        for (_, factor) in &self.rest {
            if let Factor::Var(v) = factor {
                vars.push(v.clone());
            }
        }

        vars
    }
}

impl fmt::Display for Arithmetic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let init = self.init.to_string();
        let rest = self
            .rest
            .iter()
            .map(|(op, factor)| format!("{} {}", op, factor))
            .collect::<Vec<String>>()
            .join(" ");

        if rest.is_empty() {
            write!(f, "{}", init)
        } else {
            write!(f, "{} {}", init, rest)
        }
    }
}

impl Lexeme for Arithmetic {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let mut inner_rules = parsed_rule.into_inner();
        let init = Factor::from_parsed_rule(inner_rules.next().unwrap());

        // consume every two next() calls as a pair (op, factor) until there is no more next()
        let mut rest = Vec::new();
        while let Some(op) = inner_rules.next() {
            let factor = Factor::from_parsed_rule(inner_rules.next().unwrap());
            rest.push((ArithmeticOperator::from_parsed_rule(op), factor));
        }

        Self { init, rest }
    }
}
