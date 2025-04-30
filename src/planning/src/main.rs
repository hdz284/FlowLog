use parsing::parser::Lexeme;
use parsing::{FlowLogParser, Parser, Rule};
use std::fs;

use strata::stratification::Strata;
use planning::program::ProgramQueryPlan;   
// use strata::dependencies::DependencyGraph;

fn main() {
    let program_source = "./examples/programs/doop_help.dl";
    let unparsed_str = fs::read_to_string(program_source)
        .unwrap_or_else(|_| panic!("can't read program from \"{}\"", program_source));

    // parsing
    let parsed_rule = FlowLogParser::parse(Rule::main_grammar, &unparsed_str)
        .unwrap_or_else(|error| {
            panic!(
                "can't parse program from \"{}\": \n{:?}",
                program_source, error
            )
        })
        .next()
        .unwrap();

    // print_rule(parsed_rule, 0); // print the parsed rule as a tree
    let program = parsing::parser::Program::from_parsed_rule(parsed_rule);

    // stratificaton
    let strata = Strata::from_parser(program);
    
    debugging::debugger::display_info(
        "Strata (Topological Order)",
        true,
        format!("{}\n", strata),
        true
    );

    // planning
    let program_query_plan = ProgramQueryPlan::from_strata(&strata, true);

    debugging::debugger::display_info(
        "Program Query Plans", 
        true, 
        format!("{}", program_query_plan), 
        true);

    println!("success planning");
}
