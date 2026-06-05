use crate::{ExpertSystemError, error::LineInfo};
use itertools::Itertools as _;
use std::collections::{HashMap, HashSet};

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
    pub fn parse(
        chars: &[char],
        line_info: &LineInfo,
    ) -> Result<(Self, HashSet<char>), ExpertSystemError> {
        let tokens = Self::tokenize(chars, line_info)?;
        let implied_facts = Self::check(&tokens, line_info)?;
        let tokens = Self::infix_to_rpn(tokens);
        let rule = Self::build(tokens, line_info)?;
        Ok((rule, implied_facts))
    }

    fn tokenize(chars: &[char], line_info: &LineInfo) -> Result<Vec<Token>, ExpertSystemError> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        for &c in chars {
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
                        return Err(ExpertSystemError::InvalidToken(
                            line_info.clone(),
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
                        return Err(ExpertSystemError::InvalidToken(
                            line_info.clone(),
                            current_token,
                        ));
                    }
                }
            }
        }

        if !current_token.is_empty() {
            return Err(ExpertSystemError::InvalidToken(
                line_info.clone(),
                current_token,
            ));
        }

        Ok(tokens)
    }

    fn check(tokens: &[Token], line_info: &LineInfo) -> Result<HashSet<char>, ExpertSystemError> {
        let mut cnt_open = 0;
        let mut implication_type = None;
        let mut facts_before_implication = HashSet::new();
        let mut facts_after_implication = HashSet::new();

        for token in tokens {
            match token {
                Token::LeftParenthesis => cnt_open += 1,
                Token::RightParenthesis => {
                    if cnt_open == 0 {
                        return Err(ExpertSystemError::UnbalancedParentheses(line_info.clone()));
                    }
                    cnt_open -= 1;
                }
                Token::Implication | Token::ConverseImplication | Token::Equivalence => {
                    if implication_type.is_some() {
                        return Err(ExpertSystemError::MultipleImplications(line_info.clone()));
                    }
                    if cnt_open != 0 {
                        return Err(ExpertSystemError::ParenthesesAroundImplication(
                            line_info.clone(),
                        ));
                    }
                    implication_type = Some(token);
                }
                Token::Fact(c) => {
                    if implication_type.is_some() {
                        facts_after_implication.insert(*c);
                    } else {
                        facts_before_implication.insert(*c);
                    }
                }
                Token::Xor | Token::Or | Token::And | Token::Not => {}
            }
        }

        if cnt_open != 0 {
            return Err(ExpertSystemError::UnbalancedParentheses(line_info.clone()));
        }

        let Some(implication_type) = implication_type else {
            return Err(ExpertSystemError::MissingImplication(line_info.clone()));
        };

        Ok(match implication_type {
            Token::Equivalence => {
                facts_before_implication.extend(facts_after_implication);
                facts_before_implication
            }
            Token::Implication => facts_after_implication,
            Token::ConverseImplication => facts_before_implication,
            Token::Fact(_)
            | Token::Xor
            | Token::Or
            | Token::And
            | Token::Not
            | Token::LeftParenthesis
            | Token::RightParenthesis => unreachable!(),
        })
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

    fn build(tokens: Vec<Token>, line_info: &LineInfo) -> Result<Self, ExpertSystemError> {
        let mut rules = Vec::new();

        for token in tokens {
            match token {
                Token::Fact(c) => rules.push(Self::Fact(c)),
                Token::Not => {
                    if let Some(rule) = rules.pop() {
                        rules.push(Self::Not(Box::new(rule)));
                    } else {
                        return Err(ExpertSystemError::InvalidExpression(line_info.clone()));
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
                        return Err(ExpertSystemError::InvalidExpression(line_info.clone()));
                    }
                }
                Token::LeftParenthesis | Token::RightParenthesis => unreachable!(),
            }
        }

        if rules.len() != 1 {
            return Err(ExpertSystemError::InvalidExpression(line_info.clone()));
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
    pub fn apply_de_morgan(&mut self) -> bool {
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

    pub fn remove_xor_not_not(&mut self) {
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

    pub fn remove_double_negation(&mut self) {
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

    pub fn merge(&mut self, other: Self) {
        if self.is_tautology() {
            *self = other;
        } else if !other.is_tautology() {
            // TODO: no clone
            *self = Self::And(Box::new(self.clone()), Box::new(other));
        }
    }

    pub fn set_facts(&mut self, given_facts: &HashSet<char>, antifacts: &HashSet<char>) {
        match self {
            Self::Fact(f) => {
                if given_facts.contains(f) {
                    *self = Self::Bool(true);
                } else if antifacts.contains(f) {
                    *self = Self::Bool(false);
                }
            }
            Self::Bool(_) => {}
            Self::Not(rule) => {
                rule.set_facts(given_facts, antifacts);
                if let Self::Bool(b) = rule.as_ref() {
                    *self = Self::Bool(!b);
                }
            }
            Self::And(rule1, rule2) => {
                rule1.set_facts(given_facts, antifacts);
                rule2.set_facts(given_facts, antifacts);
                match (rule1.as_ref(), rule2.as_ref()) {
                    (Self::Bool(b1), Self::Bool(b2)) => *self = Self::Bool(*b1 && *b2),
                    (Self::Bool(true), child) | (child, Self::Bool(true)) => *self = child.clone(),
                    (Self::Bool(false), _) | (_, Self::Bool(false)) => *self = Self::Bool(false),
                    _ => {}
                }
            }
            Self::Or(rule1, rule2) => {
                rule1.set_facts(given_facts, antifacts);
                rule2.set_facts(given_facts, antifacts);
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
                rule1.set_facts(given_facts, antifacts);
                rule2.set_facts(given_facts, antifacts);
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

    fn evaluate_with_variables(&self, values: &HashMap<char, bool>) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Fact(c) => values[c],
            Self::Not(node) => !node.evaluate_with_variables(values),
            Self::Or(node1, node2) => {
                node1.evaluate_with_variables(values) || node2.evaluate_with_variables(values)
            }
            Self::And(node1, node2) => {
                node1.evaluate_with_variables(values) && node2.evaluate_with_variables(values)
            }
            Self::Xor(node1, node2) => {
                node1.evaluate_with_variables(values) ^ node2.evaluate_with_variables(values)
            }
        }
    }

    // TODO: remove (store variables directly in Rule)
    pub fn get_variables(&self) -> Vec<char> {
        fn helper(tree: &Rule, variables: &mut HashSet<char>) {
            match tree {
                Rule::Bool(_) => {}
                Rule::Fact(c) => {
                    variables.insert(*c);
                }
                Rule::Not(node) => helper(node, variables),
                Rule::Or(node1, node2) | Rule::And(node1, node2) | Rule::Xor(node1, node2) => {
                    helper(node1, variables);
                    helper(node2, variables);
                }
            }
        }

        let mut variables = HashSet::new();
        helper(self, &mut variables);
        variables.into_iter().sorted().collect_vec()
    }

    fn is_satisfiable_with_facts(
        &self,
        variables: &[char],
        idx: usize,
        fact_values: &mut HashMap<char, bool>,
    ) -> bool {
        if idx == variables.len() {
            return self.evaluate_with_variables(fact_values);
        }

        let variable = variables[idx];

        if fact_values.contains_key(&variable) {
            return self.is_satisfiable_with_facts(variables, idx + 1, fact_values);
        }

        fact_values.insert(variable, false);
        if self.is_satisfiable_with_facts(variables, idx + 1, fact_values) {
            return true;
        }
        fact_values.insert(variable, true);
        if self.is_satisfiable_with_facts(variables, idx + 1, fact_values) {
            return true;
        }
        fact_values.remove_entry(&variable);

        false
    }

    pub fn is_satisfiable(&self) -> bool {
        self.is_satisfiable_with_facts(&self.get_variables(), 0, &mut HashMap::new())
    }

    pub fn is_satisfiable_with_fact(&self, fact: char, value: bool) -> bool {
        let mut variables = self.get_variables();
        variables.retain(|v| *v != fact);
        self.is_satisfiable_with_facts(&variables, 0, &mut HashMap::from([(fact, value)]))
    }

    pub const fn tautology() -> Self {
        Self::Bool(true)
    }

    const fn is_tautology(&self) -> bool {
        matches!(self, Self::Bool(true))
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
