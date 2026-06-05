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

type QueryResult = HashMap<char, Troolean>;

#[expect(clippy::enum_variant_names)]
pub enum ExecMode {
    OnlyFile(PathBuf),
    InteractiveWithFile(PathBuf),
    InteractiveWithoutFile,
}

pub fn exec(
    mode: &ExecMode,
    engine: InferenceEngine,
) -> Result<Vec<QueryResult>, ExpertSystemError> {
    let mut state = State::new(engine);
    let mut query_results = Vec::new();

    match &mode {
        ExecMode::OnlyFile(filename) | ExecMode::InteractiveWithFile(filename) => {
            let file = File::open(filename)?;
            let reader = BufReader::new(file);
            for (line_number, line) in reader.lines().enumerate() {
                query_results.extend(state.update(&line?, Some(line_number + 1))?);
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
                        match state.update(&line, None) {
                            Ok(query_result) => query_results.extend(query_result),
                            Err(err) => match err.interactive_handling() {
                                InteractiveHandling::Error => return Err(err),
                                InteractiveHandling::Warning => println!("Warning: {err}"),
                                InteractiveHandling::Acceptable => {}
                            },
                        }
                    }
                    Err(readline_error @ (ReadlineError::Io(_) | ReadlineError::Errno(_))) => {
                        return Err(ExpertSystemError::ReadlineError(readline_error));
                    }
                    Err(ReadlineError::Eof | ReadlineError::Interrupted) => {
                        println!("Bye.");
                        return Ok(query_results);
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

    Ok(query_results)
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

    fn update(
        &mut self,
        line: &str,
        line_number: Option<usize>,
    ) -> Result<Option<QueryResult>, ExpertSystemError> {
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
            return Ok(None);
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
                let query_result = self.query(&queries);
                print_query_result(&self.given_facts, &query_result);
                self.got_queries = true;
                self.last_is_query = true;
                return Ok(Some(query_result));
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

        Ok(None)
    }

    pub fn query(&self, queries: &HashSet<char>) -> QueryResult {
        let mut rule = self.rule.clone();
        rule.set_facts(&self.given_facts, &self.antifacts());
        debug_assert!(rule.is_satisfiable());

        let mut results = HashMap::new();
        for &query in queries {
            let can_be_false =
                !self.given_facts.contains(&query) && rule.is_satisfiable_with_fact(query, false);
            let result = match self.engine {
                InferenceEngine::SatSolver => {
                    let can_be_true = rule.is_satisfiable_with_fact(query, true);
                    match (can_be_false, can_be_true) {
                        (true, true) => Troolean::Ambiguous,
                        (true, false) => Troolean::False,
                        (false, true) => Troolean::True,
                        (false, false) => unreachable!("contradiction checked when setting facts"),
                    }
                }
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

fn print_query_result(given_facts: &HashSet<char>, query_result: &QueryResult) {
    fn print_facts_with_value(query_result: &QueryResult, value_to_print: Troolean) {
        let facts_to_print = query_result
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

    print_facts_with_value(query_result, Troolean::True);
    print_facts_with_value(query_result, Troolean::False);
    print_facts_with_value(query_result, Troolean::Ambiguous);
}

#[cfg(test)]
mod tests {
    use super::*;
    use Troolean::*;

    // TODO: parsing tests

    #[test]
    fn backward_chaining_only_and() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/only_and.txt")),
            InferenceEngine::BackwardChaining,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('A', True), ('F', True), ('K', True), ('P', True)]),
                HashMap::from([('A', True), ('F', True), ('K', False), ('P', True)])
            ]
        );
    }

    #[test]
    fn backward_chaining_and_or() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/and_or.txt")),
            InferenceEngine::BackwardChaining,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('A', False)]),
                HashMap::from([('A', True)]),
                HashMap::from([('A', True)]),
                HashMap::from([('A', True)]),
            ]
        );
    }

    #[test]
    fn backward_chaining_and_xor() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/and_xor.txt")),
            InferenceEngine::BackwardChaining,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('A', False)]),
                HashMap::from([('A', True)]),
                HashMap::from([('A', True)]),
                HashMap::from([('A', False)]),
            ]
        );
    }

    #[test]
    fn backward_chaining_basic_negation() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/basic_negation.txt")),
            InferenceEngine::BackwardChaining,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('A', False)]),
                HashMap::from([('A', True)]),
                HashMap::from([('A', False)]),
                HashMap::from([('A', False)]),
            ]
        );
    }

    #[test]
    fn backward_chaining_same_conclusion_in_multiple_rules() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from(
                "./files/same_conclusion_in_multiple_rules.txt",
            )),
            InferenceEngine::BackwardChaining,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('A', False)]),
                HashMap::from([('A', True)]),
                HashMap::from([('A', True)]),
                HashMap::from([('A', True)]),
            ]
        );
    }

    #[test]
    fn backward_chaining_parenthesis() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/parenthesis.txt")),
            InferenceEngine::BackwardChaining,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('E', False)]),
                HashMap::from([('E', True)]),
                HashMap::from([('E', False)]),
                HashMap::from([('E', False)]),
                HashMap::from([('E', True)]),
                HashMap::from([('E', True)]),
                HashMap::from([('E', False)]),
                HashMap::from([('E', False)]),
                HashMap::from([('E', False)]),
                HashMap::from([('E', True)]),
                HashMap::from([('E', True)]),
            ]
        );
    }

    #[test]
    fn sat_solver_only_and() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/only_and.txt")),
            InferenceEngine::SatSolver,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('A', True), ('F', True), ('K', True), ('P', True)]),
                HashMap::from([('A', True), ('F', True), ('K', Ambiguous), ('P', True)])
            ]
        );
    }

    #[test]
    fn sat_solver_or_in_conclusion() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/or_in_conclusion.txt")),
            InferenceEngine::SatSolver,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('B', Ambiguous), ('C', Ambiguous)]),
                HashMap::from([('B', False), ('C', True)]),
            ]
        );
    }

    #[test]
    fn sat_solver_xor_in_conclusion() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/xor_in_conclusion.txt")),
            InferenceEngine::SatSolver,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('B', Ambiguous), ('C', Ambiguous)]),
                HashMap::from([('B', True), ('C', False)]),
            ]
        );
    }

    #[test]
    fn sat_solver_converse_implication() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/converse_implication.txt")),
            InferenceEngine::SatSolver,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[HashMap::from([('A', True), ('B', True), ('C', True)])]
        );
    }

    #[test]
    fn sat_solver_basic_equivalence() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/basic_equivalence.txt")),
            InferenceEngine::SatSolver,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('A', Ambiguous), ('B', Ambiguous)]),
                HashMap::from([('A', True), ('B', True)]),
            ]
        );
    }

    #[test]
    fn sat_solver_tautology() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/tautology.txt")),
            InferenceEngine::SatSolver,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('A', Ambiguous), ('B', Ambiguous)]),
                HashMap::from([('A', True), ('B', Ambiguous)]),
                HashMap::from([('A', Ambiguous), ('B', True)]),
                HashMap::from([('A', True), ('B', True)]),
            ]
        );
    }

    #[test]
    fn sat_solver_double_not_parentheses() {
        let exec_result = exec(
            &ExecMode::OnlyFile(PathBuf::from("./files/double_not_parentheses.txt")),
            InferenceEngine::SatSolver,
        );
        assert!(exec_result.is_ok());
        let query_results = exec_result.unwrap();
        assert_eq!(
            &query_results,
            &[
                HashMap::from([('C', Ambiguous)]),
                HashMap::from([('C', True)])
            ]
        );
    }
}
