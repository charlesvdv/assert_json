type Value = serde_json::Value;

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

pub trait Validator {
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>>;
}

pub mod validators;
#[macro_use]
pub mod macros;
