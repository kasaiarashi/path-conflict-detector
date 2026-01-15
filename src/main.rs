use clap::Parser;
use path_conflict_detector::cli::{Args, run};
use std::process;

fn main() {
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
