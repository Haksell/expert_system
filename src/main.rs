mod program;
mod rule;

use crate::program::{ParseProgramError, Program};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    filename: PathBuf,
}

fn main() -> Result<(), ParseProgramError> {
    let args = Args::parse();
    let program = Program::parse(&args.filename)?;
    println!("{program:#?}");
    Ok(())
}
