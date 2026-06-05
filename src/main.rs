mod error;
mod exec;
mod rule;

use crate::{
    error::ExpertSystemError,
    exec::{ExecMode, exec},
};
use clap::{CommandFactory as _, Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq)]
enum InferenceEngine {
    #[value(alias("sat"))]
    SatSolver,
    #[value(alias("backward"))]
    BackwardChaining,
    // TODO: ForwardChaining
}

#[derive(Debug, Parser)]
struct Args {
    /// Path to a file containing rules, facts and queries.
    path: Option<PathBuf>,
    /// Launch an interactive shell.
    #[arg(short, long)]
    interactive: bool,
    /// Select an inference engine.
    #[arg(short, long, value_enum, default_value_t = InferenceEngine::BackwardChaining)]
    engine: InferenceEngine,
}

fn main() {
    let args = Args::parse();
    let exec_result = match (args.path, args.interactive) {
        (None, false) => {
            let mut cmd = Args::command();
            eprintln!("Error: either provide a file path or use --interactive\n");
            cmd.print_help().unwrap();
            std::process::exit(2);
        }
        (None, true) => exec(&ExecMode::InteractiveWithoutFile, args.engine),
        (Some(path), false) => exec(&ExecMode::OnlyFile(path), args.engine),
        (Some(path), true) => exec(&ExecMode::InteractiveWithFile(path), args.engine),
    };
    if let Err(err) = exec_result {
        eprintln!("Error: {err}");
    }
}
