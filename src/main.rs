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
    InvalidToken(String, String), // line, token
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

#[derive(Debug, PartialEq)]
enum Token {
    Fact(char),
    Not,
    And,
    Or,
    Xor,
    Implication,
    ConverseImplication,
    Equivalence,
    LeftParenthesis,
    RightParenthesis,
}

#[derive(Debug)]
enum Rule {
    Fact(char),
    Not(Box<Rule>),
    And(Box<Rule>, Box<Rule>),
    Or(Box<Rule>, Box<Rule>),
    Xor(Box<Rule>, Box<Rule>),
    Implication(Box<Rule>, Box<Rule>),
    Equivalence(Box<Rule>, Box<Rule>),
}

impl Rule {
    fn parse(line: &[char]) -> Result<Self, ParseProgramError> {
        let tokens = Self::tokenize(line)?;
        // Self::parse(&tokens)
        Ok(Self::Fact('Z'))
    }

    fn tokenize(line: &[char]) -> Result<Vec<Token>, ParseProgramError> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        for &c in line {
            if current_token.is_empty() {
                match c {
                    'A'..='Z' => tokens.push(Token::Fact(c)),
                    '(' => tokens.push(Token::LeftParenthesis),
                    ')' => tokens.push(Token::RightParenthesis),
                    '!' => tokens.push(Token::Not),
                    '+' | '&' => tokens.push(Token::And),
                    '|' => tokens.push(Token::Or),
                    '^' => tokens.push(Token::Xor),
                    '=' | '<' => current_token.push(c),
                    '>' if tokens.last() == Some(&Token::ConverseImplication) => {
                        if let Some(token) = tokens.last_mut() {
                            *token = Token::Equivalence;
                        }
                    }
                    _ => {
                        return Err(ParseProgramError::InvalidToken(
                            line.iter().collect(),
                            c.to_string(),
                        ));
                    }
                }
            } else {
                current_token.push(c);
                match current_token.as_str() {
                    "=>" => {
                        tokens.push(Token::Implication);
                        current_token.clear();
                    }
                    "<=" => {
                        tokens.push(Token::ConverseImplication);
                        current_token.clear();
                    }
                    _ => {
                        return Err(ParseProgramError::InvalidToken(
                            line.iter().collect(),
                            current_token,
                        ));
                    }
                }
            }
        }
        Ok(tokens)
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
