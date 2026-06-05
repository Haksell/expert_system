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
enum Troolean {
    False,
    Ambiguous,
    True,
}

#[derive(Debug, Parser)]
struct Args {
    filename: Option<PathBuf>,
    #[arg(short, long)]
    interactive: bool,
}

fn main() -> Result<(), ExpertSystemError> {
    let args = Args::parse();
    match (args.filename, args.interactive) {
        (None, false) => {
            let mut cmd = Args::command();
            eprintln!("error: either provide a filename or use --interactive\n");
            cmd.print_help()?;
            std::process::exit(2);
        }
        (None, true) => exec(&ExecMode::InteractiveWithoutFile)?,
        (Some(filename), true) => exec(&ExecMode::OnlyFile(filename))?,
        (Some(filename), false) => exec(&ExecMode::InteractiveWithFile(filename))?,
    }
    Ok(())
}
