use std::collections::BTreeMap;
use std::io::IsTerminal as _;
use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor;

use crate::{validators, Error, Validator, Value};

pub struct Input(Value);

impl Input {
    #[must_use]
    pub fn get(self) -> Value {
        self.0
    }
}

impl From<&str> for Input {
    fn from(str_input: &str) -> Input {
        let value = serde_json::from_str(str_input).expect("failed to parse JSON");
        Input(value)
    }
}

impl From<Value> for Input {
    fn from(value: Value) -> Input {
        Input(value)
    }
}

pub struct ValidatorInput(Box<dyn Validator>);

impl ValidatorInput {
    #[must_use]
    pub fn get(self) -> Box<dyn Validator> {
        self.0
    }
}

#[doc(hidden)]
macro_rules! impl_from_validator_input_default {
    (
        $($ty:ty),*
    ) => {
        $(
            impl From<$ty> for ValidatorInput {
                #[inline]
                fn from(u: $ty) -> Self {
                    ValidatorInput(Box::new(validators::eq(u)))
                }
            }
        )*
    };
}

impl_from_validator_input_default!(
    String, bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64
);

impl From<&str> for ValidatorInput {
    fn from(str_input: &str) -> Self {
        ValidatorInput(Box::new(validators::eq(String::from(str_input))))
    }
}

impl<T> From<T> for ValidatorInput
where
    T: Validator + 'static,
{
    fn from(validator: T) -> Self {
        ValidatorInput(Box::new(validator))
    }
}

#[must_use]
pub fn format_error<'a>(json: &'a Value, error: &Error<'a>) -> String {
    let serializer = SpanSerializer::serialize(json);

    let mut files = SimpleFiles::new();
    let file = files.add("", serializer.serialized_json());

    let diagnostic = Diagnostic::error()
        .with_message("Invalid JSON")
        .with_labels(vec![Label::primary(
            file,
            serializer.span(error.location()),
        )
        .with_message(error.to_string())]);

    let config = term::Config::default();
    let bytes = Vec::<u8>::new();

    let bytes = if std::io::stderr().is_terminal() {
        let mut writer = termcolor::Ansi::new(bytes);
        term::emit(&mut writer, &config, &files, &diagnostic).unwrap();
        writer.into_inner()
    } else {
        let mut writer = termcolor::NoColor::new(bytes);
        term::emit(&mut writer, &config, &files, &diagnostic).unwrap();
        writer.into_inner()
    };

    String::from_utf8(bytes).unwrap()
}

/// Serialize a JSON [Value] and keeps the span information of each
/// elements.
#[derive(Default)]
struct SpanSerializer {
    spans: BTreeMap<*const Value, Range<usize>>,
    json: String,
    current_ident: usize,
}

impl SpanSerializer {
    fn serialize(input: &Value) -> SpanSerializer {
        let mut serializer = SpanSerializer::default();
        serializer.serialize_recursive(input);
        serializer
    }

    fn serialize_recursive(&mut self, input: &Value) {
        let start = self.json.len();

        match input {
            serde_json::Value::Null => self.json.push_str("null"),
            serde_json::Value::Bool(bool_val) => self.json.push_str(&format!("{bool_val}")),
            serde_json::Value::Number(num_val) => {
                self.json.push_str(&num_val.to_string());
            }
            serde_json::Value::String(str_val) => self.json.push_str(&format!("\"{str_val}\"")),
            serde_json::Value::Array(arr_val) => {
                self.json.push_str("[\n");
                self.current_ident += 1;
                for (index, item) in arr_val.iter().enumerate() {
                    if index != 0 {
                        self.json.push_str(",\n");
                    }
                    self.ident();
                    self.serialize_recursive(item);
                }
                self.json.push('\n');
                self.current_ident -= 1;
                self.ident();
                self.json.push(']');
            }
            serde_json::Value::Object(obj_val) => {
                self.json.push_str("{\n");
                self.current_ident += 1;
                for (index, (key, value)) in obj_val.iter().enumerate() {
                    if index != 0 {
                        self.json.push_str(",\n");
                    }
                    self.ident();
                    self.json.push_str(&format!("\"{key}\": "));
                    self.serialize_recursive(value);
                }
                self.json.push('\n');
                self.current_ident -= 1;
                self.ident();
                self.json.push('}');
            }
        }

        let end = self.json.len();
        self.spans
            .insert(std::ptr::from_ref::<Value>(input), start..end);
    }

    fn ident(&mut self) {
        self.json.push_str(&" ".repeat(self.current_ident * 4));
    }

    fn serialized_json(&self) -> &str {
        &self.json
    }

    fn span(&self, val: &Value) -> Range<usize> {
        self.spans
            .get(&std::ptr::from_ref::<Value>(val))
            .expect("expected span")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::SpanSerializer;
    use crate::Value;

    #[test]
    fn serializer_primitive() {
        let value = Value::Null;

        let serializer = SpanSerializer::serialize(&value);
        assert_eq!("null", serializer.serialized_json());
        assert_eq!(0..4, serializer.span(&value));
    }

    #[test]
    fn serialize_object() {
        let value = serde_json::json!({
            "key": "value",
            "key_2": 2.1,
        });
        let num_value = value.as_object().unwrap().get("key_2").unwrap();

        let serializer = SpanSerializer::serialize(&value);
        assert_eq!(
            indoc! {r#"
                {
                    "key": "value",
                    "key_2": 2.1
                }"#},
            serializer.serialized_json()
        );
        assert_eq!(35..38, serializer.span(num_value));
    }

    #[test]
    fn serialize_array() {
        let value = serde_json::json!([
            null,
            true,
            {
                "key": -5,
            }
        ]);

        let serializer = SpanSerializer::serialize(&value);
        assert_eq!(
            indoc! {r#"
                [
                    null,
                    true,
                    {
                        "key": -5
                    }
                ]"#},
            serializer.serialized_json()
        );
    }
}
