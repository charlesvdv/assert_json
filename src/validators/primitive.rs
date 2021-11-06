use crate::{Error, Validator, Value};

pub fn string(predicate: Box<dyn Fn(&String) -> Result<(), String>>) -> Box<dyn Validator> {
    Box::new(PrimitiveValidator {
        typename: String::from("string"),
        extract: Box::new(|val: &Value| val.as_str().map(|v| String::from(v))),
        predicate,
    })
}

pub fn null() -> Box<dyn Validator> {
    Box::new(PrimitiveValidator {
        typename: String::from("null"),
        extract: Box::new(|val| val.as_null()),
        predicate: Box::new(|_| Ok(())),
    })
}

pub fn bool(predicate: Box<dyn Fn(&bool) -> Result<(), String>>) -> Box<dyn Validator> {
    Box::new(PrimitiveValidator {
        typename: String::from("bool"),
        extract: Box::new(|val| val.as_bool()),
        predicate,
    })
}

pub fn bool_true() -> Box<dyn Validator> {
    bool(Box::new(|val| {
        if *val {
            Ok(())
        } else {
            Err(String::from("value not true"))
        }
    }))
}

pub fn bool_false() -> Box<dyn Validator> {
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
        let val = (self.extract)(value).ok_or(Error::InvalidType(value, self.typename.clone()))?;

        (self.predicate)(&val).map_err(|msg| Error::InvalidValue(value, msg))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, Value};

    #[test]
    fn string() {
        let validator = super::string(Box::new(|_| Ok(())));

        assert_eq!(Ok(()), validator.validate(&Value::String("ok".to_string())));
    }

    #[test]
    fn string_invalid_value() {
        let validator = super::string(Box::new(move |_| Err(String::from("error message"))));

        assert!(matches!(
            validator.validate(&Value::String("".to_string())),
            Err(Error::InvalidValue(_, _))
        ));
    }

    #[test]
    fn string_invalid_type() {
        let validator = super::string(Box::new(|_| Ok(())));

        assert!(matches!(
            validator.validate(&Value::Null),
            Err(Error::InvalidType(_, _))
        ));
    }

    #[test]
    fn null() {
        let validator = super::null();

        assert_eq!(Ok(()), validator.validate(&Value::Null));
    }
}