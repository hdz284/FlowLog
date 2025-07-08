use parsing::parser::Lexeme;
use parsing::{FlowLogParser, Parser, Rule};
use std::fs;
use tracing::info;

use planning::program::ProgramQueryPlan;
use strata::stratification::Strata;
use tracing_subscriber::EnvFilter;
// use strata::dependencies::DependencyGraph;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let program_source = "./examples/programs/ddisasm-test.dl";
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

    debugging::debugger::display_info("Strata (Topological Order)", true, format!("{}\n", strata));

    // planning
    let program_query_plan = ProgramQueryPlan::from_strata(&strata, false, None);

    debugging::debugger::display_info(
        "Program Query Plans",
        true,
        format!("{}", program_query_plan),
    );

    /* arity analysis */
    debugging::debugger::display_info(
        "Arity Checks",
        false,
        format!(
            "Maximum arity required: {}\nMax arities per transformation:\n{}",
            program_query_plan.max_arity(),
            program_query_plan
                .arity_analysis()
                .iter()
                .map(|(name, inputs, output)| format!(
                    "  {} @ inputs: {:?} -> output: {:?}",
                    name, inputs, output
                ))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    );

    info!("success planning");
}
