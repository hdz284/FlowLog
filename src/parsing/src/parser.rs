use pest::iterators::Pair;
use std::{fmt, fs};

use crate::decl::RelDecl; // crate :: the root of the module tree
use crate::rule::FLRule;
use crate::{FlowLogParser, Parser, Rule};

pub trait Lexeme {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self;
}

#[derive(Debug, Clone)]
pub struct Program {
    edbs: Vec<RelDecl>,
    idbs: Vec<RelDecl>,
    rules: Vec<FLRule>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let edbs = self
            .edbs
            .iter()
            .map(|rel_decl| rel_decl.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        let idbs = self
            .idbs
            .iter()
            .map(|rel_decl| rel_decl.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        let rules = self
            .rules
            .iter()
            .map(|rule| rule.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        write!(
            f,
            ".in \n{}\n.printsize \n{}\n.rule \n{}",
            edbs, idbs, rules
        )
    }
}

impl Program {
    pub fn new(edbs: Vec<RelDecl>, idbs: Vec<RelDecl>, rules: Vec<FLRule>) -> Self {
        Self { edbs, idbs, rules }
    }

    pub fn edbs(&self) -> &Vec<RelDecl> {
        &self.edbs
    }

    pub fn idbs(&self) -> &Vec<RelDecl> {
        &self.idbs
    }

    pub fn rules(&self) -> &Vec<FLRule> {
        &self.rules
    }

    pub fn from_str(path: &str) -> Self {
        let unparsed_str = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("can't read program from \"{}\"", path));

        let parsed_rule = FlowLogParser::parse(Rule::main_grammar, &unparsed_str)
            .unwrap_or_else(|error| {
                panic!(
                    "can't parse program from \"{}\": \n{:?}",
                    path, error
                )
            })
            .next()
            .unwrap();
        Self::from_parsed_rule(parsed_rule)
    }
}

impl Lexeme for Program {
    fn from_parsed_rule(parsed_rule: Pair<Rule>) -> Self {
        let mut inner_rules = parsed_rule.into_inner();
        let mut edbs: Vec<RelDecl> = Vec::new();
        let mut idbs: Vec<RelDecl> = Vec::new();
        let mut rules: Vec<FLRule> = Vec::new();

        fn parse_rel_decls(vec: &mut Vec<RelDecl>, rule: Pair<Rule>) {
            let mut rel_decls = rule.into_inner();
            while let Some(rel_decl) = rel_decls.next() {
                vec.push(RelDecl::from_parsed_rule(rel_decl));
            }
        }

        fn parse_rules(vec: &mut Vec<FLRule>, rule: Pair<Rule>) {
            let mut rules_iterator = rule.into_inner();
            while let Some(rule) = rules_iterator.next() {
                vec.push(FLRule::from_parsed_rule(rule));
            }
        }

        while let Some(inner_rule) = inner_rules.next() {
            match inner_rule.as_rule() {
                Rule::edb_decl => parse_rel_decls(&mut edbs, inner_rule),
                Rule::idb_decl => parse_rel_decls(&mut idbs, inner_rule),
                Rule::rule_decl => parse_rules(&mut rules, inner_rule),
                _ => {}
            }
        }

        Self { edbs, idbs, rules }
    }
}

