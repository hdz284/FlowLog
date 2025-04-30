use parsing::parser::Lexeme;
use parsing::{FlowLogParser, Parser, Rule};
use std::fs;

use strata::stratification::Strata;
// use analyzing::dependencies::DependencyGraph;

fn main() {
    let program_source = "./examples/programs/cspa.dl";
    let unparsed_str = fs::read_to_string(program_source)
        .unwrap_or_else(|_| panic!("can't read program from \"{}\"", program_source));

    let parsed_rule = FlowLogParser::parse(Rule::main_grammar, &unparsed_str)
        .unwrap_or_else(|error| {
            panic!(
                "can't parse program from \"{}\": \n{:?}",
                program_source, error
            )
        })
        .next()
        .unwrap();

    // .next() returns the first Pair in the iterator or None if there are no more Pairs
    // Pairs :: Vec<Pair> | Pair :: a matching pair of tokens from the input (https://docs.rs/pest/2.1.3/pest/iterators/struct.Pair.html)

    // print_rule(parsed_rule, 0); // print the parsed rule as a tree

    let program = parsing::parser::Program::from_parsed_rule(parsed_rule);

    // stratificaton
    // let graph = DependencyGraph::from_program(&program);
    let strata = Strata::from_parser(program);
    
    println!("{}", strata.dependency_graph());
    println!("{}", strata);

    println!("success parse");
}
