use crate::{
    Troolean,
    error::{ExpertSystemError, InteractiveHandling, LineInfo, line_info},
    rule::Rule,
};
use itertools::Itertools as _;
use rustyline::error::ReadlineError;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead as _, BufReader},
    path::PathBuf,
};

#[expect(clippy::struct_excessive_bools)]
struct State {
    global_rule: Rule,
    facts: Vec<char>,
    got_facts: bool,
    got_queries: bool,
    last_is_query: bool,
    empty_file: bool,
}

#[expect(clippy::enum_variant_names)]
pub enum ExecMode {
    OnlyFile(PathBuf),
    InteractiveWithFile(PathBuf),
    InteractiveWithoutFile,
}

pub fn exec(mode: &ExecMode) -> Result<(), ExpertSystemError> {
    let mut state = State::new();

    match &mode {
        ExecMode::OnlyFile(filename) | ExecMode::InteractiveWithFile(filename) => {
            let file = File::open(filename)?;
            let reader = BufReader::new(file);
            for (line_number, line) in reader.lines().enumerate() {
                state.update(Some(line_number), &line?)?;
            }
        }
        ExecMode::InteractiveWithoutFile => {}
    }

    match &mode {
        ExecMode::InteractiveWithFile(_) | ExecMode::InteractiveWithoutFile => {
            let mut rl = match rustyline::DefaultEditor::new() {
                Ok(rl) => rl,
                Err(readline_error) => {
                    return Err(ExpertSystemError::ReadlineError(readline_error));
                }
            };
            loop {
                let input = rl.readline(">> ");
                match input {
                    Ok(line) => {
                        if let Err(err) = state.update(None, &line) {
                            match err.interactive_handling() {
                                InteractiveHandling::Error => return Err(err),
                                InteractiveHandling::Warning => println!("Warning: {err}"),
                                InteractiveHandling::Acceptable => {}
                            }
                        }
                    }
                    Err(readline_error @ (ReadlineError::Io(_) | ReadlineError::Errno(_))) => {
                        return Err(ExpertSystemError::ReadlineError(readline_error));
                    }
                    Err(ReadlineError::Eof | ReadlineError::Interrupted) => {
                        println!("Bye.");
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
        ExecMode::OnlyFile(_) => {}
    }

    if state.empty_file {
        return Err(ExpertSystemError::EmptyFile);
    }
    if !state.got_queries {
        return Err(ExpertSystemError::MissingQueries);
    }
    if !state.last_is_query {
        return Err(ExpertSystemError::UnusedFactsOrRules);
    }

    Ok(())
}

impl State {
    const fn new() -> Self {
        Self {
            global_rule: Rule::tautology(),
            facts: Vec::new(),
            got_facts: false,
            got_queries: false,
            last_is_query: false,
            empty_file: true,
        }
    }

    fn update(&mut self, line_number: Option<usize>, line: &str) -> Result<(), ExpertSystemError> {
        fn parse_variables(
            chars: &[char],
            error_fn: impl Fn(LineInfo, char) -> ExpertSystemError,
            line_number: Option<usize>,
            line: &str,
        ) -> Result<Vec<char>, ExpertSystemError> {
            let mut variables = Vec::with_capacity(chars.len());
            for &c in chars {
                match c {
                    'A'..='Z' => variables.push(c),
                    _ => return Err(error_fn(line_info(line_number, line), c)),
                }
            }
            Ok(variables)
        }

        let chars = line
            .chars()
            .filter(|c| !c.is_whitespace())
            .take_while(|&c| c != '#')
            .collect_vec();
        if chars.is_empty() {
            return Ok(());
        }

        self.empty_file = false;

        match chars[0] {
            '=' => {
                self.facts = parse_variables(
                    &chars[1..],
                    ExpertSystemError::InvalidFact,
                    line_number,
                    line,
                )?;
                let mut rule_with_facts = self.global_rule.clone();
                rule_with_facts.set_facts(&self.facts);
                if !rule_with_facts.is_satisfiable() {
                    return Err(ExpertSystemError::Contradiction(line_info(
                        line_number,
                        line,
                    )));
                }
                self.got_facts = true;
                self.last_is_query = false;
            }
            '?' => {
                let queries = parse_variables(
                    &chars[1..],
                    ExpertSystemError::InvalidQuery,
                    line_number,
                    line,
                )?;
                let results = query(self.global_rule.clone(), &self.facts, &queries);
                println!("={}:", self.facts.iter().collect::<String>());
                for (fact, value) in results.iter().sorted() {
                    println!("{fact} is {value:?}");
                }
                self.got_queries = true;
                self.last_is_query = true;
            }
            _ => {
                let mut new_rule = Rule::parse(&chars, line_number, line)?;
                while new_rule.apply_de_morgan() {}
                new_rule.remove_xor_not_not();
                new_rule.remove_double_negation();
                self.global_rule.merge(new_rule);
                if !self.global_rule.is_satisfiable() {
                    return Err(ExpertSystemError::Contradiction(line_info(
                        line_number,
                        line,
                    )));
                }
                self.last_is_query = false;
            }
        }

        Ok(())
    }
}

pub fn query(mut rule: Rule, facts: &[char], queries: &[char]) -> HashMap<char, Troolean> {
    rule.set_facts(facts);

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

    results
}
