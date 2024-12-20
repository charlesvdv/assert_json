use std::collections::HashMap;

use crate::{Error, Validator, Value};

/// Match if each key/value pair matches
///
/// Ignore key that are not specified. Use [`object_strict`] if you want to
/// exactly match all the key/values.
#[must_use]
pub fn object(key_validators: HashMap<String, Box<dyn Validator>>) -> impl Validator {
    ObjectValidator {
        key_validators,
        strict: false,
    }
}

/// Match if each key/value pairs matches. Fail if a key is missing in the validators.
#[must_use]
pub fn object_strict(key_validators: HashMap<String, Box<dyn Validator>>) -> impl Validator {
    ObjectValidator {
        key_validators,
        strict: true,
    }
}

/// Match if the object is empty.
#[must_use]
pub fn object_empty() -> impl Validator {
    ObjectValidator {
        key_validators: HashMap::new(),
        strict: true,
    }
}

struct ObjectValidator {
    key_validators: HashMap<String, Box<dyn Validator>>,
    strict: bool,
}

impl Validator for ObjectValidator {
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        let object = value
            .as_object()
            .ok_or_else(|| Error::InvalidType(value, String::from("object")))?;

        for (key, validator) in &self.key_validators {
            let inner_value = object
                .get(key)
                .ok_or_else(|| Error::MissingObjectKey(value, key.clone()))?;

            validator.validate(inner_value)?;
        }

        if self.strict {
            // Make sure there is no other keys than the one defined in the validator
            // if we are in strict mode.
            for (key, value) in object {
                self.key_validators
                    .get(key)
                    .ok_or_else(|| Error::UnexpectedObjectKey(value, key.clone()))
                    .map(|_| ())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{validators, Error, Validator};

    #[test]
    fn valid() {
        let mut key_validators: HashMap<String, Box<dyn Validator>> = HashMap::new();
        key_validators.insert(
            String::from("key"),
            Box::new(validators::string(|_| Ok(()))),
        );
        key_validators.insert(String::from("key1"), Box::new(validators::any()));

        let validator = super::object(key_validators);
        assert_eq!(
            Ok(()),
            validator.validate(&serde_json::json!({"key": "val", "key1": null}))
        );
    }

    #[test]
    fn missing_key() {
        let mut key_validators: HashMap<String, Box<dyn Validator>> = HashMap::new();
        key_validators.insert(
            String::from("key"),
            Box::new(validators::string(|_| Ok(()))),
        );

        let validator = super::object(key_validators);
        assert!(matches!(
            validator.validate(&serde_json::json!({})),
            Err(Error::MissingObjectKey(_, _))
        ));
    }
}
