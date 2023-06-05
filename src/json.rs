use std::collections::HashMap;

use crate::{lexer::Lexer, parser::Parser};

#[derive(Debug)]
pub enum JSONValue {
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JSONValue>),
    Object(HashMap<String, JSONValue>),
}

impl From<Vec<u8>> for JSONValue {
    fn from(value: Vec<u8>) -> Self {
        Parser::from(Lexer::new(value).tokenify()).parse()
    }
}
