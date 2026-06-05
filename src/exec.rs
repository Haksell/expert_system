use crate::{
    InferenceEngine, Troolean,
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

#[expect(clippy::enum_variant_names)]
pub enum ExecMode {
    OnlyFile(PathBuf),
    InteractiveWithFile(PathBuf),
    InteractiveWithoutFile,
}

pub fn exec(mode: &ExecMode, engine: InferenceEngine) -> Result<(), ExpertSystemError> {
    let mut state = State::new(engine);

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
            let mut rl = rustyline::DefaultEditor::new()?;
            loop {
                let input = rl.readline(">> ");
                match input {
                    Ok(line) => {
                        rl.add_history_entry(line.as_str())?;
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

#[expect(clippy::struct_excessive_bools)]
struct State {
    engine: InferenceEngine,
    rule: Rule,
    given_facts: HashSet<char>,
    implied_facts: HashSet<char>,
    got_facts: bool,
    got_queries: bool,
    last_is_query: bool,
    empty_file: bool,
}

impl State {
    fn new(engine: InferenceEngine) -> Self {
        Self {
            engine,
            rule: Rule::tautology(),
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
                let mut rule_with_facts = self.rule.clone();
                rule_with_facts.set_facts(&self.given_facts, &self.antifacts());
                if !rule_with_facts.is_satisfiable() {
                    return Err(ExpertSystemError::Contradiction(line_info));
                }
                self.got_facts = true;
                self.last_is_query = false;
            }
            '?' => {
                let queries =
                    parse_variables(&chars[1..], ExpertSystemError::InvalidQuery, &line_info)?;
                if queries.is_empty() {
                    return Err(ExpertSystemError::EmptyQuery(line_info));
                }
                let results = self.query(&queries);
                print_query_results(&self.given_facts, &results);
                self.got_queries = true;
                self.last_is_query = true;
            }
            _ => {
                let (mut new_rule, new_implied_facts) =
                    Rule::parse(&chars, &line_info, self.engine)?;
                self.implied_facts.extend(&new_implied_facts);
                while new_rule.apply_de_morgan() {}
                new_rule.remove_xor_not_not();
                new_rule.remove_double_negation();
                self.rule.merge(new_rule);
                if !self.rule.is_satisfiable() {
                    return Err(ExpertSystemError::Contradiction(line_info));
                }
                self.last_is_query = false;
            }
        }

        Ok(())
    }

    pub fn query(&self, queries: &HashSet<char>) -> HashMap<char, Troolean> {
        let mut rule = self.rule.clone();
        rule.set_facts(&self.given_facts, &self.antifacts());
        debug_assert!(rule.is_satisfiable());

        let mut results = HashMap::new();
        for &query in queries {
            let can_be_false =
                !self.given_facts.contains(&query) && rule.is_satisfiable_with_fact(query, false);
            // TODO: can_be_true = self.given_facts.contains(&query) || ...
            let can_be_true = rule.is_satisfiable_with_fact(query, true);
            let result = match self.engine {
                InferenceEngine::SatSolver => match (can_be_false, can_be_true) {
                    (true, true) => Troolean::Ambiguous,
                    (true, false) => Troolean::False,
                    (false, true) => Troolean::True,
                    (false, false) => unreachable!("contradiction checked when setting facts"),
                },
                InferenceEngine::BackwardChaining => {
                    if can_be_false {
                        Troolean::False
                    } else {
                        Troolean::True
                    }
                }
            };
            results.insert(query, result);
        }

        results
    }

    fn antifacts(&self) -> HashSet<char> {
        match self.engine {
            InferenceEngine::SatSolver => HashSet::new(),
            InferenceEngine::BackwardChaining => self
                .rule
                .get_variables()
                .iter()
                .filter(|f| !self.given_facts.contains(f) && !self.implied_facts.contains(f))
                .copied()
                .collect(),
        }
    }
}

fn print_query_results(given_facts: &HashSet<char>, results: &HashMap<char, Troolean>) {
    fn print_facts_with_value(results: &HashMap<char, Troolean>, value_to_print: Troolean) {
        let facts_to_print = results
            .iter()
            .filter_map(|(k, v)| (*v == value_to_print).then_some(k))
            .sorted()
            .collect::<String>();
        match facts_to_print.len() {
            0 => {}
            1 => println!("-> Fact {facts_to_print} is {value_to_print}."),
            _ => println!("-> Facts {facts_to_print} are {value_to_print}."),
        }
    }

    let given_facts = given_facts.iter().sorted().collect::<String>();
    match given_facts.len() {
        0 => println!("Given no fact:"),
        1 => println!("Given fact {given_facts}:"),
        _ => println!("Given facts {given_facts}:"),
    }

    print_facts_with_value(results, Troolean::True);
    print_facts_with_value(results, Troolean::False);
    print_facts_with_value(results, Troolean::Ambiguous);
}
