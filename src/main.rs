use env_logger::fmt::Color;
use failure::{Fallible, ResultExt};
use log::{Level, LevelFilter};
use simple_stock_matcher_experiment::{process_reader, OrderBook};
use std::{fs::File, io::Write, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "Simple stock matcher experiment.")]
struct Config {
    #[structopt(long = "input", short = "i", help = "Path to a yaml file with bids.")]
    bids_path: PathBuf,
    #[structopt(long = "verbose", short = "v", help = "Enable debug output.")]
    verbose: bool,
}

fn init_logging(verbose: bool) {
    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    env_logger::Builder::new()
        .filter_level(level)
        .format(|formatter, record| match record.level() {
            Level::Info => writeln!(formatter, "{}", record.args()),
            _ => {
                let mut level_style = formatter.style();
                level_style.set_color(Color::Cyan);
                writeln!(
                    formatter,
                    "[{:5}] {}",
                    level_style.value(record.level()),
                    record.args()
                )
            }
        })
        .init()
}

fn main() -> Fallible<()> {
    let args = Config::from_args();
    init_logging(args.verbose);
    let input = File::open(&args.bids_path)
        .with_context(|e| format!("Can't read {:?}: {}", args.bids_path, e))?;
    let mut order_book = OrderBook::empty();
    process_reader(&mut order_book, input)
        .with_context(|e| format!("Can't process {:?}: {}", args.bids_path, e))?;
    Ok(())
}
