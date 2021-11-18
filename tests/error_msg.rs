use assert_json::assert_json;
use indoc::indoc;

macro_rules! assert_panic_output {
    ($output:expr, $($assert:tt)+) => {{
        let out_result = std::panic::catch_unwind(|| $($assert)+);
        let out_err = out_result.err().unwrap();
        assert!(out_err.is::<String>());
        let out = out_err.downcast_ref::<String>().unwrap();
        let out = String::from_utf8(strip_ansi_escapes::strip(out.clone().into_bytes()).unwrap()).unwrap();
        assert!(out.contains($output))
    }};
}

#[test]
fn primitive_invalid_type() {
    assert_panic_output!(
        indoc! {r#"
  │
1 │ null
  │ ^^^^ Invalid value: 5"#},
        assert_json!("null", 5)
    );
}
