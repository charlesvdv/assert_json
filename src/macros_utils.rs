use crate::validators;
use crate::{Validator, Value};

pub struct Input(Value);

impl Input {
    pub fn get(self) -> Value {
        self.0
    }
}

impl From<&str> for Input {
    fn from(str_input: &str) -> Input {
        let value = serde_json::from_str(str_input).expect("failed to parse JSON");
        Input(value)
    }
}

impl From<Value> for Input {
    fn from(value: Value) -> Input {
        Input(value)
    }
}

pub struct ValidatorInput(Box<dyn Validator>);

impl ValidatorInput {
    pub fn get(self) -> Box<dyn Validator> {
        self.0
    }
}

#[doc(hidden)]
macro_rules! impl_from_validator_input_default {
    (
        $($ty:ty),*
    ) => {
        $(
            impl From<$ty> for ValidatorInput {
                #[inline]
                fn from(u: $ty) -> Self {
                    ValidatorInput(Box::new(validators::eq(u)))
                }
            }
        )*
    };
}

impl_from_validator_input_default!(
    String, bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64
);

impl From<&str> for ValidatorInput {
    fn from(str_input: &str) -> Self {
        ValidatorInput(Box::new(validators::eq(String::from(str_input))))
    }
}

impl<T> From<T> for ValidatorInput
where
    T: Validator + 'static,
{
    fn from(validator: T) -> Self {
        ValidatorInput(Box::new(validator))
    }
}
