pub type LineInfo = Option<(usize, String)>;

fn display_line(line_info: &LineInfo) -> String {
    match line_info {
        Some((line_number, line)) => format!(" at line {line_number}:\n`{line}`"),
        None => String::new(),
    }
}

#[derive(Debug)]
pub enum ExpertSystemError {
    IoError(std::io::Error),
    ReadlineError(rustyline::error::ReadlineError),
    InvalidToken(LineInfo, String),
    InvalidFact(LineInfo, char),
    InvalidQuery(LineInfo, char),
    EmptyQuery(LineInfo),
    UnbalancedParentheses(LineInfo),
    MissingImplication(LineInfo),
    MultipleImplications(LineInfo),
    ParenthesesAroundImplication(LineInfo),
    InvalidExpression(LineInfo),
    Contradiction(LineInfo),
    MissingQueries,
    EmptyFile,
    UnusedFactsOrRules,
}

impl From<std::io::Error> for ExpertSystemError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl std::fmt::Display for ExpertSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "I/O error: {err}"),
            Self::ReadlineError(err) => write!(f, "I/O error: {err}"),
            Self::MissingQueries => write!(f, "no queries found in file"),
            Self::EmptyFile => write!(f, "file is empty"),
            Self::UnusedFactsOrRules => write!(f, "unused facts or rules"),
            Self::InvalidToken(line_info, token) => {
                write!(f, "invalid token: `{token}`{}", display_line(line_info))
            }
            Self::InvalidFact(line_info, fact) => {
                write!(f, "invalid fact: `{fact}`{}", display_line(line_info))
            }
            Self::InvalidQuery(line_info, query) => {
                write!(f, "invalid query: `{query}`{}", display_line(line_info))
            }
            Self::EmptyQuery(line_info) => {
                write!(f, "empty query{}", display_line(line_info))
            }
            Self::UnbalancedParentheses(line_info) => {
                write!(f, "unbalanced parentheses{}", display_line(line_info))
            }
            Self::MissingImplication(line_info) => {
                write!(f, "missing implication{}", display_line(line_info))
            }
            Self::MultipleImplications(line_info) => {
                write!(f, "multiple implications{}", display_line(line_info))
            }
            Self::ParenthesesAroundImplication(line_info) => {
                write!(
                    f,
                    "parentheses around implication{}",
                    display_line(line_info)
                )
            }
            Self::InvalidExpression(line_info) => {
                write!(f, "invalid expression{}", display_line(line_info))
            }
            Self::Contradiction(line_info) => {
                write!(f, "contradiction{}", display_line(line_info))
            }
        }
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
            | Self::InvalidFact(_, _)
            | Self::InvalidQuery(_, _)
            | Self::EmptyQuery(_)
            | Self::UnbalancedParentheses(_)
            | Self::MissingImplication(_)
            | Self::MultipleImplications(_)
            | Self::ParenthesesAroundImplication(_)
            | Self::InvalidExpression(_)
            | Self::Contradiction(_) => InteractiveHandling::Warning,
            Self::IoError(_) | Self::ReadlineError(_) => InteractiveHandling::Error,
        }
    }
}
