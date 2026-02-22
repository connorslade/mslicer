use crate::units::LengthUnit;

use super::{MetricPrefix, TimeUnit, Unit};

// gotta love macros. this is just so readable.
macro_rules! marker_structs {
    ($($name:ident$(<$($param:ident$(:$constraint:ident)?$(=$default:ident)?),+>)?),*) => {
        $(pub struct $name$(<$($param$(:$constraint)?$(=$default)?),+>)? {
            $(
                #[allow(unused_parens)]
                _types: std::marker::PhantomData<($($param),+)>
            )?
        })*
    };
}

marker_structs![
    Second<P: MetricPrefix = Base>,
    Minute,

    Meter<P: MetricPrefix = Base>
];

impl<P: MetricPrefix> LengthUnit for Meter<P> {}
impl<P: MetricPrefix> Unit for Meter<P> {
    const FACTOR: f32 = P::FACTOR;
}

impl<P: MetricPrefix> TimeUnit for Second<P> {}
impl<P: MetricPrefix> Unit for Second<P> {
    const FACTOR: f32 = P::FACTOR;
}

impl TimeUnit for Minute {}
impl Unit for Minute {
    const FACTOR: f32 = 60.0;
}

pub struct Kilo;
pub struct Base;
pub struct Centi;
pub struct Milli;
pub struct Micro;

impl MetricPrefix for Kilo {
    const FACTOR: f32 = 1e3;
}

impl MetricPrefix for Base {
    const FACTOR: f32 = 1e0;
}

impl MetricPrefix for Centi {
    const FACTOR: f32 = 1e-2;
}

impl MetricPrefix for Milli {
    const FACTOR: f32 = 1e-3;
}

impl MetricPrefix for Micro {
    const FACTOR: f32 = 1e-6;
}
