use assert_json::assert_json;
use indoc::indoc;

macro_rules! assert_panic_output {
    ($output:expr, $($assert:tt)+) => {{
        let out_result = std::panic::catch_unwind(|| $($assert)+);
        let out_err = out_result.err().unwrap();
        assert!(out_err.is::<String>());
        let out = out_err.downcast_ref::<String>().unwrap();
        let out = String::from_utf8(strip_ansi_escapes::strip(out.clone().into_bytes()).unwrap()).unwrap();
        assert!(out.contains($output.trim()), "\n\texpected:\n{}\n\tgot:\n{}", $output.trim(), out)
    }};
}

#[test]
fn primitive_invalid_type() {
    let expected_output = indoc! {r#"
          │
        1 │ true
          │ ^^^^ Invalid type. Expected number but got bool.
    "#};

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
    )
}
