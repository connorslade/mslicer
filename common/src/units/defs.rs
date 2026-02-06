use crate::{marker_structs, units::LengthUnit};

use super::{MetricPrefix, TimeUnit, Unit};

marker_structs![
    Second<P: MetricPrefix = Base>,
    Minute,

    Meter<P: MetricPrefix = Base>
];

pub struct Kilo;
pub struct Base;
pub struct Centi;
pub struct Milli;
pub struct Micro;

impl<P: MetricPrefix> Unit for Meter<P> {
    fn to_base(val: f32) -> f32 {
        val * P::FACTOR
    }

    fn from_base(val: f32) -> f32 {
        val / P::FACTOR
    }
}

impl<P: MetricPrefix> LengthUnit for Meter<P> {}

impl<P: MetricPrefix> Unit for Second<P> {
    fn to_base(val: f32) -> f32 {
        val * P::FACTOR
    }

    fn from_base(val: f32) -> f32 {
        val / P::FACTOR
    }
}

impl<P: MetricPrefix> TimeUnit for Second<P> {}

impl Unit for Minute {
    fn to_base(val: f32) -> f32 {
        val * 60.0
    }

    fn from_base(val: f32) -> f32 {
        val / 60.0
    }
}

impl TimeUnit for Minute {}

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
