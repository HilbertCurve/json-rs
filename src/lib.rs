mod lexer;
mod parser;
pub mod json;

#[cfg(test)]
mod tests {
    use crate::json::{JSONValue, self};

    use super::lexer::{Lexer, Token};

    extern crate reqwest;

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

        let tokens = lexer.tokenify().unwrap();
        
        assert_eq!(vec![
            OpenBrace,
                StringLiteral("\"foo\"".to_owned()), Colon, StringLiteral("\"bar\"".to_owned()), Comma,
                StringLiteral("\"baz\"".to_owned()), Colon, NumericLiteral("134.0e-3".to_owned()), Comma,
                StringLiteral("\"beanz\"".to_owned()), Colon, OpenBracket, False, Comma, True, CloseBracket,
            CloseBrace], tokens)
    }

    #[test]
    fn parser_test() -> json::Result<()> {
        let buffer = std::fs::read("tests/array.json").unwrap();
        assert_eq!(JSONValue::Array(vec![
            JSONValue::Number(1.0),
            JSONValue::Number(2.0),
            JSONValue::Number(3.0),
            JSONValue::Bool(true),
            JSONValue::Null,
        ]), JSONValue::new(buffer)?);

        let buffer = std::fs::read("tests/string.json").unwrap();
        assert_eq!(JSONValue::String("asdfa sdfasdf wallalla tryn 165-08 {}{}___--=+123,./<>?".to_owned()), JSONValue::new(buffer)?);

        let buffer = std::fs::read("tests/test.json").unwrap();
        let value = JSONValue::new(buffer)?;
        assert_eq!(&JSONValue::String("bar".to_owned()), value.get("foo")?);

        Ok(())
    }
    #[test]
    fn big_parse_test() -> json::Result<()> {
        Ok(())
    }
}
