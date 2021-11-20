//! A easy and declarative way to test JSON input in Rust.
//!
//! [assert_json!] is a Rust macro heavily inspired by [serde_json::json!] macro.
//! Instead of creating a JSON value from a JSON literal, [assert_json!] makes sure
//! the JSON input conforms to the validation rules specified.
//!
//! [assert_json!] also output beautiful error message when a validation error occurs.
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
//!                 "id": 5,
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
//!                 "id": validators::u64(|&v| if v > 0 { Ok(())} else { Err(String::from("id should be greater than 0")) }),
//!                 "name": name,
//!             }
//!         }
//!     );
//! }
//! ```

/// A JSON-value. Used by the [Validator] trait.
pub type Value = serde_json::Value;

fn get_value_type_id(val: &Value) -> String {
    match val {
        serde_json::Value::Null => String::from("null"),
        serde_json::Value::Bool(_) => String::from("bool"),
        serde_json::Value::Number(_) => String::from("number"),
        serde_json::Value::String(_) => String::from("string"),
        serde_json::Value::Array(_) => String::from("array"),
        serde_json::Value::Object(_) => String::from("object"),
    }
}

/// Validation error
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error<'a> {
    #[error("Invalid type. Expected {} but got {}.", .1, get_value_type_id(.0))]
    InvalidType(&'a Value, String),
    #[error("Invalid value. Expected {1} but got {0}.")]
    InvalidValue(&'a Value, String),
    #[error("Missing key '{1}' in object")]
    MissingObjectKey(&'a Value, String),
    #[error("Key '{1}' is not expected in object")]
    UnexpectedObjectKey(&'a Value, String),
}

impl<'a> Error<'a> {
    fn location(&self) -> &'a Value {
        match self {
            Error::InvalidType(loc, _) => loc,
            Error::InvalidValue(loc, _) => loc,
            Error::MissingObjectKey(loc, _) => loc,
            Error::UnexpectedObjectKey(loc, _) => loc,
        }
    }
}

/// Abstract the validation action for [assert_json!] macro.
///
/// Any custom validation rule can be easily use in the macro
/// by implementing the [Validator::validate] method.
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
