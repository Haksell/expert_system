mod program;
mod rule;

use crate::program::{ParseProgramError, Program};
use clap::Parser;
use std::{fs::File, io::BufReader, path::PathBuf};

#[derive(Debug, Parser)]
struct Args {
    filename: PathBuf,
}

fn main() -> Result<(), ParseProgramError> {
    let args = Args::parse();
    let file = File::open(args.filename)?;
    let reader = BufReader::new(file);
    let program = Program::parse(reader)?;
    println!("{program:#?}");
    Ok(())
}
