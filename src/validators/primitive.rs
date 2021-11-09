use crate::{Error, Validator, Value};

pub fn string<F>(predicate: F) -> impl Validator
where
    F: Fn(&String) -> Result<(), String> + 'static,
{
    PrimitiveValidator {
        typename: String::from("string"),
        extract: Box::new(|val: &Value| val.as_str().map(String::from)),
        predicate,
    }
}

pub fn null() -> impl Validator {
    PrimitiveValidator {
        typename: String::from("null"),
        extract: Box::new(|val| val.as_null()),
        predicate: |_| Ok(()),
    }
}

pub fn bool<F>(predicate: F) -> impl Validator
where
    F: Fn(&bool) -> Result<(), String> + 'static,
{
    PrimitiveValidator {
        typename: String::from("bool"),
        extract: Box::new(|val| val.as_bool()),
        predicate,
    }
}

pub fn i64<F>(predicate: F) -> impl Validator
where
    F: Fn(&i64) -> Result<(), String> + 'static,
{
    PrimitiveValidator {
        typename: String::from("i64"),
        extract: Box::new(|val| val.as_i64()),
        predicate,
    }
}

pub fn u64<F>(predicate: F) -> impl Validator
where
    F: Fn(&u64) -> Result<(), String> + 'static,
{
    PrimitiveValidator {
        typename: String::from("u64"),
        extract: Box::new(|val| val.as_u64()),
        predicate,
    }
}

pub fn f64<F>(predicate: F) -> impl Validator
where
    F: Fn(&f64) -> Result<(), String> + 'static,
{
    PrimitiveValidator {
        typename: String::from("f64"),
        extract: Box::new(|val| val.as_f64()),
        predicate,
    }
}

struct PrimitiveValidator<T, F>
where
    F: Fn(&T) -> Result<(), String>,
{
    typename: String,
    extract: Box<dyn Fn(&Value) -> Option<T>>,
    predicate: F,
}

impl<T, F> Validator for PrimitiveValidator<T, F>
where
    F: Fn(&T) -> Result<(), String>,
{
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        let val = (self.extract)(value)
            .ok_or_else(|| Error::InvalidType(value, self.typename.clone()))?;

        (self.predicate)(&val).map_err(|msg| Error::InvalidValue(value, msg))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, Validator, Value};

    #[test]
    fn string() {
        let validator = super::string(|_| Ok(()));

        assert_eq!(Ok(()), validator.validate(&Value::String("ok".to_string())));
    }

    #[test]
    fn string_invalid_value() {
        let validator = super::string(|_| Err(String::from("error message")));

        assert!(matches!(
            validator.validate(&Value::String("".to_string())),
            Err(Error::InvalidValue(_, _))
        ));
    }

    #[test]
    fn string_invalid_type() {
        let validator = super::string(|_| Ok(()));

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

    #[test]
    fn i64() {
        let validator = super::i64(|_| Ok(()));

        assert_eq!(Ok(()), validator.validate(&serde_json::json!(4)));
    }
}
