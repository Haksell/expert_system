// TODO: cleaner error messages through custom impl Display
#[derive(Debug)]
pub enum ExpertSystemError {
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
    Contradiction(#[expect(unused)] usize, #[expect(unused)] String),
}

impl From<std::io::Error> for ExpertSystemError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
