use std::u8;

#[derive(Debug)]
pub enum Token {
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Colon,
    Comma,
    Literal(String),
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
        if self.pos >= self.buffer.len() {
            panic!("new position {} out of bounds for buffer length {}", self.pos, self.buffer.len());
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

    fn search(&mut self, codepoint: u8) {
        // this ensures that we don't select the current position
        self.marker = self.pos + 1;
        while self.mark() != codepoint {
            self.marker += 1;
            if self.marker >= self.buffer.len() {
                panic!("codepoint {} never found", codepoint as char);
            }
        }
    }

    fn highlighted(&self) -> &[u8] {
        &self.buffer[self.pos..=self.marker]
    }

    pub fn tokenify(&mut self) -> Vec<Token> {
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
                    self.search(b'"');
                    tokens.push(Token::Literal(String::from_utf8(self.highlighted().to_vec()).unwrap()));
                    self.pos = self.marker + 1;
                },
                _ => panic!("invalid character {} at line {}, column {}", self.line, self.column, self.curr() as char)
            }
        }
    }
}
