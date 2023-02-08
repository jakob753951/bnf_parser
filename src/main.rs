#![feature(iterator_try_collect)]

use std::{error::Error, fmt::Display, fs, path::Path};

#[derive(Debug)]
enum ParseError {
    EmptyString,
    NoPatternMatch,
    RuleNotFound(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyString => writeln!(f, "Empty string"),
            ParseError::NoPatternMatch => writeln!(f, "No pattern match"),
            ParseError::RuleNotFound(rule_name) => writeln!(f, "Rule '{}' not found", rule_name),
        }
    }
}

impl Error for ParseError {}

#[derive(Debug)]
struct BNF {
    rules: Vec<Rule>,
}

impl BNF {
    pub fn parse(value: String) -> Result<BNF, ParseError> {
        let rules_strings = value
            .split("\n\n")
            .map(String::from)
            .collect::<Vec<String>>();

        let mut rules: Vec<Rule> = Vec::new();

        for rule_string in rules_strings {
            rules.push(Rule::parse_rule(rule_string, &rules)?);
        }

        Ok(BNF { rules })
    }
}

#[derive(Clone, Debug)]
struct Rule {
    name: String,
    expressions: Vec<Expression>,
}

impl Rule {
    fn parse_rule(value: String, rules: &Vec<Rule>) -> Result<Self, ParseError> {
        let parts = value
            .split("::=")
            .map(String::from)
            .collect::<Vec<String>>();
        let name = strip_whitespace(String::from(parts[0].clone()));

        let expressions = String::from(parts[1].clone())
            .split("|")
            .map(String::from)
            .map(|str| Expression::parse_expression(str, rules))
            .try_collect::<Vec<Expression>>()?;

        Ok(Rule { name, expressions })
    }
}

#[derive(Clone, Debug)]
enum Expression {
    ChainedExpression(Vec<Expression>),
    Rule(Rule),
    String(String),
}

impl Expression {
    fn get_rule(name: String, rules: &Vec<Rule>) -> Result<Rule, ParseError> {
        let rule = rules
            .iter()
            .find(|rule| rule.name == name)
            .ok_or(ParseError::RuleNotFound(name));

        match rule {
            Ok(rule) => Ok(rule.clone()),
            Err(e) => Err(e),
        }
    }

    pub fn parse_expression(value: String, rules: &Vec<Rule>) -> Result<Expression, ParseError> {
        let stripped_value = strip_whitespace(value);
        let first_char = stripped_value
            .clone()
            .chars()
            .next()
            .ok_or(ParseError::EmptyString)?;
        match first_char {
            '"' => Ok(Expression::String(stripped_value)),
            '<' => {
                if stripped_value.chars().filter(|c| c == &'>').count() == 1 {
                    return Ok(Expression::Rule(Self::get_rule(stripped_value, rules)?))
                }
                Ok(Expression::Rule(Self::get_rule(stripped_value, rules)?))
            },
            _ => Err(ParseError::NoPatternMatch),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("input.txt");
    let str = fs::read_to_string(path)?;

    let bnf = BNF::parse(str);

    println!("{:#?}", bnf);

    Ok(())
}

fn strip_whitespace(str: String) -> String {
    str.split_whitespace()
        .map(String::from)
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_whitespace_fucking_works() {
        let input = String::from(" test :) ");
        let actual = strip_whitespace(input);
        let expected = String::from("test :)");
        assert_eq!(actual, expected);
    }
}