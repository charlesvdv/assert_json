//! Simple and declarative way for testing JSON
//!
//! ```
//! # use assert_json::assert_json;
//! #
//! let key2_value = 2.0;
//! // On the left, the raw JSON as a string. On the right, the validator parts
//! assert_json!(r#"{ "key": "value", "key2": 2.0 }"#, {"key": "value", "key2": key2_value})
//! ```
//!
//! [assert_json!] asserts a given JSON input matches the expected validation rules.
//! The validation rules are given via a JSON-like structure. This JSON-like structure
//! is expanded by the `assert_json!` macro into validation rules. `assert_json!` is
//! heavily inspired by the serde [serde_json::json!] macro.
//!
//! Custom validators can also be used for more complex use-cases. A validator is implementing
//! the [Validator] trait. For common use-cases, you can also see the functions in [validators]
//! module.
//!
//! ```
//! # use assert_json::assert_json;
//! #
//! use assert_json::validators;
//!
//! let id_validator = validators::u64(|&v| if v > 0 { Ok(()) } else { Err(format!("{} is lower than 0", v)) });
//! assert_json!(r#"{ "id": 5, "username": "cvandevo" }"#, { "id": id_validator, "username": "cvandevo" })
//! ```

/// A JSON-value. Used by the [Validator] trait.
pub type Value = serde_json::Value;

/// Validation error
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error<'a> {
    #[error("Invalid type. Expected '{1}' got '{0}'.")]
    InvalidType(&'a Value, String),
    #[error("Invalid value: {1}")]
    InvalidValue(&'a Value, String),
    #[error("Missing key '{1}' in object")]
    MissingObjectKey(&'a Value, String),
    #[error("Key '{1}' is not expected in object")]
    UnexpectedObjectKey(&'a Value, String),
}

/// Define a validation on a given JSON [Value].
/// Validator can also be chained with the [Validator::and] method.
pub trait Validator {
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>>;

    fn and<T>(self, validator: T) -> And<Self, T>
    where
        Self: Sized,
        T: Validator,
    {
        And {
            first: self,
            second: validator,
        }
    }
}

#[doc(hidden)]
pub struct And<T, U> {
    first: T,
    second: U,
}

impl<T, U> Validator for And<T, U>
where
    T: Validator,
    U: Validator,
{
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        self.first.validate(value).and(self.second.validate(value))
    }
}

/// Define custom validators for different JSON types
pub mod validators;

#[macro_use]
mod macros;
#[doc(hidden)]
pub mod macros_utils;
