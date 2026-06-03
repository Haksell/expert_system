use crate::{ParseProgramError, Token};

// TODO: don't implement Clone
#[derive(Clone, Debug)]
pub enum Rule {
    Fact(char),
    Not(Box<Rule>),
    And(Box<Rule>, Box<Rule>),
    Or(Box<Rule>, Box<Rule>),
    Xor(Box<Rule>, Box<Rule>), // TODO: think about Not(Equivalence)
    Implication(Box<Rule>, Box<Rule>),
    Equivalence(Box<Rule>, Box<Rule>),
}

impl Rule {
    pub fn parse(line: &[char]) -> Result<Self, ParseProgramError> {
        let tokens = Self::tokenize(line)?;
        assert!(!tokens.is_empty()); // TODO: remove
        Self::check(&tokens)?;
        let tokens = Self::infix_to_rpn(tokens);
        println!("{tokens:?}");
        let mut rule = Self::build(tokens)?;
        while rule.apply_de_morgan() {}
        rule.remove_double_negation();
        Ok(rule)
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
                _ => {
                    cnt_implications += matches!(
                        token,
                        Token::Implication | Token::ConverseImplication | Token::Equivalence
                    ) as u32;
                }
            }
        }

        match cnt_implications {
            0 => return Err(ParseProgramError::MissingImplication),
            1 => {}
            _ => return Err(ParseProgramError::MultipleImplications),
        }

        if cnt_open == 0 {
            Ok(())
        } else {
            Err(ParseProgramError::UnbalancedParentheses)
        }
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

    fn build(tokens: Vec<Token>) -> Result<Self, ParseProgramError> {
        let mut rules = Vec::new();

        for token in tokens {
            match token {
                Token::Fact(c) => rules.push(Rule::Fact(c)),
                Token::Not => {
                    if let Some(rule) = rules.pop() {
                        rules.push(Rule::Not(Box::new(rule)));
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
                        rules.push(Rule::from_binary_token(token, rule1, rule2));
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

    fn from_binary_token(token: Token, rule1: Rule, rule2: Rule) -> Rule {
        let rule1 = Box::new(rule1);
        let rule2 = Box::new(rule2);

        match token {
            Token::Equivalence => Rule::Equivalence(rule1, rule2),
            Token::Implication => Rule::Implication(rule1, rule2),
            Token::ConverseImplication => Rule::Implication(rule2, rule1),
            Token::Xor => Rule::Xor(rule1, rule2),
            Token::Or => Rule::Or(rule1, rule2),
            Token::And => Rule::And(rule1, rule2),
            Token::Fact(_) | Token::Not | Token::LeftParenthesis | Token::RightParenthesis => {
                unreachable!()
            }
        }
    }

    // TODO: without .clone()
    // TODO: try in one pass
    fn apply_de_morgan(&mut self) -> bool {
        match self {
            Rule::Fact(_) => false,
            Rule::Not(child) => match *child.clone() {
                Rule::Fact(_) => false,
                Rule::Not(_) => child.apply_de_morgan(),
                Rule::Or(grandchild1, grandchild2) => {
                    let mut left = Rule::Not(grandchild1);
                    let mut right = Rule::Not(grandchild2);
                    left.apply_de_morgan();
                    right.apply_de_morgan();
                    *self = Rule::And(Box::new(left), Box::new(right));
                    true
                }
                Rule::And(grandchild1, grandchild2) => {
                    let mut left = Rule::Not(grandchild1);
                    let mut right = Rule::Not(grandchild2);
                    left.apply_de_morgan();
                    right.apply_de_morgan();
                    *self = Rule::Or(Box::new(left), Box::new(right));
                    true
                }
                _ => unreachable!(),
            },
            Rule::Or(child1, child2)
            | Rule::And(child1, child2)
            | Rule::Xor(child1, child2)
            | Rule::Implication(child1, child2)
            | Rule::Equivalence(child1, child2) => {
                // store in variables to avoid short-circuiting
                let b1 = child1.apply_de_morgan();
                let b2 = child2.apply_de_morgan();
                b1 || b2
            }
            _ => unreachable!(),
        }
    }

    // TODO: without .clone()
    fn remove_double_negation(&mut self) {
        match self {
            Rule::Fact(_) => {}
            Rule::Not(child) => match *child.clone() {
                Rule::Fact(_) => {}
                Rule::Not(grandchild) => {
                    *self = *grandchild.clone();
                    self.remove_double_negation();
                }
                _ => unreachable!(),
            },
            Rule::Or(child1, child2)
            | Rule::And(child1, child2)
            | Rule::Xor(child1, child2)
            | Rule::Implication(child1, child2)
            | Rule::Equivalence(child1, child2) => {
                child1.remove_double_negation();
                child2.remove_double_negation();
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
