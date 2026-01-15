use clap::Parser;
use path_conflict_detector::cli::{run, Args};
use std::process;

fn main() {
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
