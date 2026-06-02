use clap::Parser;
use itertools::Itertools as _;
use std::{
    fs::File,
    io::{BufRead as _, BufReader},
    path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
struct Args {
    filename: PathBuf,
}

// TODO: cleaner error messages through custom impl Debug
#[derive(Debug)]
#[expect(unused)]
enum ParseProgramError {
    IoError(std::io::Error),
    InvalidLine(String),
    InvalidFact(char),
    InvalidQuery(char),
    MissingFacts,
    MissingQueries,
    DuplicateFacts,
    DuplicateQueries,
    QueriesBeforeFacts,
    RulesAfterFacts,
}

impl From<std::io::Error> for ParseProgramError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

enum BinaryOperation {
    And,
    Or,
    Xor,
}

enum Token {
    Fact(char),
    Not,
    BinaryOperation(BinaryOperation),
    LogicalLink(LogicalLink),
    LeftParenthesis,
    RightParenthesis,
}

// TODO: find a better name
#[derive(Debug)]
enum LogicalLink {
    Implication,
    Equivalence,
}

#[derive(Debug)]
enum Rule {
    Fact(char),
    Not(Box<Rule>),
    Or(Box<Rule>, Box<Rule>),
    And(Box<Rule>, Box<Rule>),
    Xor(Box<Rule>, Box<Rule>),
    Implication(Box<Rule>, Box<Rule>),
    Equivalence(Box<Rule>, Box<Rule>),
}

impl Rule {
    fn parse(line: &[char]) -> Result<Self, ParseProgramError> {
        let tokens = Self::tokenize(line)?;
        todo!()
    }

    fn tokenize(line: &[char]) -> Result<Vec<Token>, ParseProgramError> {
        Ok(Vec::new())
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
                'A'..='Z' | '!' => {
                    if facts.is_some() {
                        return Err(ParseProgramError::RulesAfterFacts);
                    }
                    rules.push(Rule::parse(&line)?);
                }
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
    println!("{program:?}");
    Ok(())
}
