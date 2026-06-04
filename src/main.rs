mod program;
mod rule;

use crate::program::{ParseProgramError, Program};
use clap::Parser;
use itertools::Itertools as _;
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
    let mut program = Program::parse(reader)?;
    match program.solve() {
        Some(results) => {
            for (fact, value) in results.iter().sorted() {
                println!("{fact} is {value:?}");
            }
        }
        None => println!("There is a contradiction in the rules."),
    }
    Ok(())
}
