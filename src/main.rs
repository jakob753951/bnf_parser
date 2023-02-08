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
        // new strategy to avoid dying on recursive or looping definitions:
        // parse all rule-names first
        // get expressions for each rule after
        // maybe do a HashSet of the rules?
        // the expressions can be parsed in the same way,
        // and the order of the rules won't matter
        // which means we can use the original spec without reordering
        let rules_strings = value
            .split("\n\n")
            .collect::<Vec<&str>>();

        let rule_names = rules_strings
            .iter()
            .map(|name| Rule::parse_name(&name))
            .try_collect::<Vec<&str>>()?;

        let mut rules: Vec<Rule> = Vec::new();

        for rule_string in rules_strings {
            rules.push(Rule::parse_rule(rule_string, &rules)?);
        }

        Ok(BNF { rules: rules })
    }
}

#[derive(Clone, Debug)]
struct Rule {
    name: String,
    expressions: Vec<Expression>,
}

impl Rule {
    fn parse_rule(value: &str, rules: &Vec<Rule>) -> Result<Self, ParseError> {
        let parts = value
            .split("::=")
            .map(String::from)
            .collect::<Vec<String>>();
        let name = parts[0].trim();

        let expressions = parts[1]
            .split("|")
            .map(|str| Expression::parse_expression(str, rules))
            .try_collect::<Vec<Expression>>()?;

        Ok(Rule {
            name: name.to_string(),
            expressions,
        })
    }

    fn parse_name<'a>(value: &'a str) -> Result<&'a str, ParseError> {
        Ok(
            value
                .split("::=")
                .next()
                .ok_or(ParseError::NoPatternMatch)?
                .clone()
                .trim()
        )
    }
}

#[derive(Clone, Debug)]
enum Expression {
    ChainedExpression(Vec<Expression>),
    Rule(Rule),
    String(String),
}

impl Expression {
    fn get_rule(name: &str, rules: &Vec<Rule>) -> Result<Rule, ParseError> {
        let rule = rules
            .iter()
            .find(|rule| rule.name == name)
            .ok_or(ParseError::RuleNotFound(name.to_string()));

        match rule {
            Ok(rule) => Ok(rule.clone()),
            Err(e) => Err(e),
        }
    }

    pub fn parse_expression(value: &str, rules: &Vec<Rule>) -> Result<Expression, ParseError> {
        let value = value.trim();
        let first_char = value
            .trim()
            .clone()
            .chars()
            .next()
            .ok_or(ParseError::EmptyString)?;
        match first_char {
            '"' => Ok(Expression::String(value.to_string())),
            '<' => {
                if value.chars().filter(|c| c == &'>').count() == 1 {
                    return Ok(Expression::Rule(Self::get_rule(value, rules)?));
                }

                let expressions = value
                    .split_whitespace()
                    .map(|name| Expression::parse_expression(name, rules))
                    .try_collect::<Vec<Expression>>()?;
                Ok(Expression::ChainedExpression(expressions))
            }
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
