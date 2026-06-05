mod error;
mod exec;
mod rule;

use crate::{
    error::ExpertSystemError,
    exec::{ExecMode, exec},
};
use clap::{CommandFactory as _, Parser};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Quadrulean {
    Contradiction, // TODO: check useful
    False,
    True,
    Ambiguous,
}
impl Quadrulean {
    const fn new(can_be_false: bool, can_be_true: bool) -> Self {
        match (can_be_false, can_be_true) {
            (false, false) => Self::Contradiction,
            (true, false) => Self::False,
            (false, true) => Self::True,
            (true, true) => Self::Ambiguous,
        }
    }
}

#[derive(Debug, Parser)]
struct Args {
    filename: Option<PathBuf>,
    #[arg(short, long)]
    interactive: bool,
    // TODO
    //#[arg(long)]
    //ambiguous: bool,
}

fn main() {
    let args = Args::parse();
    let exec_result = match (args.filename, args.interactive) {
        (None, false) => {
            let mut cmd = Args::command();
            eprintln!("error: either provide a filename or use --interactive\n");
            cmd.print_help().unwrap();
            std::process::exit(2);
        }
        (None, true) => exec(&ExecMode::InteractiveWithoutFile),
        (Some(filename), false) => exec(&ExecMode::OnlyFile(filename)),
        (Some(filename), true) => exec(&ExecMode::InteractiveWithFile(filename)),
    };
    if let Err(err) = exec_result {
        eprintln!("Error: {err}");
    }
}
