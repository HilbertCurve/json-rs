use std::collections::HashMap;

use crate::lexer::{Token, TokenPos};
use crate::json::{*, self};

pub struct Parser {
    /// Array of lexed tokens
    tokens: Vec<TokenPos>,
    /// Current token
    pos: usize,
}

impl From<Vec<TokenPos>> for Parser {
    fn from(tokens: Vec<TokenPos>) -> Self {
        Self {
            tokens,
            pos: 0,
        }
    }
}

impl Parser {
    #[inline]
    fn curr(&self) -> Token {
        self.tokens[self.pos].0.clone()
    }
    #[inline]
    fn advance(&mut self, len: usize) {
        self.pos += len;
    }

    fn expect(&mut self, expected: Token) -> json::Result<()> {
        if self.curr() == expected {
            self.pos += 1;
            Ok(())
        } else {
            Err(JSONError::SyntaxError(format!("expected {:?}, found {:?}", expected, self.curr())))
        }
    }

    /// Parse tokens in current
    pub fn parse(&mut self) -> json::Result<JSONValue> {
        let (line, column) = (self.tokens[self.pos].1, self.tokens[self.pos].2);
        match self.curr().clone() {
            Token::OpenBrace => {
                // begin object
                let mut ret: HashMap<String, JSONValue> = HashMap::new();

                self.advance(1);

                // catches the case of an empty object
                if self.curr() == Token::CloseBrace {
                    return Ok(JSONValue::Object(ret))
                }

                // while last character is a comma
                loop {
                    // expect a string literal as a key
                    let key = match self.curr().clone() {
                        // chops off the quotations
                        Token::StringLiteral(val) => val[1..val.len() - 1].to_owned(),
                        _ => return Err(JSONError::SyntaxError(format!("expected string literal at line {line}, column {column}"))),
                    };
                    self.advance(1);

                    // expect a colon
                    self.expect(Token::Colon)?;
                    // expect a JSONValue
                    let val = self.parse()?;
                    self.advance(1);

                    ret.insert(key, val);

                    if self.curr() == Token::CloseBrace {
                        break;
                    }
                    self.expect(Token::Comma)?;
                }
                
                Ok(JSONValue::Object(ret))
            },
            Token::CloseBrace => {
                Err(JSONError::SyntaxError(format!("unexpected token `CloseBrace` at line {line}, column {column}")))
            },
            Token::OpenBracket => {
                // begin array
                let mut ret: Vec<JSONValue> = vec![];

                // parse next token continuously, until the end of the array is reached
                self.pos += 1;

                // catch the case of an empty array
                if self.curr() == Token::CloseBracket {
                    return Ok(JSONValue::Array(ret));
                }

                loop {
                    ret.push(self.parse()?);
                    // moves us off of value
                    self.pos += 1;

                    // if we're at the end of the array...
                    if self.curr() == Token::CloseBracket {
                        break;
                    }

                    self.expect(Token::Comma)?;
                }

                Ok(JSONValue::Array(ret))
            },
            Token::CloseBracket => {
                Err(JSONError::SyntaxError(format!("unexpected token `CloseBracket` at line {line}, column {column}")))
            },
            Token::Colon => {
                Err(JSONError::SyntaxError(format!("unexpected token `Colon` at line {line}, column {column}")))
            },
            Token::Comma => {
                Err(JSONError::SyntaxError(format!("unexpected token `Comma` at line {line}, column {column}")))
            },
            Token::StringLiteral(val) => {
                // begin string

                // StringLiteral includes the '"' characters; filter those off
                let trimmed = val[1..val.len() - 1].to_owned();
                let t_iter: Vec<char> = trimmed.chars().collect();
                let mut formatted: Vec<char> = vec![];
                let mut i = 0;
                while i < t_iter.len() {
                    if t_iter[i] == '\\' {
                        formatted.push(match t_iter[i+1] {
                            '"' => '"',
                            '\\' => '\\',
                            '/' => '/',
                            'b' => '\u{0008}',
                            'f' => '\u{000c}',
                            'n' => '\u{000a}',
                            'r' => '\u{000d}',
                            't' => '\u{0009}',
                            'u' => {
                                let chars: String = t_iter[i+2..i+6].iter().collect();

                                let num = u16::from_str_radix(&chars, 16)
                                    .or(Err(JSONError::ValueError(format!("invalid hexadecimal code: {}", chars))))? as u32;
                                
                                i += 4;
                                match char::from_u32(num) {
                                    Some(v) => v,
                                    None => return Err(JSONError::ValueError(format!("invalid utf16 hexadecimal code: {}", &chars)))
                                }
                            }
                            _ => return Err(JSONError::ValueError(format!("invalid escape char: {}", t_iter[i+1])))
                        });
                        i += 1;
                    } else {
                        formatted.push(t_iter[i]);
                    }
                    i += 1;
                }

                Ok(JSONValue::String(String::from_utf8(formatted.iter().map(|c| *c as u8).collect()).unwrap()))
            },
            Token::NumericLiteral(val) => {
                // begin number
                Ok(JSONValue::Number(val.parse().unwrap()))
            },
            Token::True => {
                Ok(JSONValue::Bool(true))
            },
            Token::False => {
                Ok(JSONValue::Bool(false))
            },
            Token::Null => {
                Ok(JSONValue::Null)
            }
            Token::Unknown(text) => {
                Err(JSONError::SyntaxError(format!("unexpected token `{text}` at line {line}, column {column}")))
            }
        }
    }
}
