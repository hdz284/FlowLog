use crate::{head::Head, parser::Lexeme, Rule};
use pest::iterators::Pair;
use crate::compare::ComparisonExpr;
use std::fmt;
use tracing::error;


/*
    Atom: NAME(AtomArg, AtomArg, ...)
    AtomArg: Var(String) | Const(Const) | Placeholder
    Const: Integer(i32) | Text(String)
*/

// atom_arg = var | const | placeholder
#[derive(Debug, Clone)]
pub enum AtomArg {
    Var(String),
    Const(Const),
    Placeholder,
}

impl AtomArg {
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const(_))
    }

    pub fn is_placeholder(&self) -> bool {
        matches!(self, Self::Placeholder)
    }

    pub fn as_var(&self) -> &String {
        match self {
            Self::Var(var) => var,
            _ => panic!("expects var: {:?}", self),
        }
    }
}

impl fmt::Display for AtomArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(var) => write!(f, "{}", var),
            Self::Const(constant) => write!(f, "{}", constant),
            Self::Placeholder => write!(f, "_"),
        }
    }
}

impl Lexeme for AtomArg {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        match parsed_rule.as_rule() {
            Rule::variable => Self::Var(parsed_rule.as_str().to_string()), // to_string() copies the string
            Rule::constant => Self::Const(Const::from_parsed_rule(parsed_rule)),
            Rule::placeholder => Self::Placeholder,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Const {
    Integer(i32),
    Text(String),
}

impl Const {
    pub fn integer(&self) -> i32 {
        match self {
            Self::Integer(int) => *int,
            _ => panic!("expects ints: {:?}", self),
        }
    }
}

impl fmt::Display for Const {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(int) => write!(f, "{}", int),
            Self::Text(text) => write!(f, "{}", text),
        }
    }
}

impl Lexeme for Const {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let inner = parsed_rule.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::integer => Self::Integer(inner.as_str().parse::<i32>().unwrap()),
            Rule::string => Self::Text(inner.as_str().to_string()),
            _ => { error!("constant parsing panic {:?}", inner); unreachable!() }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Atom {
    name: String,
    arguments: Vec<AtomArg>,
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.arguments
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Atom {
    pub fn from_str(name: &str, arguments: Vec<AtomArg>) -> Self {
        Self {
            name: name.to_string(),
            arguments,
        }
    }

    pub fn push_arg(&mut self, arg: AtomArg) {
        self.arguments.push(arg);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &Vec<AtomArg> {
        &self.arguments
    }

    pub fn arity(&self) -> usize {
        self.arguments.len()
    }
}

impl Lexeme for Atom {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let mut inner_rules = parsed_rule.into_inner();
        let name = inner_rules.next().unwrap().as_str(); // name of the atom
        // print!(".atom name = {:?}\n", name);
        // print!(".atom args = {:?}\n", inner_rules);

        let arguments = inner_rules
            .map(|arg| {
                let arg_inner = arg.into_inner().next().unwrap();
                AtomArg::from_parsed_rule(arg_inner)
            })
            .collect();

        Self::from_str(name, arguments)
    }
}

/*
    FLRule: <Head> :- <Predicate>, <Predicate>, ...
    Predicate: <Atom> | !<Atom> | <Comparison>
*/

#[derive(Debug, Clone)]
pub enum Predicate {
    AtomPredicate(Atom),
    NegatedAtomPredicate(Atom),
    ComparePredicate(ComparisonExpr),
}

impl Predicate {
    pub fn arguments(&self) -> Vec<&AtomArg> {
        match self {
            Self::AtomPredicate(atom) => atom.arguments().iter().collect(),
            Self::NegatedAtomPredicate(atom) => atom.arguments().iter().collect(),
            Self::ComparePredicate(_) => panic!("Predicate.arguments() on cmpr"),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::AtomPredicate(atom) => atom.name(),
            Self::NegatedAtomPredicate(atom) => atom.name(),
            Self::ComparePredicate(_) => panic!("Predicate.name() on cmpr"),
        }
    }
}


impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AtomPredicate(atom) => write!(f, "{}", atom),
            Self::NegatedAtomPredicate(atom) => write!(f, "!{}", atom),
            Self::ComparePredicate(expr) => write!(f, "{}", expr),
        }
    }
}

impl Lexeme for Predicate {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        match parsed_rule.as_rule() {
            Rule::atom => {
                let atom = Atom::from_parsed_rule(parsed_rule);
                Self::AtomPredicate(atom)
            }
            Rule::neg_atom => {
                // an extra layer of parsing for negation to the atom level (neg_atom >> { "!" ~ atom})
                let inner_rule = parsed_rule.into_inner().next().unwrap();
                let negated_atom = Atom::from_parsed_rule(inner_rule);
                Self::NegatedAtomPredicate(negated_atom)
            }
            Rule::compare_expr => {
                let compare_expr = ComparisonExpr::from_parsed_rule(parsed_rule);
                Self::ComparePredicate(compare_expr)
            }
            _ => unreachable!(),
        }
    }
}



/*
    FLRule: <Head> :- <Predicate>, <Predicate>, ...
*/
#[derive(Debug, Clone)]
pub struct FLRule {
    head: Head,
    rhs: Vec<Predicate>,
    is_planning: bool,
    is_sip: bool,
}

impl fmt::Display for FLRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} :- {}.",
            self.head,
            self.rhs
                .iter()
                .map(|pred| pred.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl FLRule {
    pub fn new(head: Head, rhs: Vec<Predicate>, is_planning: bool, is_sip: bool) -> Self {
        Self { head, rhs, is_planning, is_sip }
    }

    pub fn head(&self) -> &Head {
        &self.head
    }

    pub fn rhs(&self) -> &Vec<Predicate> {
        &self.rhs
    }

    pub fn is_planning(&self) -> bool {
        self.is_planning
    }

    pub fn is_sip(&self) -> bool {
        self.is_sip
    }

    pub fn get(&self, i: usize) -> &Predicate {
        &self.rhs[i]
    }
}

impl Lexeme for FLRule {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let mut inner_rules = parsed_rule.into_inner();

        /* parsing the head */
        let head = Head::from_parsed_rule(inner_rules.next().unwrap());
        /* parsing the rhs */
        let rhs = inner_rules
            .next()
            .unwrap()
            .into_inner()
            .map(|pred| {
                let pred_inner = pred.into_inner().next().unwrap();
                /* parsing the predicate */
                Predicate::from_parsed_rule(pred_inner)
            })
            .collect();

        // if inner has next, print it
        match inner_rules.next() {
            Some(next) => match next.as_str() {
                ".plan" => Self::new(head, rhs, true, false),
                ".sip" => Self::new(head, rhs, false, true),
                ".optimize" => Self::new(head, rhs, true, true),
                _ => unreachable!(),
            },
            None => Self::new(head, rhs, false, false),
        }
    }
}
