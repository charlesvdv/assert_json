use crate::validators;
use crate::{Error, Validator, Value};

pub fn array(array_validators: Vec<Box<dyn Validator>>) -> impl Validator {
    ArrayValidator {
        validators: array_validators,
    }
}

pub fn array_size(expected_size: usize) -> impl Validator {
    ArrayValidator {
        validators: (0..expected_size)
            .map(|_| Box::new(validators::any()) as Box<dyn Validator>)
            .collect(),
    }
}

pub fn array_empty() -> impl Validator {
    ArrayValidator { validators: vec![] }
}

struct ArrayValidator {
    validators: Vec<Box<dyn Validator>>,
}

impl Validator for ArrayValidator {
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        let value_vec = value
            .as_array()
            .ok_or_else(|| Error::InvalidType(value, String::from("array")))?;

        if value_vec.len() != self.validators.len() {
            return Err(Error::InvalidValue(
                value,
                format!(
                    "expected {} elements got {}",
                    self.validators.len(),
                    value_vec.len()
                ),
            ));
        }

        value_vec
            .iter()
            .zip(self.validators.iter())
            .try_for_each(|(val, validator)| validator.validate(val))
    }
}

pub fn array_for_each(validator: impl Validator) -> impl Validator {
    ArrayForEachValidator { validator }
}

struct ArrayForEachValidator<T>
where
    T: Validator,
{
    validator: T,
}

impl<T> Validator for ArrayForEachValidator<T>
where
    T: Validator,
{
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        let value_vec = value
            .as_array()
            .ok_or_else(|| Error::InvalidType(value, String::from("array")))?;

        value_vec
            .iter()
            .try_for_each(|val| self.validator.validate(val))
    }
}

#[cfg(test)]
mod tests {
    use crate::validators;
    use crate::{Error, Validator};

    #[test]
    fn non_array() {
        let validator = super::array(vec![]);

        assert!(matches!(
            validator.validate(&serde_json::json!(null)),
            Err(Error::InvalidType(_, _))
        ));
    }

    #[test]
    fn empty() {
        let validator = super::array(vec![]);

        assert_eq!(Ok(()), validator.validate(&serde_json::json!([])));
    }

    #[test]
    fn with_different_value() {
        let validator = super::array(vec![
            Box::new(validators::eq(5)),
            Box::new(validators::null()),
        ]);

        assert_eq!(Ok(()), validator.validate(&serde_json::json!([5, null,])))
    }

    #[test]
    fn different_size() {
        let validator = super::array(vec![]);

        assert!(matches!(
            validator.validate(&serde_json::json!([null])),
            Err(Error::InvalidValue(_, _))
        ));
    }

    #[test]
    fn non_matching_array_value() {
        let validator = super::array(vec![Box::new(validators::null())]);

        assert!(matches!(
            validator.validate(&serde_json::json!([5])),
            Err(Error::InvalidType(_, _))
        ));
    }

    #[test]
    fn array_size() {
        let validator = super::array_size(3);

        assert_eq!(
            Ok(()),
            validator.validate(&serde_json::json!([4, "test", 3.4]))
        );
    }

    #[test]
    fn for_each() {
        let validator = super::array_for_each(validators::eq(String::from("test")));

        assert_eq!(
            Ok(()),
            validator.validate(&serde_json::json!(["test", "test", "test"]))
        );
    }
}
