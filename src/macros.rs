#[macro_export]
macro_rules! assert_json {
    ($val:expr, $($validator:tt)+) => {
        use $crate::validators;
        use $crate::{Validator};

        struct Input(crate::Value);

        impl From<&str> for Input {
            fn from(str_input: &str) -> Input {
                let value = serde_json::from_str(str_input)
                    .expect("failed to parse JSON");
                Input(value)
            }
        }

        impl From<crate::Value> for Input {
            fn from(value: crate::Value) -> Input {
                Input(value)
            }
        }

        let validator = expand_json_validator!($($validator)+);
        let input: Input = $val.into();
        let result = validator.validate(&input.0);
        if let Err(error) = result {
            panic!("assertion failed: json: {}", error)
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! expand_json_validator {
    (null) => {
        validators::null()
    };

    (true) => {
        validators::bool_true()
    };

    (false) => {
        validators::bool_false()
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn assert_json_null() {
        assert_json!("null", null);
    }

    #[test]
    #[should_panic]
    fn assert_json_null_not_valid() {
        assert_json!("null", true);
    }
}
