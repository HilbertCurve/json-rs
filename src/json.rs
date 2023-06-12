use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use std::ops::{Index, IndexMut};

use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Debug)]
pub enum JSONError {
    SyntaxError(String),
    LexerError(String),
    ParseError(String),
    ValueError(String),
    KeyError(String),
    IndexError(String),
}

#[derive(Clone, Debug, PartialEq)]
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
    ///////////////////////////////////////////////
    // Functions that assume `self` is an Object //
    ///////////////////////////////////////////////

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
                Err(JSONError::ValueError(format!("expected object, found {:?}", other.name())))
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
                Err(JSONError::ValueError(format!("expected object, found {:?}", other.name())))
            }
        }
    }
    pub fn obj_insert(&mut self, key: &str, value: JSONValue) -> Result<()> {
        match self {
            Self::Object(map) => {
                if let Some(_) = map.get_mut(key) {
                    Err(JSONError::KeyError(format!("key {} already in object", key)))
                } else {
                    map.insert(key.to_string(), value);
                    Ok(())
                }
            }
            other => {
                Err(JSONError::ValueError(format!("expected object, found {:?}", other.name())))
            }
        }
    }
    pub fn obj_remove(&mut self, key: &str) -> Result<(String, JSONValue)> {
        match self {
            Self::Object(map) => {
                if let Some(v) = map.remove(key) {
                    Ok((key.to_string(), v))
                } else {
                    Err(JSONError::KeyError(format!("key {} not found", key)))
                }
            }
            other => {
                Err(JSONError::ValueError(format!("expected object, found {:?}", other.name())))
            }
        }
    }


    //////////////////////////////////////////////
    // Functions that assume `self` is an Array //
    //////////////////////////////////////////////

    pub fn try_index(&self, index: usize) -> Result<&JSONValue> {
        match self {
            Self::Array(arr) => {
                if let Some(val) = arr.get(index) {
                    Ok(val)
                } else {
                    Err(JSONError::ValueError(format!("index {} out of bounds for length {}", index, arr.len())))
                }
            }
            other => {
                Err(JSONError::ValueError(format!("expected array, found {}", other.name())))
            }
        }
    }
    pub fn try_index_mut(&mut self, index: usize) -> Result<&mut JSONValue> {
        match self {
            Self::Array(arr) => {
                let len = arr.len().clone();

                if let Some(val) = arr.get_mut(index) {
                    Ok(val)
                } else {
                    Err(JSONError::ValueError(format!("index {} out of bounds for length {}", index, len)))
                }
            }
            other => {
                Err(JSONError::ValueError(format!("expected array, found {}", other.name())))
            }
        }
    }
    pub fn arr_push(&mut self, val: JSONValue) -> Result<()> {
        match self {
            Self::Array(arr) => {
                arr.push(val);
                Ok(())
            }
            other => {
                Err(JSONError::ValueError(format!("expected array, found {}", other.name())))
            }
        }
    }
    pub fn arr_pop(&mut self) -> Result<JSONValue> {
        match self {
            Self::Array(arr) => {
                if let Some(v) = arr.pop() {
                    Ok(v)
                } else {
                    Err(JSONError::ValueError("cannot pop an array of zero length".to_string()))
                }
            }
            other => {
                Err(JSONError::ValueError(format!("expected array, found {}", other.name())))
            }
        }
    }
    pub fn arr_insert(&mut self, pos: usize, val: JSONValue) -> Result<()> {
        match self {
            Self::Array(arr) => {
                let len = arr.len().clone();

                if pos > len {
                    Err(JSONError::IndexError(format!("index {} out of bounds for length {}", pos, len)))
                } else {
                    arr.insert(pos, val);

                    Ok(())
                }
            }
            other => {
                Err(JSONError::ValueError(format!("expected array, found {}", other.name())))
            }
        }
    }
    pub fn arr_remove(&mut self, pos: usize) -> Result<JSONValue> {
        match self {
            Self::Array(arr) => {
                let len = arr.len().clone();

                if pos > len {
                    Err(JSONError::IndexError(format!("index {} out of bounds for length {}", pos, len)))
                } else {
                    Ok(arr.remove(pos))
                }
            }
            other => {
                Err(JSONError::ValueError(format!("expected array, found {}", other.name())))
            }
        }
    }

    #[inline]
    pub const fn null() -> Self {
        Self::Null
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

    // used for debug messages
    fn name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
            Self::Null => "null",
        }
    }
}

pub trait Cast<T> {
    fn cast(&self) -> Result<T>;
}

impl Cast<bool> for JSONValue {
    fn cast(&self) -> Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            other => Err(JSONError::ValueError(format!("expected boolean, found {:?}", other.name())))
        }
    }
}

impl Cast<f64> for JSONValue {
    fn cast(&self) -> Result<f64> {
        match self {
            Self::Number(v) => Ok(*v),
            other => Err(JSONError::ValueError(format!("expected number, found {:?}", other.name())))
        }
    }
}

impl Cast<String> for JSONValue {
    fn cast(&self) -> Result<String> {
        match self {
            Self::String(s) => Ok(s.clone()),
            other => Err(JSONError::ValueError(format!("expected string, found {:?}", other.name())))
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

impl From<f64> for JSONValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

// macro for implementing From<> traits for numeric types
macro_rules! impl_from_int {
    {$($type_name:ty) +} => {
        $(impl From<$type_name> for JSONValue {
            fn from(value: $type_name) -> Self {
                Self::Number(value as f64)
            }
        })+
    }
}

macro_rules! impl_cast_int {
    {$($type_name:ty) +} => {
        $(impl Cast<$type_name> for JSONValue {
            fn cast(&self) -> crate::json::Result<$type_name> {
                match self {
                    Self::Number(v) => Ok(v.clone() as $type_name),
                    other => Err(JSONError::ValueError(format!("expected number, found {:?}", other.name()))),
                }
            }
        })+
    }
}

impl_from_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32);
impl_cast_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32);

impl From<String> for JSONValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for JSONValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<&[JSONValue]> for JSONValue {
    fn from(value: &[JSONValue]) -> Self {
        Self::Array(value.to_vec())
    }
}

impl From<()> for JSONValue {
    fn from(_: ()) -> Self {
        Self::Null
    }
}

impl<T> From<Option<T>> for JSONValue where JSONValue: From<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => <Self as From<T>>::from(v),
            None => Self::Null,
        }
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

// index section. Usage of these traits doesn't have Result<> protection, use wisely!
impl Index<&str> for JSONValue {
    type Output = JSONValue;
    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<&str> for JSONValue {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl Index<String> for JSONValue {
    type Output = JSONValue;
    fn index(&self, index: String) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl IndexMut<String> for JSONValue {
    fn index_mut(&mut self, index: String) -> &mut Self::Output {
        self.get_mut(&index).unwrap()
    }
}

impl Index<usize> for JSONValue {
    type Output = JSONValue;
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            JSONValue::Array(arr) => &arr[index],
            other => panic!("expected array, found {:?}", other.name()),
        }
    }
}

impl IndexMut<usize> for JSONValue {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            JSONValue::Array(arr) => &mut arr[index],
            other => panic!("expected array, found {:?}", other.name()),
        }
    }
}
