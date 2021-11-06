#[macro_export]
macro_rules! assert_json {
    ($val:expr , $($validator:tt)+) => {
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

        struct ValidatorInput(Box<dyn Validator>);

        impl From<&str> for ValidatorInput {
            fn from(str_input: &str) -> ValidatorInput {
                let str_input = String::from(str_input);
                ValidatorInput(validators::string(Box::new(move |val| {
                    if val.as_str() == str_input {
                        Ok(())
                    } else {
                        Err(format!("expected '{}' got '{}'", str_input, val))
                    }
                })))
            }
        }

        impl From<Box<dyn Validator>> for ValidatorInput {
            fn from(validator: Box<dyn Validator>) -> ValidatorInput {
                ValidatorInput(validator)
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

/// Heavily inspired by https://github.com/serde-rs/json.
/// Thanks dtolnay!
#[macro_export]
#[doc(hidden)]
macro_rules! expand_json_validator {
    // *******************************************************************
    // object handling
    // *******************************************************************

    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        expand_json_validator!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@object $object:ident [$($key:tt)+] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        json_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.insert(($($key)+).into(), $value);
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (: null $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object [$($key)+] (expand_json_validator!(null)) $($rest)*);
    };

    // Next value is `true`.
    (@object $object:ident ($($key:tt)+) (: true $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object [$($key)+] (expand_json_validator!(true)) $($rest)*);
    };

    // Next value is `false`.
    (@object $object:ident ($($key:tt)+) (: false $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object [$($key)+] (expand_json_validator!(false)) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object [$($key)+] (expand_json_validator!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object [$($key)+] (expand_json_validator!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object [$($key)+] (expand_json_validator!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        expand_json_validator!(@object $object [$($key)+] (expand_json_validator!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        expand_json_validator!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        expand_json_validator!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@object $object:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        json_unexpected!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        json_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Refuse to absorb colon token into key expression.
    (@object $object:ident ($($key:tt)*) (: $($unexpected:tt)+) $copy:tt) => {
        json_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    // *******************************************************************
    // primitive handling
    // *******************************************************************

    (null) => {
        validators::null()
    };

    (true) => {
        validators::bool_true()
    };

    (false) => {
        validators::bool_false()
    };

    ({}) => {
        validators::object(HashMap::new())
    };

    ({ $($tt:tt)+ }) => {
        validators::object({
            let mut object = std::collections::HashMap::new();
            expand_json_validator!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    ($other:expr) => {
        {
            let validator: ValidatorInput = $other.into();
            validator.0
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! json_unexpected {
    () => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! json_expect_expr_comma {
    ($e:expr , $($tt:tt)*) => {};
}

#[cfg(test)]
mod test {
    #[test]
    fn assert_json_null() {
        assert_json!("null", null);
    }

    #[test]
    fn assert_json_validator() {
        assert_json!("null", crate::validators::any());
    }

    #[test]
    #[should_panic]
    fn assert_json_null_not_valid() {
        assert_json!("null", true);
    }

    #[test]
    fn assert_simple_object_bool_value() {
        assert_json!(r#"{"key": true}"#, { "key": true });
    }

    #[test]
    fn assert_simple_object_str_value() {
        assert_json!(r#"{"key": "value"}"#, { "key": "value" });
    }

    #[test]
    fn assert_json_string() {
        assert_json!(r#""test""#, "test");
    }
}
