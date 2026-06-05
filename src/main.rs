mod error;
mod exec;
mod rule;

use crate::{
    error::ExpertSystemError,
    exec::{ExecMode, exec},
};
use clap::{CommandFactory as _, Parser};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Troolean {
    False,
    Ambiguous,
    True,
}

impl std::fmt::Display for Troolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::False => write!(f, "false"),
            Self::Ambiguous => write!(f, "ambiguous"),
            Self::True => write!(f, "true"),
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
