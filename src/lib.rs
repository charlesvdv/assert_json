//! A easy and declarative way to test JSON input in Rust.
//!
//! [`assert_json`!] is a Rust macro heavily inspired by [`serde_json::json`!] macro.
//! Instead of creating a JSON value from a JSON literal, [`assert_json`!] makes sure
//! the JSON input conforms to the validation rules specified.
//!
//! [`assert_json`!] also output beautiful error message when a validation error occurs.
//!
//! ```
//! # use assert_json::assert_json;
//! # use assert_json::validators;
//! #
//! #[test]
//! fn test_json_ok() {
//!     let json = r#"
//!         {
//!             "status": "success",
//!             "result": {
//!                 "age": 26,
//!                 "name": "charlesvdv"
//!             }
//!         }
//!     "#;
//!
//!     let name = "charlesvdv";
//!
//!     assert_json!(json, {
//!             "status": "success",
//!             "result": {
//!                 "age": validators::u64(|&v| if v >= 18 { Ok(())} else { Err(String::from("age should be greater or equal than 18")) }),
//!                 "name": name,
//!             }
//!         }
//!     );
//! }
//! ```

use core::fmt;

/// A JSON-value. Used by the [Validator] trait.
pub type Value = serde_json::Value;

fn get_value_type_id(val: &Value) -> &'static str {
    match val {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Validation error
#[derive(Debug, PartialEq)]
pub enum Error<'a> {
    InvalidType(&'a Value, String),
    InvalidValue(&'a Value, String),
    MissingObjectKey(&'a Value, String),
    UnexpectedObjectKey(&'a Value, String),
    UnmatchedValidator(&'a Value, usize),
}

impl<'a> std::error::Error for Error<'a> {}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType(v, s) => write!(
                f,
                "Invalid type. Expected {} but got {}.",
                s,
                get_value_type_id(v)
            ),
            Self::InvalidValue(v, s) => write!(f, "Invalid value. Expected {s} but got {v}."),
            Self::MissingObjectKey(_v, s) => write!(f, "Missing key '{s}' in object"),
            Self::UnexpectedObjectKey(_v, s) => write!(f, "Key '{s}' is not expected in object"),
            Self::UnmatchedValidator(_v, s) => write!(f, "No match for expected array element {s}"),
        }
    }
}

impl<'a> Error<'a> {
    fn location(&self) -> &'a Value {
        match self {
            Error::InvalidType(loc, _)
            | Error::InvalidValue(loc, _)
            | Error::MissingObjectKey(loc, _)
            | Error::UnexpectedObjectKey(loc, _)
            | Error::UnmatchedValidator(loc, _) => loc,
        }
    }
}

/// Abstract the validation action for [`assert_json`!] macro.
///
/// Any custom validation rule can be easily use in the macro
/// by implementing the [`Validator::validate`] method.
///
/// ```
/// use assert_json::{assert_json, Error, Validator, Value};
///
/// fn optional_string(expected: Option<String>) -> impl Validator {
///     OptionalStringValidator { expected }
/// }
///
/// /// Matches a null JSON value if expected is None, else check if the strings
/// /// are equals
/// struct OptionalStringValidator {
///     expected: Option<String>,
/// }
///
/// impl Validator for OptionalStringValidator {
///     fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
///         if let Some(expected_str) = &self.expected {
///             let string_value = value
///                 .as_str()
///                 .ok_or_else(|| Error::InvalidType(value, String::from("string")))?;
///
///             if expected_str == string_value {
///                 Ok(())
///             } else {
///                 Err(Error::InvalidValue(value, expected_str.clone()))
///             }
///         } else {
///             value.as_null()
///                 .ok_or_else(|| Error::InvalidType(value, String::from("null")))
///         }
///     }
/// }
///
/// let json = r#"
///     {
///         "key": "value",
///         "none": null
///     }
/// "#;
/// assert_json!(json, {
///     "key": optional_string(Some(String::from("value"))),
///     "none": optional_string(None),
/// });
/// ```
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

/// Custom validators for different JSON types
pub mod validators;

#[macro_use]
mod macros;
#[doc(hidden)]
pub mod macros_utils;
