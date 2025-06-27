use clap::Parser as ClapParser;

use strata::stratification::Strata;
use planning::program::ProgramQueryPlan;   
use reading::FALLBACK_ARITY;
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
    let program_query_plan = ProgramQueryPlan::from_strata(&strata, args.no_sharing());

    debugger::display_info(
        "Program Query Plans", 
        true, 
        format!("{}", program_query_plan), 
        true
    );

    /* arity analysis */
    debugging::debugger::display_info(
        "Arity Checks",
        false,
        format!(
            "Maximum arity required: {}\nMaximal incomparable (key, value) arity pairs: {:?}\nMax arities per transformation:\n{}",
            program_query_plan.max_arity(),
            program_query_plan.maximal_arity_pairs(),
            program_query_plan.arity_analysis()
                .iter()
                .map(|(name, inputs, output)| format!("  {} @ inputs: {:?} -> output: {:?}", name, inputs, output))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        true
    );

    /* Determine if fat mode should be used based on arity and user preference */
    let use_fat_mode = program_query_plan.should_use_fat_mode(args.fat_mode(), FALLBACK_ARITY);
    
    /* If fat mode was forced due to high arity, inform the user */
    if use_fat_mode && !args.fat_mode() {
        println!("WARNING: Fat mode automatically enabled due to high arity (> {})", FALLBACK_ARITY);
        println!("         Maximal incomparable arity pairs found: {:?}", program_query_plan.maximal_arity_pairs());
    }
    
    /* (4) executing (dataflow) */
    program_execution(
        args,
        strata,
        program_query_plan.program_plan().to_owned(),
        use_fat_mode,
    );

    println!("success query");
}


// ./target/debug/executing -p ./examples/programs/tc.dl -f ./examples/facts -c ./examples/csvs -v