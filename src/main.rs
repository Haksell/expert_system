mod program;
mod rule;

use crate::program::{ParseProgramError, start};
use clap::Parser;
use std::{fs::File, io::BufReader, path::PathBuf};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Troolean {
    False,
    Ambiguous,
    True,
}

#[derive(Debug, Parser)]
struct Args {
    filename: PathBuf,
}

fn main() -> Result<(), ParseProgramError> {
    let args = Args::parse();
    let file = File::open(args.filename)?;
    let reader = BufReader::new(file);
    start(reader)?;
    Ok(())
}
