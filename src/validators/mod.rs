use crate::{Error, Validator, Value};
use std::fmt::Debug;

mod object;
mod primitive;

pub use object::*;
pub use primitive::*;

pub fn any() -> impl Validator {
    AnyValidator {}
}

struct AnyValidator {}

impl Validator for AnyValidator {
    fn validate<'a>(&self, _: &'a Value) -> Result<(), Error<'a>> {
        Ok(())
    }
}

pub fn eq<T>(expected: T) -> impl Validator
where
    T: Into<Value> + Clone + Debug + 'static,
{
    EqValidator { expected }
}

struct EqValidator<T>
where
    T: Into<Value> + Clone + Debug,
{
    expected: T,
}

impl<T> Validator for EqValidator<T>
where
    T: Into<Value> + Clone + Debug,
{
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        if value == &self.expected.clone().into() {
            Ok(())
        } else {
            Err(Error::InvalidValue(value, format!("{:?}", self.expected)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, Validator, Value};

    #[test]
    fn any() {
        let validator = super::any();

        assert_eq!(Ok(()), validator.validate(&Value::Null));
    }

    #[test]
    fn eq_string() {
        let validator = super::eq("test");

        assert_eq!(Ok(()), validator.validate(&serde_json::json!("test")))
    }

    #[test]
    fn eq_string_fail() {
        let validator = super::eq(String::from("test"));

        assert!(matches!(
            validator.validate(&serde_json::json!("not expected")),
            Err(Error::InvalidValue(_, _))
        ));
    }
}
