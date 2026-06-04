use crate::{Troolean, rule::Rule};
use itertools::Itertools as _;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead as _, BufReader},
};

pub fn start(reader: BufReader<File>) -> Result<(), ParseProgramError> {
    fn parse_variables(
        line: &[char],
        error_fn: impl Fn(char) -> ParseProgramError,
    ) -> Result<Vec<char>, ParseProgramError> {
        let mut variables = Vec::with_capacity(line.len());
        for &c in line {
            match c {
                'A'..='Z' => variables.push(c),
                _ => return Err(error_fn(c)),
            }
        }
        Ok(variables)
    }

    let mut global_rule = Rule::tautology();
    let mut facts = Vec::new();
    let mut got_facts = false;
    let mut got_queries = false;
    let mut last_is_query = false;
    let mut empty_file = true;

    for line in reader.lines() {
        let line = line?
            .chars()
            .filter(|c| !c.is_whitespace())
            .take_while(|&c| c != '#')
            .collect_vec();
        if line.is_empty() {
            continue;
        }

        empty_file = false;

        match line[0] {
            '=' => {
                facts = parse_variables(&line[1..], ParseProgramError::InvalidFact)?;
                got_facts = true;
                last_is_query = false;
            }
            '?' => {
                let queries = parse_variables(&line[1..], ParseProgramError::InvalidQuery)?;
                match solve(global_rule.clone(), &facts, &queries) {
                    Some(results) => {
                        for (fact, value) in results.iter().sorted() {
                            println!("{fact} is {value:?}");
                        }
                    }
                    None => println!("There is a contradiction in the rules."),
                }
                got_queries = true;
                last_is_query = true;
            }
            _ => {
                let mut new_rule = Rule::parse(&line)?;
                while new_rule.apply_de_morgan() {}
                new_rule.remove_xor_not_not();
                new_rule.remove_double_negation();
                global_rule = Rule::merge(global_rule, new_rule);
                last_is_query = false;
            }
        }
    }

    if empty_file {
        return Err(ParseProgramError::EmptyFile);
    }
    if !got_facts {
        return Err(ParseProgramError::MissingFacts);
    }
    if !got_queries {
        return Err(ParseProgramError::MissingQueries);
    }
    if !last_is_query {
        return Err(ParseProgramError::UnusedFactsOrRules);
    }

    Ok(())
}

pub fn solve(mut rule: Rule, facts: &[char], queries: &[char]) -> Option<HashMap<char, Troolean>> {
    rule.set_facts(facts);
    if !rule.is_satisfiable() {
        return None;
    }

    let mut results = HashMap::new();
    for &query in queries {
        let result = if facts.contains(&query) || !rule.is_satisfiable_with_fact(query, false) {
            Troolean::True
        } else if !rule.is_satisfiable_with_fact(query, true) {
            Troolean::False
        } else {
            Troolean::Ambiguous
        };
        results.insert(query, result);
    }

    Some(results)
}

// TODO: cleaner error messages through custom impl Debug
#[derive(Debug)]
pub enum ParseProgramError {
    IoError(#[expect(unused)] std::io::Error),
    InvalidToken(#[expect(unused)] String, #[expect(unused)] String), // line, token
    InvalidFact(#[expect(unused)] char),
    InvalidQuery(#[expect(unused)] char),
    MissingFacts,
    MissingQueries,
    UnbalancedParentheses,
    MissingImplication,
    MultipleImplications,
    ParenthesesAroundImplication,
    InvalidExpression,
    EmptyFile,
    UnusedFactsOrRules,
}

impl From<std::io::Error> for ParseProgramError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
