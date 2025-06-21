use clap::Parser as ClapParser;

use strata::stratification::Strata;
use planning::program::ProgramQueryPlan;   
use executing::dataflow::program_execution;
use executing::arg::Args;
use debugging::debugger;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    /* CL args parsing */
    let args = Args::parse(); 
    
    debugger::display_info(
        "Arguments", 
        false, 
        format!("{:#?}", args), 
        args.verbose()
    );

    /* (1) program parsing */
    let program = parsing::parser::Program::from_str(args.program());

    debugger::display_info(
        "Parsed Program",
        false,
        format!("{}", program),
        args.verbose()
    );

    /* (2) stratification */
    let strata = Strata::from_parser(program);

    debugger::display_info(
        "Strata",
        false,
        format!("{}\n{}", strata.dependency_graph(), strata),
        args.verbose()
    );

    /* (3) planning (catalog and query plan) */
    let program_query_plan = ProgramQueryPlan::from_strata(&strata, args.is_global_optimized());

    debugger::display_info(
        "Program Query Plans", 
        true, 
        format!("{}", program_query_plan), 
        true
    );

    /* (4) executing (dataflow) */
    program_execution(
        args.verbose(),
        args.timely_args(),
        args.facts(),
        args.delimiter().as_bytes()[0],
        strata,
        program_query_plan.program_plan().to_owned(),
        args.csvs()
    );

    println!("success query");
}


// ./target/debug/executing -p ./examples/programs/tc.dl -f ./examples/facts -c ./examples/csvs -v