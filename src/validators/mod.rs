use crate::{Error, Validator, Value};

mod object;
mod primitive;

pub use object::*;
pub use primitive::*;

pub fn any() -> Box<dyn Validator> {
    Box::new(AnyValidator {})
}

struct AnyValidator {}

impl Validator for AnyValidator {
    fn validate<'a>(&self, _: &'a Value) -> Result<(), Error<'a>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Value;

    #[test]
    fn any() {
        let validator = super::any();

        assert_eq!(Ok(()), validator.validate(&Value::Null));
    }
}
