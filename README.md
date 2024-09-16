# json-rs
A lightweight json parser and serializer.

This crate does not rely on any I/O actions, and purely works with `&str` objects.

Example:

`my_file.json`:
```json
{
    "foo": "bar",
    "baz": [
        2,
        3.4,
        false
    ],
    "nested": {
        "inner_foo": "inner_bar",
        "has_answer": [
            40,
            41,
            42,
            43e1
        ]
    }
}
```
`main.rs`:
```rust
use std::fs;

fn main() -> json::Result<()> {
    let values: JSONValue = JSONValue::from_str(fs::read("my_file.json"))?;
    let bar: String = values["foo"].cast()?;
    assert_eq!(values["foo"], "bar");
    assert_eq!(values["baz"][2], false);
    assert_eq!(values["nested"]["has_answer"][2], 42);
}
```

## Todo:
- Documentation 💀
- More Tests
- Ensure JSON Compliance: string literal types, escape chars, etc.
- Work on macro stuff
