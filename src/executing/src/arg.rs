use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// path of the Datalog program
    #[arg(short, long)]
    program: String,

    /// direct path of the EDBs .facts
    #[arg(short, long)]
    facts: String,

    /// direct path of the IDBs .csv
    #[arg(short, long)]
    csvs: Option<String>,

    /// delimiter
    #[arg(short, long, default_value = ",")]
    delimiter: String,

    /// enable fat mode for larger arities (uses heap-allocated SmallVec)
    #[arg(long, default_value_t = false)]
    fat_mode: bool,

    /// disable common subexpression reuse to examine and compare the benefit of this reuse
    #[arg(long, default_value_t = false)]
    no_sharing: bool,

    /// timely arguments
    /// -w, --workers: number of per-process worker threads.
    #[arg(short, long, default_value_t = 1)]
    workers: usize,

    /// optimization Level
    /// 0: as is, 1: sip, 2: planning, 3: sip + planning
    #[arg(short = 'O', value_parser = clap::value_parser!(u8).range(0..=3))]
    opt_level: Option<u8>,
}

impl Args {
    pub fn program(&self) -> &String {
        &self.program
    }

    pub fn program_name(&self) -> String {
        std::path::Path::new(&self.program)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown_program".into())
    }

    pub fn facts(&self) -> String {
        (&self.facts).to_owned()
    }

    pub fn csvs(&self) -> Option<String> {
        self.csvs.clone()
    }

    pub fn delimiter(&self) -> &String {
        &self.delimiter
    }

    pub fn fat_mode(&self) -> bool {
        self.fat_mode
    }

    pub fn no_sharing(&self) -> bool {
        self.no_sharing
    }

    pub fn timely_args(&self) -> Vec<String> {
        vec![
            String::from("-w"),
            String::from(format!("{}", &self.workers)),
        ]
    }

    pub fn opt_level(&self) -> Option<u8> {
        self.opt_level
    }
}
