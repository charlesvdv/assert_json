#[macro_export]
macro_rules! assert_json {
    ($val:expr , $($validator:tt)+) => ({
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

        impl_from_validator_input_default!(String, bool, u8, u16, u32, u64, usize,
            i8, i16, i32, i64, isize, f32, f64);

        impl From<&str> for ValidatorInput {
            fn from(str_input: &str) -> Self {
                ValidatorInput(Box::new(validators::eq(String::from(str_input))))
            }
        }

        impl<T> From<T> for ValidatorInput where T: Validator + 'static {
            fn from(validator: T) -> Self {
                ValidatorInput(Box::new(validator))
            }
        }

        let validator = expand_json_validator!($($validator)+);
        let input: Input = $val.into();
        let result = validator.validate(&input.0);
        if let Err(error) = result {
            panic!("assertion failed: json: {}", error)
        }
    });
}

#[macro_export]
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

/// Heavily inspired by https://github.com/serde-rs/json.
/// Thanks dtolnay!
#[macro_export]
#[doc(hidden)]
macro_rules! expand_json_validator {
    // *******************************************************************
    // array handling
    // *******************************************************************

    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        expand_json_vec_validator![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        expand_json_vec_validator![$($elems),*]
    };

    // Next element is `null`.
    (@array [$($elems:expr,)*] null $($rest:tt)*) => {
        expand_json_validator!(@array [$($elems,)* Box::new(expand_json_validator!(null))] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        expand_json_validator!(@array [$($elems,)* Box::new(expand_json_validator!([$($array)*]))] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        expand_json_validator!(@array [$($elems,)* Box::new(expand_json_validator!({$($map)*}))] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        expand_json_validator!(@array [$($elems,)* expand_json_validator!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        expand_json_validator!(@array [$($elems,)* expand_json_validator!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        expand_json_validator!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        json_unexpected!($unexpected)
    };

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
        expand_json_validator!(@object $object [$($key)+] (Box::new(expand_json_validator!(null))) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object [$($key)+] (Box::new(expand_json_validator!([$($array)*]))) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        expand_json_validator!(@object $object [$($key)+] (Box::new(expand_json_validator!({$($map)*}))) $($rest)*);
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

    ([]) => {
        validators::array_empty()
    };

    ([ $($tt:tt)+ ]) => {
        // {
        //     let mut validators_array = vec![];
        // }
        validators::array(expand_json_validator!(@array [] $($tt)+))
        // $crate::Value::Array(json_internal!(@array [] $($tt)+))
    };

    ({}) => {
        validators::object(std::collections::HashMap::new())
    };

    ({ $($tt:tt)+ }) => {
        validators::object({
            let mut object: std::collections::HashMap<String, Box<dyn Validator>> = std::collections::HashMap::new();
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

// The expand_json_validator macro above cannot invoke vec directly because it uses
// local_inner_macros. A vec invocation there would resolve to $crate::vec.
// Instead invoke vec here outside of local_inner_macros.
#[macro_export]
#[doc(hidden)]
macro_rules! expand_json_vec_validator {
    ($($content:tt)*) => {
        vec![$($content)*]
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
    fn assert_json_with_serde_input() {
        assert_json!(serde_json::json!("hello"), "hello");
    }

    #[test]
    fn assert_json_null() {
        assert_json!("null", null);
    }

    #[test]
    fn assert_json_number() {
        assert_json!("23", 23);
        assert_json!("2.3", 2.3);
    }

    #[test]
    fn assert_json_bool() {
        assert_json!("true", true);
        assert_json!("false", false);
    }

    #[test]
    fn assert_json_string() {
        assert_json!(r#""str""#, "str")
    }

    #[test]
    fn assert_json_object_empty() {
        assert_json!("{}", {});
    }

    #[test]
    fn assert_json_object() {
        assert_json!(r#"{
                "null": null,
                "bool_true": true,
                "bool_false": false,
                "num_int": -6,
                "num_float": 2.4,
                "str": "test",
                "inner_obj": {
                    "test": "hello"
                },
                "inner_empty_obj": {},
                "inner_array": [1, 3],
                "inner_empty_arr": []
            }"#,
            {
                "null": null,
                "bool_true": true,
                "bool_false": false,
                "num_int": -6,
                "num_float": 2.4,
                "str": "test",
                "inner_obj": {
                    "test": "hello"
                },
                "inner_empty_obj": {},
                "inner_array": [1, 3],
                "inner_empty_arr": [],
            }
        );
    }

    #[test]
    fn assert_json_array_empty() {
        assert_json!("[]", []);
    }

    #[test]
    #[should_panic]
    fn assert_json_array_empty_err() {
        assert_json!("[null]", []);
    }

    #[test]
    fn assert_json_array() {
        assert_json!(
            r#"[
                null,
                true,
                false,
                8,
                8.9,
                "str",
                { "key": null },
                {},
                [false, "hello"],
                []
            ]"#,
            [
                null,
                true,
                false,
                8,
                8.9,
                "str",
                { "key": null },
                {},
                [false, "hello"],
                []
            ]
        );
    }

    #[test]
    fn assert_json_custom_validator() {
        assert_json!("null", crate::validators::any());
    }

    #[test]
    fn assert_json_validator_with_and() {
        assert_json!(
            r#""test""#,
            crate::validators::any().and(crate::validators::eq(String::from("test")))
        );
    }

    #[test]
    #[should_panic]
    fn assert_json_null_not_valid() {
        assert_json!("null", true);
    }

    #[test]
    fn assert_json_is_expression() {
        assert_json!("null", null) // the missing ";" is normal
                                   // this is to test if the the assert_json macros
                                   // can be used as an expression like assert_eq!
    }

    #[test]
    fn assert_json_with_variable() {
        let num = 5;
        assert_json!("5", num);
    }
}
