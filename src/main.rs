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
}

impl From<std::io::Error> for ParseProgramError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Fact(char),
    Equivalence,
    ConverseImplication,
    Implication,
    Xor,
    Or,
    And,
    Not,
    LeftParenthesis,
    RightParenthesis,
}

impl Token {
    fn precedence(&self) -> u32 {
        match self {
            Token::Equivalence => 1,
            Token::ConverseImplication => 1,
            Token::Implication => 1,
            Token::Xor => 2,
            Token::Or => 3,
            Token::And => 4,
            Token::Not => 5,
            Token::Fact(_) | Token::LeftParenthesis | Token::RightParenthesis => {
                unreachable!()
            }
        }
    }
}

#[derive(Debug)]
enum Rule {
    Fact(char),
    Not(Box<Rule>),
    And(Box<Rule>, Box<Rule>),
    Or(Box<Rule>, Box<Rule>),
    Xor(Box<Rule>, Box<Rule>), // TODO: think about Not(Equivalence)
    Implication(Box<Rule>, Box<Rule>),
    Equivalence(Box<Rule>, Box<Rule>),
}

impl Rule {
    fn parse(line: &[char]) -> Result<Self, ParseProgramError> {
        let tokens = Self::tokenize(line)?;
        assert!(!tokens.is_empty());
        let tokens = Self::clean(tokens)?;
        let tokens = Self::infix_to_rpn(tokens);
        println!("{tokens:?}");
        // Self::build(&tokens)
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

    fn clean(tokens: Vec<Token>) -> Result<Vec<Token>, ParseProgramError> {
        // TODO: don't add left and right parenthesis
        let mut cleaned_tokens = vec![Token::LeftParenthesis];
        let mut cnt_open = 0;
        let mut cnt_implications = 0;

        for token in tokens {
            match token {
                Token::LeftParenthesis => cnt_open += 1,
                Token::RightParenthesis => {
                    if cnt_open == 0 {
                        return Err(ParseProgramError::UnbalancedParentheses);
                    }
                    cnt_open -= 1;
                }
                Token::Not if cleaned_tokens.last() == Some(&Token::Not) => {
                    cleaned_tokens.pop();
                }
                _ => {
                    cnt_implications += matches!(
                        token,
                        Token::Implication | Token::ConverseImplication | Token::Equivalence
                    ) as u32;
                    cleaned_tokens.push(token)
                }
            }
        }

        match cnt_implications {
            0 => return Err(ParseProgramError::MissingImplication),
            1 => {}
            _ => return Err(ParseProgramError::MultipleImplications),
        }

        if cnt_open == 0 {
            cleaned_tokens.push(Token::RightParenthesis);
            Ok(cleaned_tokens)
        } else {
            Err(ParseProgramError::UnbalancedParentheses)
        }
    }

    fn infix_to_rpn(tokens: Vec<Token>) -> Vec<Token> {
        let mut output = Vec::new();
        let mut operators = Vec::new();

        for token in tokens {
            match token {
                Token::Fact(_) => output.push(token),
                Token::Not | Token::LeftParenthesis => operators.push(token),
                Token::RightParenthesis => {
                    while operators.last() != Some(&Token::LeftParenthesis) {
                        output.push(operators.pop().unwrap());
                    }
                    operators.pop();
                    if let Some(Token::Not) = operators.last() {
                        output.push(operators.pop().unwrap());
                    }
                }
                Token::Equivalence
                | Token::ConverseImplication
                | Token::Implication
                | Token::Xor
                | Token::Or
                | Token::And => {
                    while operators.last().is_some_and(|top| {
                        top != &Token::LeftParenthesis && top.precedence() >= token.precedence()
                    }) {
                        output.push(operators.pop().unwrap());
                    }
                    // TODO: handle NOT???
                    operators.push(token);
                }
            }
        }

        while let Some(operator) = operators.pop() {
            assert_ne!(operator, Token::LeftParenthesis);
            output.push(operator);
        }

        output
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
