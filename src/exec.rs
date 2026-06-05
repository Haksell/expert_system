use crate::{
    Quadrulean,
    error::{ExpertSystemError, InteractiveHandling, LineInfo},
    rule::Rule,
};
use itertools::Itertools as _;
use rustyline::error::ReadlineError;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead as _, BufReader},
    path::PathBuf,
};

#[expect(clippy::struct_excessive_bools)]
struct State {
    global_rule: Rule,
    given_facts: HashSet<char>,
    // TODO: better name
    implied_facts: HashSet<char>,
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
                state.update(&line?, Some(line_number + 1))?;
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
                        if let Err(err) = state.update(&line, None) {
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
    fn new() -> Self {
        Self {
            global_rule: Rule::tautology(),
            given_facts: HashSet::new(),
            implied_facts: HashSet::new(),
            got_facts: false,
            got_queries: false,
            last_is_query: false,
            empty_file: true,
        }
    }

    fn update(&mut self, line: &str, line_number: Option<usize>) -> Result<(), ExpertSystemError> {
        fn parse_variables(
            chars: &[char],
            error_fn: impl Fn(LineInfo, char) -> ExpertSystemError,
            line_info: &LineInfo,
        ) -> Result<HashSet<char>, ExpertSystemError> {
            let mut variables = HashSet::with_capacity(chars.len());
            for &c in chars {
                match c {
                    'A'..='Z' => {
                        variables.insert(c);
                    }
                    _ => return Err(error_fn(line_info.clone(), c)),
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

        let line_info = line_number.map(|ln| (ln, line.to_owned()));

        match chars[0] {
            '=' => {
                self.given_facts =
                    parse_variables(&chars[1..], ExpertSystemError::InvalidFact, &line_info)?;
                let mut rule_with_facts = self.global_rule.clone();
                rule_with_facts.set_facts(&self.given_facts, &self.implied_facts);
                if !rule_with_facts.is_satisfiable() {
                    return Err(ExpertSystemError::Contradiction(line_info));
                }
                self.got_facts = true;
                self.last_is_query = false;
            }
            '?' => {
                let queries =
                    parse_variables(&chars[1..], ExpertSystemError::InvalidQuery, &line_info)?;
                let results = query(
                    self.global_rule.clone(),
                    &self.given_facts,
                    &self.implied_facts,
                    &queries,
                );
                // TODO: better print
                println!("={}:", self.given_facts.iter().collect::<String>());
                for (fact, value) in results.iter().sorted() {
                    println!("{fact} is {value:?}");
                }
                self.got_queries = true;
                self.last_is_query = true;
            }
            _ => {
                let (mut new_rule, new_implied_facts) = Rule::parse(&chars, &line_info)?;
                self.implied_facts.extend(&new_implied_facts);
                while new_rule.apply_de_morgan() {}
                new_rule.remove_xor_not_not();
                new_rule.remove_double_negation();
                self.global_rule.merge(new_rule);
                if !self.global_rule.is_satisfiable() {
                    return Err(ExpertSystemError::Contradiction(line_info));
                }
                self.last_is_query = false;
            }
        }

        Ok(())
    }
}

// TODO: inside impl State
pub fn query(
    mut rule: Rule,
    given_facts: &HashSet<char>,
    implied_facts: &HashSet<char>,
    queries: &HashSet<char>,
) -> HashMap<char, Quadrulean> {
    // println!("{given_facts:?} {implied_facts:?}");
    rule.set_facts(given_facts, implied_facts);
    // println!("{rule:#?}");

    let mut results = HashMap::new();
    for &query in queries {
        let can_be_false =
            !given_facts.contains(&query) && rule.is_satisfiable_with_fact(query, false);
        // let can_be_true = rule.is_satisfiable_with_fact(query, true);
        let result = if can_be_false {
            Quadrulean::False
        } else {
            Quadrulean::True
        };
        results.insert(query, result);
    }

    results
}
