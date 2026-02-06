use std::{fmt::Debug, marker::PhantomData, ops};

use super::{Div, Mul, Unit};

pub struct Value<Q: Unit> {
    value: f32,
    _unit: PhantomData<Q>,
}

impl<Q: Unit> Value<Q> {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            _unit: PhantomData,
        }
    }

    pub fn convert<T: Unit>(&self) -> Value<T> {
        Value {
            value: T::from_base(Q::to_base(self.value)),
            _unit: PhantomData,
        }
    }

    pub fn raw(&self) -> f32 {
        self.value
    }

    pub fn raw_mut(&mut self) -> &mut f32 {
        &mut self.value
    }

    pub fn get<T: Unit>(&self) -> f32 {
        T::from_base(Q::to_base(self.value))
    }
}

impl<Q: Unit> Debug for Value<Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.value))
    }
}

impl<Q: Unit> Clone for Value<Q> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _unit: PhantomData,
        }
    }
}

impl<Q: Unit> Copy for Value<Q> {}

impl<Q: Unit> Default for Value<Q> {
    fn default() -> Self {
        Self {
            value: 0.0,
            _unit: PhantomData,
        }
    }
}

impl<Q: Unit> PartialEq for Value<Q> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<A: Unit, B: Unit> ops::Mul<Value<B>> for Value<A> {
    type Output = Value<Mul<A, B>>;

    fn mul(self, rhs: Value<B>) -> Self::Output {
        Value {
            value: self.value * rhs.value,
            _unit: PhantomData,
        }
    }
}

impl<A: Unit, B: Unit> ops::Div<Value<B>> for Value<A> {
    type Output = Value<Div<A, B>>;

    fn div(self, rhs: Value<B>) -> Self::Output {
        Value {
            value: self.value / rhs.value,
            _unit: PhantomData,
        }
    }
}

impl<Q: Unit> ops::Mul<f32> for Value<Q> {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.value *= rhs;
        self
    }
}

impl<Q: Unit> ops::Mul<Value<Q>> for f32 {
    type Output = Value<Q>;

    fn mul(self, rhs: Value<Q>) -> Self::Output {
        Value::new(rhs.value * self)
    }
}

impl<Q: Unit> ops::Div<f32> for Value<Q> {
    type Output = Self;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.value /= rhs;
        self
    }
}

impl<Q: Unit> ops::Div<Value<Q>> for f32 {
    type Output = Value<Q>;

    fn div(self, rhs: Value<Q>) -> Self::Output {
        Value::new(rhs.value / self)
    }
}

impl<Q: Unit> serde::Serialize for Value<Q> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(self.value)
    }
}

impl<'de, Q: Unit> serde::Deserialize<'de> for Value<Q> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct F32Visitor;

        impl<'de> serde::de::Visitor<'de> for F32Visitor {
            type Value = f32;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a float")
            }

            fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(value)
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(value as f32)
            }
        }

        let value = deserializer.deserialize_f32(F32Visitor)?;
        Ok(Value {
            value,
            _unit: PhantomData,
        })
    }
}
