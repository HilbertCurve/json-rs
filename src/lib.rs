mod lexer;
mod parser;
mod json;

#[cfg(test)]
mod tests {
    use super::lexer::Lexer;

    #[test]
    fn lexer_test() {
        let new_buf = "{}[] \n :,::,    \"asfasdf\" ";
        let mut lexer: Lexer = Lexer::new(new_buf.as_bytes().to_vec());

        let tokens = lexer.tokenify();

        println!("{:?}", tokens);
    }
}
