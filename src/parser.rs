use std::collections::HashMap;

use crate::lexer::Token;
use crate::json::*;

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

    fn expect(&mut self, expected: Token) {
        if self.curr() == expected {
            self.pos += 1;
        } else {
            panic!("expected {:?}, found {:?}", expected, self.curr())
        }
    }

    /// Parse tokens in current
    pub fn parse(&mut self) -> JSONValue {
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
                        _ => panic!("expected string literal"),
                    };
                    self.advance(1);

                    // expect a colon
                    self.expect(Token::Colon);
                    // expect a JSONValue
                    let val = self.parse();
                    self.advance(1);

                    println!("{}", self.pos);

                    ret.insert(key, val);

                    if self.curr() == Token::CloseBrace {
                        break;
                    }
                    self.expect(Token::Comma);
                }
                
                JSONValue::Object(ret)
            },
            Token::CloseBrace => {
                panic!("...")
            },
            Token::OpenBracket => {
                // begin array
                let mut ret: Vec<JSONValue> = vec![];

                // parse next token continuously, until the end of the array is reached
                self.pos += 1;
                loop {
                    ret.push(self.parse());
                    // moves us off of value
                    self.pos += 1;

                    // if we're at the end of the array...
                    if self.curr() == Token::CloseBracket {
                        break;
                    }

                    self.expect(Token::Comma);
                }

                JSONValue::Array(ret)
            },
            Token::CloseBracket => {
                panic!("...")
            },
            Token::Colon => {
                panic!("...")
            },
            Token::Comma => {
                panic!("...")
            },
            Token::StringLiteral(val) => {
                // begin string
                // StringLiteral includes the '"' characters; filter those off

                JSONValue::String(val[1..val.len() - 1].to_owned())
            },
            Token::NumericLiteral(val) => {
                // begin number
                // TODO: does this parse every type of JSON number??
                JSONValue::Number(val.parse().unwrap())
            },
            Token::True => {
                JSONValue::Bool(true)
            },
            Token::False => {
                JSONValue::Bool(false)
            },
            Token::Unknown(_) => {
                panic!("unknown ...")
            }
        }
    }
}
