// TODO: cleaner error messages through custom impl Display
#[derive(Debug)]
pub enum ExpertSystemError {
    IoError(#[expect(unused)] std::io::Error),
    InvalidToken(#[expect(unused)] String, #[expect(unused)] String), // line, token
    InvalidFact(#[expect(unused)] char),
    InvalidQuery(#[expect(unused)] char),
    MissingQueries,
    UnbalancedParentheses,
    MissingImplication,
    MultipleImplications,
    ParenthesesAroundImplication,
    InvalidExpression,
    EmptyFile,
    UnusedFactsOrRules,
    Contradiction(#[expect(unused)] usize, #[expect(unused)] String),
    ReadlineError(#[expect(unused)] rustyline::error::ReadlineError),
}

impl From<std::io::Error> for ExpertSystemError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

pub enum InteractiveHandling {
    Error,
    Warning,
    Acceptable,
}

impl ExpertSystemError {
    pub const fn interactive_handling(&self) -> InteractiveHandling {
        match self {
            Self::EmptyFile | Self::MissingQueries | Self::UnusedFactsOrRules => {
                InteractiveHandling::Acceptable
            }
            Self::InvalidToken(_, _)
            | Self::InvalidFact(_)
            | Self::InvalidQuery(_)
            | Self::UnbalancedParentheses
            | Self::MissingImplication
            | Self::MultipleImplications
            | Self::ParenthesesAroundImplication
            | Self::InvalidExpression
            | Self::Contradiction(_, _) => InteractiveHandling::Warning,
            Self::IoError(_) | Self::ReadlineError(_) => InteractiveHandling::Error,
        }
    }
}
