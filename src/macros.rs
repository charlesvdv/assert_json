#[macro_export]
macro_rules! assert_json {
    ($val:expr , $($validator:tt)+) => ({
        #[allow(unused_imports)]
        use $crate::Validator;
        use $crate::macros_utils::*;

        let validator = $crate::expand_json_validator!($($validator)+);
        let input = Into::<Input>::into($val).get();
        let result = validator.validate(&input);
        if let Err(error) = result {
            panic!("assertion failed: json: {}", error)
        }
    });
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
        $crate::expand_json_vec_validator![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        $crate::expand_json_vec_validator![$($elems),*]
    };

    // Next element is `null`.
    (@array [$($elems:expr,)*] null $($rest:tt)*) => {
        $crate::expand_json_validator!(@array [$($elems,)* Box::new($crate::expand_json_validator!(null))] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        $crate::expand_json_validator!(@array [$($elems,)* Box::new($crate::expand_json_validator!([$($array)*]))] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        $crate::expand_json_validator!(@array [$($elems,)* Box::new($crate::expand_json_validator!({$($map)*}))] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        $crate::expand_json_validator!(@array [$($elems,)* $crate::expand_json_validator!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        $crate::expand_json_validator!(@array [$($elems,)* $crate::expand_json_validator!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        $crate::expand_json_validator!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        $crate::json_unexpected!($unexpected)
    };

    // *******************************************************************
    // object handling
    // *******************************************************************

    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        $crate::expand_json_validator!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@object $object:ident [$($key:tt)+] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        $crate::json_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.insert(($($key)+).into(), $value);
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (: null $($rest:tt)*) $copy:tt) => {
        $crate::expand_json_validator!(@object $object [$($key)+] (Box::new($crate::expand_json_validator!(null))) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        $crate::expand_json_validator!(@object $object [$($key)+] (Box::new($crate::expand_json_validator!([$($array)*]))) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        $crate::expand_json_validator!(@object $object [$($key)+] (Box::new($crate::expand_json_validator!({$($map)*}))) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        $crate::expand_json_validator!(@object $object [$($key)+] ($crate::expand_json_validator!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        $crate::expand_json_validator!(@object $object [$($key)+] ($crate::expand_json_validator!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::expand_json_validator!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::expand_json_validator!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@object $object:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        $crate::json_unexpected!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        $crate::json_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        $crate::expand_json_validator!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Refuse to absorb colon token into key expression.
    (@object $object:ident ($($key:tt)*) (: $($unexpected:tt)+) $copy:tt) => {
        $crate::json_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        $crate::expand_json_validator!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    // *******************************************************************
    // primitive handling
    // *******************************************************************

    (null) => {
        $crate::validators::null()
    };

    ([]) => {
        $crate::validators::array_empty()
    };

    ([ $($tt:tt)+ ]) => {
        // {
        //     let mut validators_array = vec![];
        // }
        $crate::validators::array($crate::expand_json_validator!(@array [] $($tt)+))
        // $crate::Value::Array(json_internal!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::validators::object(std::collections::HashMap::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::validators::object({
            let mut object: std::collections::HashMap<String, Box<dyn $crate::Validator>> = std::collections::HashMap::new();
            $crate::expand_json_validator!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    ($other:expr) => {
        {
            let validator: ValidatorInput = $other.into();
            validator.get()
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
