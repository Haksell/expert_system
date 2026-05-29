use clap::Parser;
use itertools::Itertools as _;
use std::{
    fs::File,
    io::{BufRead as _, BufReader},
    path::{Path, PathBuf},
};

use crate::ParseProgramError::InvalidFact;

#[derive(Debug, Parser)]
struct Args {
    filename: PathBuf,
}

// TODO: cleaner error messages through custom impl Debug
#[derive(Debug)]
enum ParseProgramError {
    IoError(#[expect(unused)] std::io::Error),
    InvalidLine(#[expect(unused)] String),
    InvalidFact(char),
    InvalidQuery(char),
    MissingFacts,
    MissingQueries,
}

#[derive(Debug)]
struct Rule {}

impl Rule {
    fn parse(line: &[char]) -> Result<Self, ParseProgramError> {
        Ok(Self {})
    }
}

impl From<std::io::Error> for ParseProgramError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[derive(Debug)]
struct Program {
    rules: Vec<Rule>,
    facts: Vec<char>,
    queries: Vec<char>,
}

impl Program {
    fn parse(filename: &Path) -> Result<Self, ParseProgramError> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let mut rules = Vec::new();
        let mut facts = None;
        let mut queries = None;

        for line in reader.lines() {
            let line = line?
                .chars()
                .filter(|c| !c.is_whitespace())
                .take_while(|&c| c != '#')
                .collect_vec();
            if line.is_empty() {
                continue;
            }
            match line[0] {
                'A'..='Z' => rules.push(Rule::parse(&line)?),
                '=' => {
                    facts = Some(Self::parse_chars(
                        &line[1..],
                        ParseProgramError::InvalidFact,
                    )?)
                }
                '?' => {
                    queries = Some(Self::parse_chars(
                        &line[1..],
                        ParseProgramError::InvalidQuery,
                    )?)
                }
                _ => return Err(ParseProgramError::InvalidLine(line.iter().collect())),
            }
        }

        let Some(facts) = facts else {
            return Err(ParseProgramError::MissingFacts);
        };
        let Some(queries) = queries else {
            return Err(ParseProgramError::MissingQueries);
        };

        Ok(Self {
            rules,
            facts,
            queries,
        })
    }

    fn parse_chars(
        line: &[char],
        error_fn: impl Fn(char) -> ParseProgramError,
    ) -> Result<Vec<char>, ParseProgramError> {
        let mut letters = Vec::with_capacity(line.len());
        for &c in line {
            match c {
                'A'..='Z' => letters.push(c),
                _ => return Err(error_fn(c)),
            }
        }
        Ok(letters)
    }
}

fn main() -> Result<(), ParseProgramError> {
    let args = Args::parse();
    let program = Program::parse(&args.filename)?;
    println!("{program:?}");
    Ok(())
}
