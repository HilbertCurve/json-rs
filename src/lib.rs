mod lexer;
mod parser;
mod json;

pub use json::*;

#[cfg(test)]
mod tests {
    use crate::JSONValue;

    use super::lexer::{Lexer, Token};

    #[test]
    fn lexer_test() {
        use Token::*;
        let new_buf = 
        "{
            \"foo\": \"bar\",
            \"baz\": 134.0e-3,
            \"beanz\": [false, true]
        }";
        let mut lexer: Lexer = Lexer::new(new_buf.as_bytes().to_vec());

        let tokens = lexer.tokenify();
        
        assert_eq!(vec![
            OpenBrace,
                StringLiteral("\"foo\"".to_owned()), Colon, StringLiteral("\"bar\"".to_owned()), Comma,
                StringLiteral("\"baz\"".to_owned()), Colon, NumericLiteral("134.0e-3".to_owned()), Comma,
                StringLiteral("\"beanz\"".to_owned()), Colon, OpenBracket, False, Comma, True, CloseBracket,
            CloseBrace], tokens)
    }

    #[test]
    fn parser_test() {
        let new_buf = std::fs::read("tests/test.json").unwrap();
        println!("{:?}", JSONValue::from(new_buf));
    }
}
