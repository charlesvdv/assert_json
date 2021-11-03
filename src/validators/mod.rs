use crate::{Error, Validator, Value};

mod primitive;
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

#[cfg(test)]
mod tests {
    use crate::{Validator, Value};

    #[test]
    fn any() {
        let validator = super::any();

        assert_eq!(Ok(()), validator.validate(&Value::Null));
    }
}
