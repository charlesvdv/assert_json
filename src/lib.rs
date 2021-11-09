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

    fn and(self, validator: Box<dyn Validator>) -> And<Self>
    where
        Self: Sized,
    {
        And {
            first: self,
            second: validator,
        }
    }
}

pub struct And<T> {
    first: T,
    second: Box<dyn Validator>,
}

impl<T> Validator for And<T>
where
    T: Validator,
{
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        self.first.validate(value).and(self.second.validate(value))
    }
}

pub mod validators;
#[macro_use]
pub mod macros;
