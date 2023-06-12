mod lexer;
mod parser;
pub mod json;

#[cfg(test)]
mod tests {
    use crate::json::{JSONValue, self, Cast};

    use super::lexer::Lexer;

    #[test]
    fn lexer_test() {
        let new_buf = 
        "
{
    \"foo\": \"bar\",
    \"baz\": 134.0e-3,
    \"beanz\": $ [false, true]
}";
        let mut lexer: Lexer = Lexer::new(new_buf.as_bytes().to_vec());

        lexer.tokenify().expect_err("this should error");
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
        ]), JSONValue::try_from(buffer)?);

        let buffer = std::fs::read("tests/string.json").unwrap();
        assert_eq!(JSONValue::String("asdfa sdfasdf wallalla tryn 165-08 {}{}___--=+123,./<>?".to_owned()), JSONValue::try_from(buffer)?);

        Ok(())
    }

    macro_rules! int_test {
        ($var:expr, $($type_name:ty)+) => {
            $(let _var: $type_name = $var.cast()?;)+
        };
    }

    #[test]
    fn json_test() -> json::Result<()> {
        let value = JSONValue::try_from(std::fs::read("tests/test.json").unwrap())?;
        
        let a: String = value.get("foo")?.cast()?;
        assert_eq!("bar", a);
        let b: String = value["foo"].cast()?;
        assert_eq!(a, b);

        let bool: bool = value["baz"][0].cast()?;
        assert_eq!(bool, true);
        let bool_2: bool = value.get("baz")?.try_index(0)?.cast()?;
        assert_eq!(bool, bool_2);

        int_test!(value["obj"]["b"], i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32);
        let the_answer: u8 = value["qux"]["qux_obj"]["nest_arr"][3]["hi"].cast()?;
        assert_eq!(42, the_answer);

        Ok(())
    }

    // TODO
    #[test]
    fn serialize_test() -> json::Result<()> {
        Ok(())
    }

    #[test]
    fn big_parse_test() -> json::Result<()> {
        let mut s: String = String::from("{");

        for i in 0..2<<16 {
            s.push_str(&format!("\"name_{0}\":{0},", i));
        }

        s.push_str(&format!("\"name_{0}\":{0}", 2<<16));
        s.push('}');

        let value = JSONValue::try_from(s.as_bytes().to_vec())?;

        for i in 0..=2<<16 {
            let v: i32 = value[format!("name_{}", i)].cast()?;
            assert_eq!(i, v);
        }

        Ok(())
    }
}
