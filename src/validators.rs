use crate::{Validator, Value, Error};

pub fn any() -> impl Validator {
    AnyValidator{}
}

struct AnyValidator {}

impl Validator for AnyValidator {
    fn validate<'a>(&self, _: &'a Value) -> Result<(), Error<'a>> {
        Ok(())
    }
}

pub fn string(predicate: Box<dyn Fn(&String) -> Result<(), String>>) -> impl Validator {
    PrimitiveValidator {
        typename: String::from("string"),
        extract: Box::new(|val: &Value| val.as_str().map(|v| String::from(v))),
        predicate,
    }
}

pub fn null() -> impl Validator {
    PrimitiveValidator {
        typename: String::from("null"),
        extract: Box::new(|val| val.as_null()),
        predicate: Box::new(|_| Ok(()))
    }
}

pub fn bool(predicate: Box<dyn Fn(&bool) -> Result<(), String>>) -> impl Validator {
    PrimitiveValidator {
        typename: String::from("bool"),
        extract: Box::new(|val| val.as_bool()),
        predicate,
    }
}

pub fn bool_true() -> impl Validator {
    bool(Box::new(|val| {
        if *val {
            Ok(())
        } else {
            Err(String::from("value not true"))
        }
    }))
}

pub fn bool_false() -> impl Validator {
    bool(Box::new(|val| {
        if !*val {
            Ok(())
        } else {
            Err(String::from("value not false"))
        }
    }))
}

struct PrimitiveValidator<T> {
    typename: String,
    extract: Box<dyn Fn(&Value) -> Option<T>>,
    predicate: Box<dyn Fn(&T) -> Result<(), String>>,
}

impl<T> Validator for PrimitiveValidator<T> {
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        let val = (self.extract)(value)
            .ok_or(Error::InvalidType(value, self.typename.clone()))?;

        (self.predicate)(&val)
            .map_err(|msg| Error::InvalidValue(value, msg))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Validator, Value, Error};

    #[test]
    fn any() {
        let validator = super::any();

        assert_eq!(Ok(()), validator.validate(&Value::Null));
    }

    #[test]
    fn string() {
        let validator = super::string(Box::new(|_| Ok(())));

        assert_eq!(Ok(()), validator.validate(&Value::String("ok".to_string())));
    }

    #[test]
    fn string_invalid_value() {
        let _error_msg = String::from("error message");
        let validator = super::string(Box::new(move |_| Err(_error_msg.clone())));

        assert!(matches!(validator.validate(&Value::String("".to_string())), Err(Error::InvalidValue(_, _error_msg))));

    }

    #[test]
    fn string_invalid_type() {
        let validator = super::string(Box::new(|_| Ok(())));

        let _expected_type = String::from("string");
        assert!(matches!(validator.validate(&Value::Null), Err(Error::InvalidType(_, _expected_type))));
    }

    #[test]
    fn null() {
        let validator = super::null();

        assert_eq!(Ok(()), validator.validate(&Value::Null));
    }
}