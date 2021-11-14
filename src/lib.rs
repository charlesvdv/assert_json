pub type Value = serde_json::Value;

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

pub mod validators;
#[macro_use]
pub mod macros;
#[doc(hidden)]
pub mod macros_utils;
