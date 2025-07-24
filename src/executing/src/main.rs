use catalog::head::idb_catalog_from_program;
use clap::Parser as ClapParser;

use debugging::debugger;
use executing::arg::Args;
use executing::dataflow::program_execution;
use mimalloc::MiMalloc;
use planning::program::ProgramQueryPlan;
use reading::{KV_MAX, ROW_MAX};
use strata::stratification::Strata;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    /* initialize tracing subscriber for logging */
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    /* CL args parsing */
    let args = Args::parse();

    debugger::display_info("Arguments", false, format!("{:#?}", args));

    /* (1) program parsing */
    let program = parsing::parser::Program::from_str(args.program());

    debugger::display_info("Parsed Program", false, format!("{}", program));

    /* (2) stratification */
    let strata = Strata::from_parser(program.clone());

    debugger::display_info(
        "Strata",
        false,
        format!("{}\n{}", strata.dependency_graph(), strata),
    );

    /* (3) planning (catalog and query plan) */
    let program_query_plan =
        ProgramQueryPlan::from_strata(&strata, args.no_sharing(), args.opt_level());

    debugger::display_info(
        "Program Query Plans",
        true,
        format!("{}", program_query_plan),
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
    );

    /* Determine if fat mode should be used based on arity and user preference */
    let use_fat_mode = program_query_plan.should_use_fat_mode(args.fat_mode(), KV_MAX, ROW_MAX);

    /* If fat mode was forced due to high arity, inform the user */
    if use_fat_mode && !args.fat_mode() {
        warn!("WARNING: Fat mode automatically enabled due to high arity");
        warn!(
            "         Maximal incomparable arity pairs found: {:?}",
            program_query_plan.maximal_arity_pairs()
        );
    }

    let idb_map = idb_catalog_from_program(&program);

    /* (4) executing (dataflow) */
    program_execution(
        args,
        strata,
        program_query_plan.program_plan().to_owned(),
        use_fat_mode,
        idb_map,
    );

    info!("success query");
}

// ./target/debug/executing -p ./examples/programs/tc.dl -f ./examples/facts -c ./examples/csvs -v
