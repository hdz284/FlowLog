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
    csvs: String,

    /// evaluate w/o spilling the output
    // #[arg(short, long)]
    // evaluation_only: bool,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// delimiter
    #[arg(short, long, default_value = ",")]
    delimiter: String,

    /// is optimized
    #[arg(short, long, default_value_t = false)]
    issip: bool,

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

    pub fn csvs(&self) -> String {
        (&self.csvs).to_owned()
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    // pub fn evaluation_only(&self) -> bool {
    //     self.evaluation_only
    // }

    pub fn delimiter(&self) -> &String {
        &self.delimiter
    }

    pub fn is_global_optimized(&self) -> bool {
        self.issip
    }

    pub fn timely_args(&self) -> Vec<String> {
        vec![
            String::from("-w"),
            String::from(format!("{}", &self.workers)),
        ]
    }
}



pub fn test() {
    let args = Args::parse();
    println!("{:?}", args);
}