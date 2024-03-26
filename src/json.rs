use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use std::ops::{Index, IndexMut};

use crate::lexer::Lexer;
use crate::parser::Parser;

/// # JSONError
///
/// An enumeration of all possible errors that could be thrown when using this library. Some errors
/// are at the parsing level caught before a JSONValue is parsed, and others are caught while
/// using or processing it.
///
/// These errors are typically used in expressions using [`json::Result<T>`](Result).
#[derive(Debug)]
pub enum JSONError {
    /// An error involving the validity of an inputted JSON string. For example, this error would be
    /// returned if someone tries to parse JSON data with an unterminated string.
    SyntaxError(String),
    /// An error involving the types of operations being done on a `JSONValue`. For example, this
    /// error would be returned if someone tries to index a `Null` object.
    ValueError(String),
    /// An error used when trying to index a `JSONObject`.
    KeyError(String),
    /// An error used when trying to index a `JSONArray`.
    IndexError(String),
}

impl Error for JSONError {}

impl Display for JSONError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntaxError(what) => write!(f, "JSON Syntax Error: {}", what),
            Self::ValueError(what) => write!(f, "JSON Value Error: {}", what),
            Self::KeyError(what) => write!(f, "JSON Key Error: {}", what),
            Self::IndexError(what) => write!(f, "JSON Index Error: {}", what),
        }
    }
}

/// # JSONValue
///
/// The primitive enum type for all values that can be stored in a JSON file, with each enum variant
/// having a unary tuple type that mimics the behavior of that JSON object.
#[derive(Clone, Debug, PartialEq)]
pub enum JSONValue {
    /// The primitive boolean type.
    Bool(bool),
    /// The primitive numeric type. JSON numbers are automatically assumed to be double-precision
    /// floating point numbers.
    Number(f64),
    /// The primitive ASCII string type.
    String(String),
    /// The primitive Array type. Under the hood, this is just a vector of other `JSONValue`s.
    Array(Vec<JSONValue>),
    /// The primitive Object type. Under the hood, this is a HashMap between strings and
    /// `JSONValue`s, mirroring the object key-value pairs in JSON files.
    Object(HashMap<String, JSONValue>),
    /// The primitive Null type, similar to the Rust zero-sized tuple `()`.
    Null,
}

/// # json::Result
///
/// Primary form of error management, used like the `std::result::Result` type.
pub type Result<T> = std::result::Result<T, JSONError>;

impl JSONValue {
    ///////////////////////////////////////////////
    // Functions that assume `self` is an Object //
    ///////////////////////////////////////////////

    /// Queries for a reference to a value in a `JSONValue::Object`. This is a safer version of
    /// indexing by string with the angle bracket notation.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not a json `Object`,
    /// - `Err(KeyError)` if `key` is not found in this `Object`,
    /// - `Ok(&JSONValue)` with a reference to the queried value otherwise.
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

    /// Queries for a mutable reference to a value in a `JSONValue::Object`.This is a safer version
    /// of indexing by string with the angle bracket notation.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not a json `Object`,
    /// - `Err(KeyError)` if `key` is not found in this `Object`,
    /// - `Ok(&mut JSONValue)` with a mutable reference to the queried value otherwise.
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

    /// Inserts a `value` into a `JSONValue::Object`.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not a json `Object`,
    /// - `Err(KeyError)` if `key` is already found in this `Object`,
    /// - `Ok` with a reference to the queried value otherwise.
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

    /// Removes a `value` from a `JSONValue::Object`.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not a json `Object`,
    /// - `Err(KeyError)` if `key` is not found in this `Object`,
    /// - `Ok` with the key-value pair of the removed object.
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

    /// Queries for a reference to a value in a `JSONValue::Array`. This is a safer version of
    /// indexing by integer with the angle bracket notation.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not a json `Array`, or if `index` is larger than array size
    /// - `Ok(&JSONValue)` with a reference to the queried value otherwise.
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

    /// Queries for a mutable reference to a value in a `JSONValue::Array`. This is a safer version
    /// of indexing by integer with the angle bracket notation.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not a json `Array`, or if `index` is larger than array size
    /// - `Ok(&mut JSONValue)` with a mutable reference to the queried value otherwise.
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

    /// Adds a value at the end of a `JSONValue::Array`.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not the `Array` enum variant,
    /// - `Ok` otherwise.
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

    /// Removes the last value from a `JSONValue::Array`.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not the `Array` enum variant,
    /// - `Ok` with a reference to the queried value otherwise.
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

    /// Adds a value at position `pos` in a `JSONValue::Array`.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not the `Array` enum variant,
    /// - `Err(IndexError)` if `pos` is out of bounds for the array,
    /// - `Ok` otherwise.
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

    /// Removes the value at position `pos` in a `JSONValue::Array`.
    ///
    /// Returns:
    /// - `Err(ValueError)` if `self` is not the `Array` enum variant,
    /// - `Err(IndexError)` if `pos` is out of bounds for the array,
    /// - `Ok(JSONValue)`, the removed value otherwise.
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

    /// Constructs a JSON null value.
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

///////////////////////////////////
// JSON-to-Rust Type Conversions //
///////////////////////////////////

/// A helper trait that performs non-consuming type conversions from
/// JSONValues to Rust primitive types.
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

impl_cast_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32);

///////////////////////////////////
// Rust-to-JSON Type Conversions //
///////////////////////////////////

// equivalent to <Self as FromStr>::from_str(self, &Vec<u8>::to_string())
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

// macro for auto-implementing From<> traits for numeric types
macro_rules! impl_from_int {
    {$($type_name:ty) +} => {
        $(impl From<$type_name> for JSONValue {
            fn from(value: $type_name) -> Self {
                Self::Number(value as f64)
            }
        })+
    }
}



impl_from_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32);

// NOTE: this directly constructs a JSONValue::String, and does not perform any parsing
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

impl From<Vec<JSONValue>> for JSONValue {
    fn from(value: Vec<JSONValue>) -> Self {
        Self::Array(value)
    }
}


/// Constructs a JSON null value. Equivalent to Self::null()
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

///////////////////////////////
// JSON-Text I/O Conversions //
///////////////////////////////

// conversion from raw json text into a JSONValue
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

//////////////////////////////////////////
// Indexing without Result<> protection //
//////////////////////////////////////////

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

impl<T> PartialEq<T> for JSONValue
    where JSONValue: Cast<T>,
    T: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        let res: Result<T> = self.cast();
        match res {
            Ok(v) => &v == other,
            Err(_) => false,
        }
    }
}
