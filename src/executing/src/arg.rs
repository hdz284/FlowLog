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
}

impl Args {
    pub fn program(&self) -> &String {
        &self.program
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
}