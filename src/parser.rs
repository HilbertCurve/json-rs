use std::collections::HashMap;

use crate::lexer::Token;
use crate::json::{*, self};

pub struct Parser {
    /// Array of lexed tokens
    tokens: Vec<Token>,
    /// Current token
    pos: usize,
}

impl From<Vec<Token>> for Parser {
    fn from(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
        }
    }
}

impl Parser {
    #[inline]
    fn curr(&self) -> Token {
        self.tokens[self.pos].clone()
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
            Err(JSONError::ParseError(format!("expected {:?}, found {:?}", expected, self.curr())))
        }
    }

    /// Parse tokens in current
    pub fn parse(&mut self) -> json::Result<JSONValue> {
        match self.tokens[self.pos].clone() {
            Token::OpenBrace => {
                // begin object
                let mut ret: HashMap<String, JSONValue> = HashMap::new();

                self.advance(1);

                // while last character is a comma
                loop {
                    // expect a string literal as a key
                    let key = match self.tokens[self.pos].clone() {
                        // chops off the quotations
                        Token::StringLiteral(val) => val[1..val.len() - 1].to_owned(),
                        _ => return Err(JSONError::ParseError("expected string literal".to_owned())),
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
                Err(JSONError::ParseError("unexpected token: CloseBrace".to_owned()))
            },
            Token::OpenBracket => {
                // begin array
                let mut ret: Vec<JSONValue> = vec![];

                // parse next token continuously, until the end of the array is reached
                self.pos += 1;
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
                Err(JSONError::ParseError("unexpected token: CloseBracket".to_owned()))
            },
            Token::Colon => {
                Err(JSONError::ParseError("unexpected token: Colon".to_owned()))
            },
            Token::Comma => {
                Err(JSONError::ParseError("unexpected token: Comma".to_owned()))
            },
            Token::StringLiteral(val) => {
                // begin string
                // StringLiteral includes the '"' characters; filter those off

                Ok(JSONValue::String(val[1..val.len() - 1].to_owned()))
            },
            Token::NumericLiteral(val) => {
                // begin number
                // TODO: propper number parser
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
                Err(JSONError::ParseError(format!("unexpected token: {text}")))
            }
        }
    }
}
