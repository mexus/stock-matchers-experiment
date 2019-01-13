use failure::{Fallible, ResultExt};
use simple_stock_matcher_experiment::{cup::BidsCup, process_reader};
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "Simple stock matcher experiment")]
struct Config {
    #[structopt(long = "input", short = "i", help = "Path to a yaml file with bids")]
    bids_path: PathBuf,
}

fn main() -> Fallible<()> {
    let args = Config::from_args();
    let input = File::open(&args.bids_path)
        .with_context(|e| format!("Can't read {:?}: {}", args.bids_path, e))?;
    let mut cup = BidsCup::new();
    process_reader(&mut cup, input)
        .with_context(|e| format!("Can't process {:?}: {}", args.bids_path, e))?;
    Ok(())
}
