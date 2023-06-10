use std::{collections::HashMap, fmt::Display, str::FromStr, any::Any};

use crate::{lexer::Lexer, parser::Parser};

#[derive(Debug)]
pub enum JSONError {
    SyntaxError(String),
    LexerError(String),
    ParseError(String),
    ValueError(String),
    KeyError(String),
}

#[derive(Debug, PartialEq)]
pub enum JSONValue {
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JSONValue>),
    Object(HashMap<String, JSONValue>),
    Null,
}

pub type Result<T> = std::result::Result<T, JSONError>;

impl JSONValue {
    pub fn get(&self, key: &str) -> Result<&JSONValue> {
        match self {
            Self::Object(vals) => {
                if let Some(val) = vals.get(key) {
                    Ok(val)
                } else {
                    Err(JSONError::KeyError(format!("key {} not found", key)))
                }
            }
            other => {
                Err(JSONError::ValueError(format!("expected Object, found {:?}", other)))
            }
        }
    }
    pub fn get_mut(&mut self, key: &str) -> Result<&mut JSONValue> {
        match self {
            Self::Object(vals) => {
                if let Some(val) = vals.get_mut(key) {
                    Ok(val)
                } else {
                    Err(JSONError::KeyError(format!("key {} not found", key)))
                }
            }
            other => {
                Err(JSONError::ValueError(format!("expected Object, found {:?}", other)))
            }
        }
    }

    // helper function to assist with <JSONValue as Display>::fmt(). Allows printed
    // JSON text to auto-format spacing. 
    fn fmt_recursive(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        match self {
            Self::Bool(b) => { write!(f, "{}", b)?; }
            Self::Number(n) => { write!(f, "{}", n)?; }
            Self::String(s) => { write!(f, "\"{}\"", s)?; }
            Self::Array(arr) => {
                let tab_width = level * 4;
                write!(f, "[\n")?;
                for i in 0..arr.len() {
                    write!(f, "    {: <1$}", "", tab_width)?;
                    arr[i].fmt_recursive(f, level + 1)?;
                    if i != arr.len() - 1 {
                        write!(f, ",")?;
                    }
                    write!(f, "\n")?;
                }
                write!(f, "{: <1$}]", "", tab_width)?;
            }
            Self::Object(obj) => {
                let tab_width = level * 4;
                write!(f, "{{\n")?;
                let mut i = 0;
                for key in obj.keys() {
                    write!(f, "    {: <1$}", "", tab_width)?;
                    write!(f, "\"{}\": ", key)?;
                    obj[key].fmt_recursive(f, level + 1)?;
                    if i != obj.len() - 1 {
                        write!(f, ",")?;
                        i += 1;
                    }
                    write!(f, "\n")?;
                }
                write!(f, "{: <1$}}}", "", tab_width)?;
            }
            Self::Null => { write!(f, "null")?; }
        }

        Ok(())
    }
}

trait Cast<T> {
    fn cast(&self) -> Result<T>;
}

impl Cast<bool> for JSONValue {
    fn cast(&self) -> Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            other => Err(JSONError::ValueError(format!("expected boolean, found {:?}", other.type_id())))
        }
    }
}

impl TryFrom<Vec<u8>> for JSONValue {
    type Error = JSONError;

    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Parser::from(
            Lexer::new(value).tokenify()?
        ).parse()
    }
}

impl FromStr for JSONValue {
    type Err = JSONError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::try_from(s.as_bytes().to_vec())
    }
}

impl Display for JSONValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_recursive(f, 0)
    }
}
