use std::{marker::PhantomData, ops};

use crate::units::{Meter, TimeUnit, defs::Second};

use super::LengthUnit;

impl<L: LengthUnit> Length<L> {
    pub fn get<T: LengthUnit>(&self) -> f32 {
        T::from_base(L::to_base(self.value))
    }

    pub fn convert<T: LengthUnit>(&self) -> Length<T> {
        Length::new(self.get::<T>())
    }
}

impl<L: TimeUnit> Time<L> {
    pub fn get<T: TimeUnit>(&self) -> f32 {
        T::from_base(L::to_base(self.value))
    }

    pub fn convert<T: TimeUnit>(&self) -> Time<T> {
        Time::new(self.get::<T>())
    }
}

impl<L: LengthUnit, T: TimeUnit> Velocity<L, T> {
    pub fn get<L2: LengthUnit, T2: TimeUnit>(&self) -> f32 {
        T2::to_base(L2::from_base(T::from_base(L::to_base(self.value))))
    }

    pub fn convert<L2: LengthUnit, T2: TimeUnit>(&self) -> Velocity<L2, T2> {
        Velocity::new(self.get::<L2, T2>())
    }
}

impl<T1: TimeUnit, T2: TimeUnit> ops::Add<Time<T2>> for Time<T1> {
    type Output = Time<T1>;

    fn add(self, rhs: Time<T2>) -> Self::Output {
        Time::new(self.value + rhs.get::<T1>())
    }
}

impl<A: LengthUnit, B: LengthUnit> ops::Div<Length<B>> for Length<A> {
    type Output = f32;

    fn div(self, rhs: Length<B>) -> Self::Output {
        A::to_base(self.value) / B::to_base(rhs.value)
    }
}

impl<L: LengthUnit, T: TimeUnit> ops::Div<Time<T>> for Length<L> {
    type Output = Velocity<L, T>;

    fn div(self, rhs: Time<T>) -> Self::Output {
        Velocity::new(self.value / rhs.value)
    }
}

impl<L1: LengthUnit, L2: LengthUnit, T2: TimeUnit> ops::Div<Velocity<L2, T2>> for Length<L1> {
    type Output = Time<T2>;

    fn div(self, rhs: Velocity<L2, T2>) -> Self::Output {
        Time::new(L1::to_base(self.value) / L2::to_base(rhs.value))
    }
}

macro_rules! quantity {
    ($($name:ident<$($param:ident: $constraint:ident $(= $default:ident)?),+>),*) => {
        $(
            pub struct $name<$($param: $constraint $(= $default)?),+> {
                value: f32,
                #[allow(unused_parens)]
                _unit: PhantomData<($($param),+)>,
            }

            impl<$($param: $constraint),+> $name<$($param),+> {
                pub fn new(value: f32) -> Self {
                    Self {
                        value,
                        _unit: PhantomData,
                    }
                }

                pub fn raw(&self) -> f32 {
                    self.value
                }

                pub fn raw_mut(&mut self) -> &mut f32 {
                    &mut self.value
                }
            }

            impl<$($param: $constraint),+> std::fmt::Debug for $name<$($param),+> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_fmt(format_args!("{:?}", self.value))
                }
            }

            impl<$($param: $constraint),+> Default for $name<$($param),+> {
                fn default() -> Self {
                    Self::new(0.0)
                }
            }

            impl<$($param: $constraint),+> Clone for $name<$($param),+> {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl<$($param: $constraint),+> Copy for $name<$($param),+> {}

            impl<$($param: $constraint),+> PartialEq for $name<$($param),+> {
                fn eq(&self, other: &Self) -> bool {
                    self.value == other.value
                }
            }

            impl<$($param: $constraint),+> ops::Mul<f32> for $name<$($param),+> {
                type Output = Self;

                fn mul(mut self, rhs: f32) -> Self::Output {
                    self.value *= rhs;
                    self
                }
            }

            impl<$($param: $constraint),+> ops::Mul<$name<$($param),+>> for f32 {
                type Output = $name<$($param),+>;

                fn mul(self, rhs: $name<$($param),+>) -> Self::Output {
                    $name {
                        value: rhs.value * self,
                        _unit: PhantomData,
                    }
                }
            }

            impl<$($param: $constraint),+> ops::Div<f32> for $name<$($param),+> {
                type Output = Self;

                fn div(mut self, rhs: f32) -> Self::Output {
                    self.value /= rhs;
                    self
                }
            }

            impl<$($param: $constraint),+> ops::Div<$name<$($param),+>> for f32 {
                type Output = $name<$($param),+>;

                fn div(self, rhs: $name<$($param),+>) -> Self::Output {
                    $name {
                        value: self / rhs.value,
                        _unit: PhantomData,
                    }
                }
            }

            impl<$($param: $constraint),+> serde::Serialize for $name<$($param),+> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_f32(self.value)
                }
            }

            impl<'de, $($param: $constraint),+> serde::Deserialize<'de> for $name<$($param),+> {
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
                    Ok($name {
                        value,
                        _unit: PhantomData,
                    })
                }
            }

        )*
    };
}

quantity! [
    Length<U: LengthUnit = Meter>,
    Time<U: TimeUnit = Second>,
    Velocity<L: LengthUnit, T: TimeUnit>
];
