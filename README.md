# assert_json

A easy and declarative way to test JSON input in Rust.
`assert_json` is a Rust macro heavily inspired by serde [json macro](https://docs.serde.rs/serde_json/macro.json.html).
Instead of creating a JSON value from a JSON literal, `assert_json` makes sure
the JSON input conforms to the validation rules specified.

`assert_json` also output beautiful error message when a validation error occurs.

## How to use

```rust
use assert_json::assert_json;
use assert_json::validators;

#[test]
fn test_json_ok() {
    let json = r#"
        {
            "status": "success",
            "result": {
                "id": 5,
                "name": "charlesvdv"
            }
        }
    "#;

    let name = "charlesvdv";

    assert_json!(json, {
            "status": "success",
            "result": {
                "id": validators::u64(|&v| if v > 0 { Ok(())} else { Err(String::from("id should be greater than 0")) }),
                "name": name,
            }
        }
    );
}
```

Any variables or expressions are interpoled as validation rules matching the type and value
of the variable/expression passed to the macro.

Now, if JSON input is changed to something incorrect like this:

```diff
    let json = r#"
        {
            "status": "success",
            "result": {
                "id": 5,
-                "name": "charlesvdv"
+                "name": "incorrect name"
            }
        }
    "#;
```

You will get an comprehensible error message like this one:

```
thread 'xxxx' panicked at 'error: Invalid JSON
  ┌─ :4:17
  │
4 │         "name": "incorrect name"
  │                 ^^^^^^^^^^^^^^^^ Invalid value. Expected "charlesvdv" but got "incorrect name".
```

### Custom validators

A set of validators are already implemented in the `validators` module.
If required, one can also creates its own validation routine by implementing the `Validator` trait.

```rust
use assert_json::{assert_json, Error, Validator, Value};

fn optional_string(expected: Option<String>) -> impl Validator {
    OptionalStringValidator { expected }
}

/// Matches a null JSON value if expected is None, else check if the strings
/// are equals
struct OptionalStringValidator {
    expected: Option<String>,
}

impl Validator for OptionalStringValidator {
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        if let Some(expected_str) = &self.expected {
            let string_value = value
                .as_str()
                .ok_or_else(|| Error::InvalidType(value, String::from("string")))?;

            if expected_str == string_value {
                Ok(())
            } else {
                Err(Error::InvalidValue(value, expected_str.clone()))
            }
        } else {
            value.as_null()
                .ok_or_else(|| Error::InvalidType(value, String::from("null")))
        }
    }
}

let json = r#"
    {
        "key": "value",
        "none": null
    }
"#;
assert_json!(json, {
    "key": optional_string(Some(String::from("value"))),
    "none": optional_string(None),
});
```

## Alternatives

- [assert-json-diff](https://github.com/davidpdrsn/assert-json-diff)


## Acknowledgments

Thanks a lot to the [serde-rs/json](https://github.com/serde-rs/json) project members
and especially those who contributed to the `json!` macro.