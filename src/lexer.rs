use std::u8;

use crate::json::{self, JSONError};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Colon,
    Comma,
    StringLiteral(String),
    NumericLiteral(String),
    True,
    False,
    Null,
    Unknown(String),
}

#[derive(Clone, Debug)]
pub struct TokenPos(pub Token, pub usize, pub usize);

pub struct Lexer {
    buffer: Vec<u8>,
    pos: usize,
    marker: usize, 
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(buffer: Vec<u8>) -> Lexer {
        Lexer {
            buffer,
            pos: 0,
            marker: 0,
            line: 1,
            column: 1,
        }
    }

    #[inline]
    fn curr(&self) -> u8 {
        self.buffer[self.pos]
    }
    #[inline]
    fn mark(&self) -> u8 {
        self.buffer[self.marker]
    }

    /// Advance lexer by `len` bytes, adjusting column and line positions as necessary
    fn advance(&mut self, len: usize) -> json::Result<()> {
        // err if out of bounds
        if self.pos + len > self.buffer.len() {
            return Err(JSONError::LexerError(
                format!(
                    "new position {} out of bounds for buffer length {}",
                    self.pos + len,
                    self.buffer.len(),
                )
            ));
        }

        // find locations of all the line breaks
        let mut line_breaks: Vec<usize> = vec![];
        for i in self.pos..self.pos + len {
            if self.buffer[i] == b'\n' {
                line_breaks.push(i);
            }
        }

        // advance raw character position
        self.pos += len;
        self.marker = self.pos;

        // if line breaks were found...
        if line_breaks.len() > 0 {
            // increment line count by number of '\n' chars found
            self.line += line_breaks.len();
            // set column pos to the offset from the last line break
            self.column = self.pos - line_breaks.pop().unwrap();
        } else {
            // advance column position
            self.column += len;
        }

        Ok(())
    }

    fn seek(&mut self, codepoint: u8) -> json::Result<()> {
        // this ensures that we don't select the current position
        self.marker = self.pos + 1;
        while self.mark() != codepoint {
            self.marker += 1;
            if self.marker >= self.buffer.len() {
                return Err(JSONError::LexerError(
                    format!(
                        "codepoint {} never found",
                        codepoint as char,
                    )
                ));
            }
        }
        // to include seeked-for character
        self.marker += 1;

        Ok(())
    }

    fn seek_in(&mut self, low: u8, high: u8) {
        while self.marker < self.buffer.len() && self.mark() >= low && self.mark() <= high {
            self.marker += 1;
        }
    }

    fn seek_all(&mut self, values: &[u8]) {
        while self.marker < self.buffer.len() {
            if values.iter().any(|&val| val == self.mark()) {
                self.marker += 1;
            } else {
                break;
            }
        }
    }

    fn highlighted(&self) -> &str {
        core::str::from_utf8(&self.buffer[self.pos..self.marker]).unwrap()
    }

    pub fn tokenify(&mut self) -> json::Result<Vec<TokenPos>> {
        // quick and dirty; will switch to better system later
        const ALPHABET: [u8; 52] = [
            b'a', b'b', b'c', b'd', b'e', b'f', b'g',
            b'h', b'i', b'j', b'k', b'l', b'm', b'n',
            b'o', b'p', b'q', b'r', b's', b't', b'u',
            b'v', b'w', b'x', b'y', b'z',
            b'A', b'B', b'C', b'D', b'E', b'F', b'G',
            b'H', b'I', b'J', b'K', b'L', b'M', b'N',
            b'O', b'P', b'Q', b'R', b'S', b'T', b'U',
            b'V', b'W', b'X', b'Y', b'Z',
        ];

        self.pos = 0;

        let mut tokens: Vec<TokenPos> = vec![];

        loop {
            if self.pos == self.buffer.len() {
                break Ok(tokens);
            }
            match self.curr() {
                b'{' => {
                    tokens.push(TokenPos(Token::OpenBrace, self.line, self.column));
                    self.advance(1)?;
                },
                b'}' => {
                    tokens.push(TokenPos(Token::CloseBrace, self.line, self.column));
                    self.advance(1)?;
                },
                b'[' => {
                    tokens.push(TokenPos(Token::OpenBracket, self.line, self.column));
                    self.advance(1)?;
                },
                b']' => {
                    tokens.push(TokenPos(Token::CloseBracket, self.line, self.column));
                    self.advance(1)?;
                },
                b':' => {
                    tokens.push(TokenPos(Token::Colon, self.line, self.column));
                    self.advance(1)?;
                },
                b',' => {
                    tokens.push(TokenPos(Token::Comma, self.line, self.column));
                    self.advance(1)?;
                },
                b' ' => {
                    self.advance(1)?;
                },
                b'\n' => {
                    self.advance(1)?;
                },
                b'"' => {
                    self.seek(b'"')?;
                    tokens.push(TokenPos(
                        Token::StringLiteral(self.highlighted().to_owned()),
                        self.line,
                        self.column,
                    ));
                    self.advance(self.marker - self.pos)?;
                },
                b't' => {
                    self.seek_all(&ALPHABET);

                    if self.highlighted() == "true" {
                        tokens.push(TokenPos(Token::True, self.line, self.column));
                    } else {
                        tokens.push(TokenPos(
                            Token::Unknown(self.highlighted().to_owned()),
                            self.line,
                            self.column,
                        ));
                    }

                    self.advance(self.marker - self.pos)?;
                },
                b'f' => {
                    self.seek_all(&ALPHABET);

                    if self.highlighted() == "false" {
                        tokens.push(TokenPos(Token::False, self.line, self.column));
                    } else {
                        tokens.push(TokenPos(
                            Token::Unknown(self.highlighted().to_owned()),
                            self.line,
                            self.column,
                        ));
                    }

                    self.advance(self.marker - self.pos)?;
                },
                b'n' => {
                    self.seek_all(&ALPHABET);

                    if self.highlighted() == "null" {
                        tokens.push(TokenPos(Token::Null, self.line, self.column));
                    } else {
                        tokens.push(TokenPos(
                            Token::Unknown(self.highlighted().to_owned()),
                            self.line,
                            self.column,
                        ));
                    }

                    self.advance(self.marker - self.pos)?;
                },
                b'A'..=b'z' => {
                    self.seek_in(b'A', b'z');
                    tokens.push(TokenPos(
                        Token::Unknown(self.highlighted().to_owned()),
                        self.line,
                        self.column,
                    ));
                    self.advance(self.marker - self.pos)?;
                },
                b'0'..=b'9' | b'-' | b'+' | b'.' => {
                    const NUM_CHARS: [u8; 15] = [
                        b'0', b'1', b'2', b'3',
                        b'4', b'5', b'6', b'7',
                        b'8', b'9', b'.', b'e',
                        b'E', b'+', b'-',
                    ];
                    self.seek_all(&NUM_CHARS);
                    tokens.push(TokenPos(
                        Token::NumericLiteral(self.highlighted().to_owned()),
                        self.line,
                        self.column,
                    ));
                    self.advance(self.marker - self.pos)?;
                },
                _ => {
                    break Err(JSONError::LexerError(
                        format!(
                            "invalid character '{}' at line {}, column {}",
                            self.curr() as char,
                            self.line,
                            self.column,
                        )
                    ));
                }
            }
        }
    }
}
