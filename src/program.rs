use crate::rule::Rule;
use itertools::Itertools as _;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead as _, BufReader},
};

// TODO: cleaner error messages through custom impl Debug
#[derive(Debug)]
pub enum ParseProgramError {
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

#[derive(Debug)]
pub struct Program {
    rule: Rule,
    facts: Vec<char>,
    queries: Vec<char>,
}

impl Program {
    pub fn parse(reader: BufReader<File>) -> Result<Self, ParseProgramError> {
        fn parse_variables(
            variables_to_fill: &mut Option<Vec<char>>,
            line: &[char],
            invalid_fn: impl Fn(char) -> ParseProgramError,
            duplicate_fn: ParseProgramError,
        ) -> Result<(), ParseProgramError> {
            if variables_to_fill.is_some() {
                return Err(duplicate_fn);
            }
            let mut variables = Vec::with_capacity(line.len());
            for &c in line {
                match c {
                    'A'..='Z' => variables.push(c),
                    _ => return Err(invalid_fn(c)),
                }
            }
            variables_to_fill.replace(variables);
            Ok(())
        }

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
                '=' => parse_variables(
                    &mut facts,
                    &line[1..],
                    ParseProgramError::InvalidFact,
                    ParseProgramError::DuplicateFacts,
                )?,
                '?' => {
                    if facts.is_none() {
                        return Err(ParseProgramError::QueriesBeforeFacts);
                    }
                    parse_variables(
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

    pub fn solve(&mut self) -> HashMap<char, bool> {
        self.rule.set_facts(&self.facts);
        let mut rez = HashMap::new();
        println!("{:?}", self.rule);

        rez
    }
}
