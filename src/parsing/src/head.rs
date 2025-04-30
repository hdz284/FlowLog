use crate::{arithmetic::Arithmetic, parser::Lexeme, Rule};
use pest::iterators::Pair;
use std::fmt;

#[derive(Debug, Clone)]
pub enum HeadArg {
    Var(String),
    Arith(Arithmetic),
    // GroupBy(Arithmetic),
}

impl HeadArg {
    pub fn vars(&self) -> Vec<&String> {
        match self {
            Self::Var(var) => vec![var],
            Self::Arith(arith) => arith.vars(),
            // Self::GroupBy(arith) => todo!("GroupBy unimplemented"),
        }
    }
}

impl fmt::Display for HeadArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(var) => write!(f, "{}", var),
            Self::Arith(arith) => write!(f, "{}", arith),
            // Self::GroupBy(arith) => write!(f, "{}", arith),
        }
    }
}

impl Lexeme for HeadArg {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let arithmic = 
            match parsed_rule.as_rule() {
                Rule::arithmics => Arithmetic::from_parsed_rule(parsed_rule), 
                // (subsumed by arithmics) Rule::variable => Self::Var(parsed_rule.as_str().to_string()), // to_string() copies the string
                Rule::aggregate => todo!("GroupBy unimplemented yet"),
                _ => unreachable!(),
            };

        if arithmic.is_var() {
            Self::Var(arithmic.init().vars()[0].to_string()) // parse as a variable (e.g. x, y, z)
        } else {
            Self::Arith(arithmic) // parse as an arithmetic expression (e.g. x + y, x * y, x - y)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Head {
    name: String,
    head_arguments: Vec<HeadArg>,
}

impl fmt::Display for Head {
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
    pub fn new(name: String, head_arguments: Vec<HeadArg>) -> Self {
        Self {
            name,
            head_arguments,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn head_arguments(&self) -> &Vec<HeadArg> {
        &self.head_arguments
    }

    pub fn arity(&self) -> usize {
        self.head_arguments.len()
    }
}

impl Lexeme for Head {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let mut inner_rules = parsed_rule.into_inner();
        let name = inner_rules.next().unwrap().as_str().to_string();

        let head_arguments = inner_rules
            .map(|head_arg| HeadArg::from_parsed_rule(head_arg))
            .collect::<Vec<HeadArg>>();

        Self::new(name, head_arguments)
    }
}
