mod rule;

use clap::Parser;
use itertools::Itertools as _;
use std::{
    fs::File,
    io::{BufRead as _, BufReader},
    path::{Path, PathBuf},
};

use crate::rule::Rule;

#[derive(Debug, Parser)]
struct Args {
    filename: PathBuf,
}

// TODO: cleaner error messages through custom impl Debug
#[derive(Debug)]
enum ParseProgramError {
    IoError(#[expect(unused)] std::io::Error),
    InvalidToken(#[expect(unused)] String, #[expect(unused)] String), // line, token
    InvalidFact(#[expect(unused)] char),
    InvalidQuery(#[expect(unused)] char),
    MissingFacts,
    MissingQueries,
    DuplicateFacts,
    DuplicateQueries,
    QueriesBeforeFacts,
    RulesAfterFacts,
    UnbalancedParentheses,
    MissingImplication,
    MultipleImplications,
    BuildFailed,
    ParenthesesAroundImplication, // TODO: more specific
}

impl From<std::io::Error> for ParseProgramError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Token {
    Fact(char),
    Equivalence,
    Implication,
    ConverseImplication,
    Xor,
    Or,
    And,
    Not,
    LeftParenthesis,
    RightParenthesis,
}

impl Token {
    fn precedence(self) -> u32 {
        match self {
            Self::Equivalence | Self::ConverseImplication | Self::Implication => 1,
            Self::Xor => 2,
            Self::Or => 3,
            Self::And => 4,
            Self::Not => 5,
            Self::Fact(_) | Self::LeftParenthesis | Self::RightParenthesis => {
                unreachable!()
            }
        }
    }
}

#[derive(Debug)]
struct Program {
    rule: Rule,
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
                '=' => Self::parse_letters(
                    &mut facts,
                    &line[1..],
                    ParseProgramError::InvalidFact,
                    ParseProgramError::DuplicateFacts,
                )?,
                '?' => {
                    if facts.is_none() {
                        return Err(ParseProgramError::QueriesBeforeFacts);
                    }
                    Self::parse_letters(
                        &mut queries,
                        &line[1..],
                        ParseProgramError::InvalidQuery,
                        ParseProgramError::DuplicateQueries,
                    )?;
                }
                _ => {
                    if facts.is_some() {
                        return Err(ParseProgramError::RulesAfterFacts);
                    }
                    rules.push(Rule::parse(&line)?);
                    println!("{:?}", rules.last().unwrap());
                }
            }
        }

        let Some(facts) = facts else {
            return Err(ParseProgramError::MissingFacts);
        };
        let Some(queries) = queries else {
            return Err(ParseProgramError::MissingQueries);
        };

        Ok(Self {
            rule: Rule::merge(rules),
            facts,
            queries,
        })
    }

    fn parse_letters(
        letters_to_fill: &mut Option<Vec<char>>,
        line: &[char],
        invalid_fn: impl Fn(char) -> ParseProgramError,
        duplicate_fn: ParseProgramError,
    ) -> Result<(), ParseProgramError> {
        if letters_to_fill.is_some() {
            return Err(duplicate_fn);
        }
        let mut letters = Vec::with_capacity(line.len());
        for &c in line {
            match c {
                'A'..='Z' => letters.push(c),
                _ => return Err(invalid_fn(c)),
            }
        }
        letters_to_fill.replace(letters);
        Ok(())
    }
}

fn main() -> Result<(), ParseProgramError> {
    let args = Args::parse();
    let program = Program::parse(&args.filename)?;
    println!("{program:#?}");
    Ok(())
}
