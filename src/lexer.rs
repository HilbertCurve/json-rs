use std::u8;

#[derive(Debug, Clone, PartialEq)]
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
    Unknown(String),
}

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
            line: 0,
            column: 0,
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
    fn advance(&mut self, len: usize) {
        // panic if out of bounds
        if self.pos + len > self.buffer.len() {
            panic!("new position {} out of bounds for buffer length {}", self.pos + len, self.buffer.len());
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
    }

    fn seek(&mut self, codepoint: u8) {
        // this ensures that we don't select the current position
        self.marker = self.pos + 1;
        while self.mark() != codepoint {
            self.marker += 1;
            if self.marker >= self.buffer.len() {
                panic!("codepoint {} never found", codepoint as char);
            }
        }
        // to include seeked-for character
        self.marker += 1;
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

    pub fn tokenify(&mut self) -> Vec<Token> {
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

        let mut tokens: Vec<Token> = vec![];

        loop {
            if self.pos == self.buffer.len() {
                break tokens;
            }
            match self.curr() {
                b'{' => {
                    tokens.push(Token::OpenBrace);
                    self.advance(1);
                },
                b'}' => {
                    tokens.push(Token::CloseBrace);
                    self.advance(1);
                },
                b'[' => {
                    tokens.push(Token::OpenBracket);
                    self.advance(1);
                },
                b']' => {
                    tokens.push(Token::CloseBracket);
                    self.advance(1);
                },
                b':' => {
                    tokens.push(Token::Colon);
                    self.advance(1);
                },
                b',' => {
                    tokens.push(Token::Comma);
                    self.advance(1);
                },
                b' ' => {
                    self.advance(1);
                },
                b'\n' => {
                    self.advance(1);
                },
                b'"' => {
                    self.seek(b'"');
                    tokens.push(Token::StringLiteral(self.highlighted().to_owned()));
                    self.advance(self.marker - self.pos);
                },
                b't' => {
                    self.seek_all(&ALPHABET);

                    if self.highlighted() == "true" {
                        tokens.push(Token::True);
                    } else {
                        tokens.push(Token::Unknown(self.highlighted().to_owned()));
                    }

                    self.advance(self.marker - self.pos);
                },
                b'f' => {
                    self.seek_all(&ALPHABET);

                    if self.highlighted() == "false" {
                        tokens.push(Token::False);
                    } else {
                        tokens.push(Token::Unknown(self.highlighted().to_owned()));
                    }

                    self.advance(self.marker - self.pos);
                },
                b'A'..=b'z' => {
                    self.seek_in(b'A', b'z');
                    tokens.push(Token::Unknown(self.highlighted().to_owned()));
                    self.advance(self.marker - self.pos);
                },
                b'0'..=b'9' => {
                    const NUM_CHARS: [u8; 15] = [
                        b'0', b'1', b'2', b'3',
                        b'4', b'5', b'6', b'7',
                        b'8', b'9', b'.', b'e',
                        b'E', b'+', b'-',
                    ];
                    self.seek_all(&NUM_CHARS);
                    tokens.push(Token::NumericLiteral(self.highlighted().to_owned()));
                    self.advance(self.marker - self.pos);
                },
                _ => panic!("invalid character '{}' at line {}, column {}", self.curr() as char, self.line, self.column)
            }
        }
    }
}
