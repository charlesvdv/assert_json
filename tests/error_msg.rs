use std::{any::Any, io::IsTerminal as _};

use assert_json::{assert_json, validators};
use indoc::indoc;

macro_rules! assert_panic_output {
    ($expected_output:expr, $($assert:tt)+) => {{
        let out_result = std::panic::catch_unwind(|| $($assert)+);
        let err = out_result_to_string(out_result);
        let expected_output = $expected_output.trim();
        assert!(err.contains(expected_output), "\n\texpected:\n{expected_output}\n\tgot:\n{err}")
    }};
}

#[expect(unsafe_code)]
fn out_result_to_string(result: Result<(), Box<dyn Any + Send>>) -> String {
    let err = result.unwrap_err();
    let s = err
        .downcast::<String>()
        .expect("the assert output should be a String");

    // ANSI escapes should only be written when `assert_json!` is called from a terminal
    if std::io::stderr().is_terminal() {
        let bytes = strip_ansi_escapes::strip(s.into_bytes());
        unsafe { String::from_utf8_unchecked(bytes) }
    } else {
        *s
    }
}

#[test]
fn primitive_invalid_type() {
    let expected_output = indoc! {r"
          │
        1 │ true
          │ ^^^^ Invalid type. Expected number but got bool.
    "};

    assert_panic_output!(expected_output, assert_json!("true", 5));
}

#[test]
fn missing_object_key() {
    let expected_output = indoc! {r#"
        1 │ ╭ {
        2 │ │     "key": "val"
        3 │ │ }
          │ ╰─^ Missing key 'missing_key' in object
    "#};

    assert_panic_output!(
        expected_output,
        assert_json!(r#"{ "key": "val" }"#, {
            "key": "val",
            "missing_key": null,
        })
    );
}

#[test]
fn test_readme_example() {
    // If the error is updated, don't forget to update the README!
    let expected_output = indoc! {r#"
          │
        4 │         "name": "incorrect name"
          │                 ^^^^^^^^^^^^^^^^ Invalid value. Expected "charlesvdv" but got "incorrect name".
    "#};
    let json = r#"
        {
            "status": "success",
            "result": {
                "id": 5,
                "name": "incorrect name"
            }
        }
    "#;
    assert_panic_output!(
        expected_output,
        assert_json!(json, {
                "status": "success",
                "result": {
                    "id": validators::u64(|&v| if v > 0 { Ok(())} else { Err(String::from("id should be greater than 0")) }),
                    "name": "charlesvdv",
                }
            }
        )
    );
}
