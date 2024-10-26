use std::collections::HashSet;

use crate::{validators, Error, Validator, Value};

/// Match each array element to a specific validator.
pub fn array(array_validators: Vec<Box<dyn Validator>>) -> impl Validator {
    ArrayValidator {
        validators: array_validators,
    }
}

/// Match the array size.
pub fn array_size(expected_size: usize) -> impl Validator {
    ArrayValidator {
        validators: (0..expected_size)
            .map(|_| Box::new(validators::any()) as Box<dyn Validator>)
            .collect(),
    }
}

/// Match empty array.
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

/// Each supplied validator matches a different array element, in any order.
pub fn array_contains(validators: Vec<Box<dyn Validator>>) -> impl Validator {
    UnorderedArrayValidator { validators }
}

struct UnorderedArrayValidator {
    validators: Vec<Box<dyn Validator>>,
}

impl Validator for UnorderedArrayValidator {
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        let value_vec = value
            .as_array()
            .ok_or_else(|| Error::InvalidType(value, String::from("array")))?;
        let mut matched_values: HashSet<usize> = HashSet::new();
        for (m, validator) in self.validators.iter().enumerate() {
            if let Some((n, _)) = value_vec
                .iter()
                .enumerate()
                .filter(|(n, _)| !matched_values.contains(n))
                .find(|(_, v)| validator.validate(v).is_ok())
            {
                matched_values.insert(n);
            } else {
                return Err(Error::UnmatchedValidator(value, m));
            }
        }
        Ok(())
    }
}

/// Match if each element match the validator
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
    use crate::{validators, Error, Validator};

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
    fn array_contains() {
        let validator = validators::array_contains(vec![
            Box::new(validators::eq(1)),
            Box::new(validators::eq(2)),
        ]);

        assert_eq!(Ok(()), validator.validate(&serde_json::json!([3, 2, 1])));
    }

    #[test]
    fn array_contains_repetition() {
        let validator = validators::array_contains(vec![
            Box::new(validators::eq(1)),
            Box::new(validators::eq(1)),
        ]);

        assert!(matches!(
            validator.validate(&serde_json::json!([3, 1])),
            Err(Error::UnmatchedValidator(_, _)),
        ));
    }

    #[test]
    fn array_does_not_contain() {
        let validator = validators::array_contains(vec![
            Box::new(validators::eq(1)),
            Box::new(validators::eq(2)),
        ]);

        assert!(matches!(
            validator.validate(&serde_json::json!([3, 1])),
            Err(Error::UnmatchedValidator(_, _)),
        ));
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
