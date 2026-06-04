use crate::ParseProgramError;
use itertools::Itertools as _;

// TODO: don't implement Clone
#[derive(Clone, Debug)]
pub enum Rule {
    Bool(bool),
    Fact(char),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Xor(Box<Self>, Box<Self>),
}

impl Rule {
    pub fn parse(line: &[char]) -> Result<Self, ParseProgramError> {
        let tokens = Self::tokenize(line)?;
        Self::check(&tokens)?;
        let tokens = Self::infix_to_rpn(tokens);
        println!("{tokens:?}");
        Self::build(tokens)
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

        if !current_token.is_empty() {
            return Err(ParseProgramError::InvalidToken(
                line.iter().collect(),
                current_token,
            ));
        }

        Ok(tokens)
    }

    fn check(tokens: &[Token]) -> Result<(), ParseProgramError> {
        let mut cnt_open = 0;
        let mut found_implication = false;

        for token in tokens {
            match token {
                Token::LeftParenthesis => cnt_open += 1,
                Token::RightParenthesis => {
                    if cnt_open == 0 {
                        return Err(ParseProgramError::UnbalancedParentheses);
                    }
                    cnt_open -= 1;
                }
                Token::Implication | Token::ConverseImplication | Token::Equivalence => {
                    if found_implication {
                        return Err(ParseProgramError::MultipleImplications);
                    }
                    if cnt_open != 0 {
                        return Err(ParseProgramError::ParenthesesAroundImplication);
                    }
                    found_implication = true;
                }
                Token::Fact(_) | Token::Xor | Token::Or | Token::And | Token::Not => {}
            }
        }

        if !found_implication {
            return Err(ParseProgramError::MissingImplication);
        }
        if cnt_open != 0 {
            return Err(ParseProgramError::UnbalancedParentheses);
        }

        Ok(())
    }

    // TODO: handle broken input (A&B|)
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
                    // TODO: remove?
                    if operators.last() == Some(&Token::Not) {
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

    fn build(tokens: Vec<Token>) -> Result<Self, ParseProgramError> {
        let mut rules = Vec::new();

        for token in tokens {
            match token {
                Token::Fact(c) => rules.push(Self::Fact(c)),
                Token::Not => {
                    if let Some(rule) = rules.pop() {
                        rules.push(Self::Not(Box::new(rule)));
                    } else {
                        return Err(ParseProgramError::BuildFailed);
                    }
                }
                Token::Equivalence
                | Token::ConverseImplication
                | Token::Implication
                | Token::Xor
                | Token::Or
                | Token::And => {
                    if let (Some(rule2), Some(rule1)) = (rules.pop(), rules.pop()) {
                        rules.push(Self::from_binary_token(token, rule1, rule2));
                    } else {
                        return Err(ParseProgramError::BuildFailed);
                    }
                }
                Token::LeftParenthesis | Token::RightParenthesis => unreachable!(),
            }
        }

        if rules.len() != 1 {
            return Err(ParseProgramError::BuildFailed);
        }

        Ok(rules.pop().unwrap())
    }

    fn from_binary_token(token: Token, rule1: Self, rule2: Self) -> Self {
        let rule1 = Box::new(rule1);
        let rule2 = Box::new(rule2);

        match token {
            Token::Equivalence => Self::Not(Box::new(Self::Xor(rule1, rule2))),
            Token::Implication => Self::Or(Box::new(Self::Not(rule1)), rule2),
            Token::ConverseImplication => Self::Or(rule1, Box::new(Self::Not(rule2))),
            Token::Xor => Self::Xor(rule1, rule2),
            Token::Or => Self::Or(rule1, rule2),
            Token::And => Self::And(rule1, rule2),
            Token::Fact(_) | Token::Not | Token::LeftParenthesis | Token::RightParenthesis => {
                unreachable!()
            }
        }
    }

    // TODO: try in one pass
    fn apply_de_morgan(&mut self) -> bool {
        match self {
            Self::Bool(_) | Self::Fact(_) => false,
            // TODO: remove .clone()
            Self::Not(child) => match *child.clone() {
                Self::Bool(_) | Self::Fact(_) => false,
                Self::Not(_) => child.apply_de_morgan(),
                Self::Or(grandchild1, grandchild2) => {
                    let mut left = Self::Not(grandchild1);
                    let mut right = Self::Not(grandchild2);
                    left.apply_de_morgan();
                    right.apply_de_morgan();
                    *self = Self::And(Box::new(left), Box::new(right));
                    true
                }
                Self::And(grandchild1, grandchild2) => {
                    let mut left = Self::Not(grandchild1);
                    let mut right = Self::Not(grandchild2);
                    left.apply_de_morgan();
                    right.apply_de_morgan();
                    *self = Self::Or(Box::new(left), Box::new(right));
                    true
                }
                Self::Xor(grandchild1, grandchild2) => {
                    let mut left = Self::Not(grandchild1);
                    let mut right = grandchild2;
                    left.apply_de_morgan();
                    right.apply_de_morgan();
                    *self = Self::Xor(Box::new(left), right);
                    true
                }
            },
            Self::Or(child1, child2) | Self::And(child1, child2) | Self::Xor(child1, child2) => {
                // store in variables to avoid short-circuiting
                let b1 = child1.apply_de_morgan();
                let b2 = child2.apply_de_morgan();
                b1 || b2
            }
        }
    }

    fn remove_xor_not_not(&mut self) {
        match self {
            Self::Bool(_) | Self::Fact(_) => {}
            Self::Not(child) => {
                child.remove_xor_not_not();
            }
            Self::Xor(child1, child2) => {
                child1.remove_xor_not_not();
                child2.remove_xor_not_not();
                if let (Self::Not(grandchild1), Self::Not(grandchild2)) =
                    (child1.as_ref(), child2.as_ref())
                {
                    *self = Self::Xor(grandchild1.clone(), grandchild2.clone());
                }
            }
            Self::Or(child1, child2) | Self::And(child1, child2) => {
                child1.remove_xor_not_not();
                child2.remove_xor_not_not();
            }
        }
    }

    fn remove_double_negation(&mut self) {
        match self {
            Self::Bool(_) | Self::Fact(_) => {}
            // TODO: remove .clone()
            Self::Not(child) => match *child.clone() {
                Self::Bool(_) | Self::Fact(_) => {}
                Self::Not(grandchild) => {
                    *self = *grandchild;
                    self.remove_double_negation();
                }
                Self::And(..) | Self::Or(..) | Self::Xor(..) => unreachable!(),
            },
            Self::Or(child1, child2) | Self::And(child1, child2) | Self::Xor(child1, child2) => {
                child1.remove_double_negation();
                child2.remove_double_negation();
            }
        }
    }

    pub fn merge(rules: Vec<Self>) -> Self {
        fn helper(rules: &mut [Option<Rule>], lo: usize, hi: usize) -> Rule {
            if lo == hi - 1 {
                rules[lo].take().unwrap()
            } else {
                let mi = usize::midpoint(lo, hi);
                Rule::And(
                    Box::new(helper(rules, lo, mi)),
                    Box::new(helper(rules, mi, hi)),
                )
            }
        }

        // TODO: without option
        let mut rules = rules.into_iter().map(Some).collect_vec();
        let rules_count = rules.len();
        let mut rule = helper(&mut rules, 0, rules_count);
        while rule.apply_de_morgan() {}
        rule.remove_xor_not_not();
        rule.remove_double_negation();
        rule
    }

    pub fn set_facts(&mut self, facts: &[char]) {
        for f in facts {
            self.set_fact(*f);
        }
    }

    // TODO: remove clones
    fn set_fact(&mut self, c: char) {
        match self {
            Self::Fact(f) if *f == c => *self = Self::Bool(true),
            Self::Bool(_) | Self::Fact(_) => {}
            Self::Not(rule) => {
                rule.set_fact(c);
                if let Self::Bool(b) = rule.as_ref() {
                    *self = Self::Bool(!b);
                }
            }
            Self::And(rule1, rule2) => {
                rule1.set_fact(c);
                rule2.set_fact(c);
                match (rule1.as_ref(), rule2.as_ref()) {
                    (Self::Bool(b1), Self::Bool(b2)) => *self = Self::Bool(*b1 && *b2),
                    (Self::Bool(true), child) | (child, Self::Bool(true)) => *self = child.clone(),
                    (Self::Bool(false), _) | (_, Self::Bool(false)) => *self = Self::Bool(false),
                    _ => {}
                }
            }
            Self::Or(rule1, rule2) => {
                rule1.set_fact(c);
                rule2.set_fact(c);
                match (rule1.as_ref(), rule2.as_ref()) {
                    (Self::Bool(b1), Self::Bool(b2)) => *self = Self::Bool(*b1 || *b2),
                    (Self::Bool(false), child) | (child, Self::Bool(false)) => {
                        *self = child.clone();
                    }
                    (Self::Bool(true), _) | (_, Self::Bool(true)) => *self = Self::Bool(true),
                    _ => {}
                }
            }
            Self::Xor(rule1, rule2) => {
                rule1.set_fact(c);
                rule2.set_fact(c);
                match (rule1.as_ref(), rule2.as_ref()) {
                    (Self::Bool(b1), Self::Bool(b2)) => *self = Self::Bool(*b1 ^ *b2),
                    (Self::Bool(true), Self::Not(child)) | (Self::Not(child), Self::Bool(true)) => {
                        *self = *child.clone();
                    }
                    (Self::Bool(false), Self::Not(child))
                    | (Self::Not(child), Self::Bool(false)) => {
                        *self = Self::Not(child.clone());
                    }
                    (Self::Bool(true), child) | (child, Self::Bool(true)) => {
                        *self = Self::Not(Box::new(child.clone()));
                    }
                    (Self::Bool(false), child) | (child, Self::Bool(false)) => {
                        *self = child.clone();
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Token {
    Fact(char),
    Equivalence,
    Implication,
    ConverseImplication,
    Xor,
    Or,
    And,
    Not,
    LeftParenthesis,
    RightParenthesis,
}

impl Token {
    fn precedence(self) -> u32 {
        match self {
            Self::Equivalence | Self::ConverseImplication | Self::Implication => 1,
            Self::Xor => 2,
            Self::Or => 3,
            Self::And => 4,
            Self::Not => 5,
            Self::Fact(_) | Self::LeftParenthesis | Self::RightParenthesis => {
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn infix_to_rpn() {
        assert_eq!(
            Rule::infix_to_rpn(vec![Fact('C'), Implication, Fact('E')]),
            vec![Fact('C'), Fact('E'), Implication]
        );
        assert_eq!(
            Rule::infix_to_rpn(vec![
                Fact('A'),
                And,
                Fact('B'),
                And,
                Fact('C'),
                Implication,
                Fact('D')
            ]),
            vec![
                Fact('A'),
                Fact('B'),
                And,
                Fact('C'),
                And,
                Fact('D'),
                Implication
            ]
        );
        assert_eq!(
            Rule::infix_to_rpn(vec![Fact('A'), And, Not, Fact('B'), Implication, Fact('F')]),
            vec![Fact('A'), Fact('B'), Not, And, Fact('F'), Implication]
        );
        assert_eq!(
            Rule::infix_to_rpn(vec![Fact('A'), And, Fact('B'), Or, Fact('C')]),
            vec![Fact('A'), Fact('B'), And, Fact('C'), Or]
        );
        assert_eq!(
            Rule::infix_to_rpn(vec![Fact('A'), Or, Fact('B'), And, Fact('C')]),
            vec![Fact('A'), Fact('B'), Fact('C'), And, Or]
        );
        assert_eq!(
            Rule::infix_to_rpn(vec![
                LeftParenthesis,
                Fact('A'),
                Or,
                Fact('B'),
                RightParenthesis,
                And,
                Fact('C')
            ]),
            vec![Fact('A'), Fact('B'), Or, Fact('C'), And]
        );
        assert_eq!(
            Rule::infix_to_rpn(vec![
                Fact('A'),
                Or,
                LeftParenthesis,
                Fact('B'),
                And,
                Fact('C'),
                RightParenthesis
            ]),
            vec![Fact('A'), Fact('B'), Fact('C'), And, Or]
        );
        assert_eq!(
            Rule::infix_to_rpn(vec![
                Fact('A'),
                And,
                Not,
                LeftParenthesis,
                Fact('B'),
                Or,
                Fact('C'),
                RightParenthesis
            ]),
            vec![Fact('A'), Fact('B'), Fact('C'), Or, Not, And]
        );
    }
}
