use crate::marker_structs;

use super::{MetricPrefix, Unit};

marker_structs![
    Time<U: Unit = Second>,
    Second<P: MetricPrefix = Base>,
    Minute<P: MetricPrefix = Base>,
    Hour,

    Length<U: Unit = Meter>,
    Meter<P: MetricPrefix = Base>,
    Mile
];

pub struct Kilo;
pub struct Base;
pub struct Centi;
pub struct Milli;
pub struct Micro;

impl<U: Unit> Unit for Length<U> {
    fn to_base(val: f32) -> f32 {
        U::to_base(val)
    }

    fn from_base(val: f32) -> f32 {
        U::from_base(val)
    }
}

impl<U: Unit> Unit for Time<U> {
    fn to_base(val: f32) -> f32 {
        U::to_base(val)
    }

    fn from_base(val: f32) -> f32 {
        U::from_base(val)
    }
}

impl<P: MetricPrefix> Unit for Meter<P> {
    fn to_base(val: f32) -> f32 {
        P::to_base(val)
    }

    fn from_base(val: f32) -> f32 {
        P::from_base(val)
    }
}

impl<P: MetricPrefix> Unit for Second<P> {
    fn to_base(val: f32) -> f32 {
        P::to_base(val)
    }

    fn from_base(val: f32) -> f32 {
        P::from_base(val)
    }
}

impl Unit for Minute {
    fn to_base(val: f32) -> f32 {
        val * 60.0
    }

    fn from_base(val: f32) -> f32 {
        val / 60.0
    }
}

impl Unit for Hour {
    fn to_base(val: f32) -> f32 {
        val * (60.0 * 60.0)
    }

    fn from_base(val: f32) -> f32 {
        val / (60.0 * 60.0)
    }
}

impl Unit for Mile {
    fn to_base(val: f32) -> f32 {
        val * 1609.34
    }

    fn from_base(val: f32) -> f32 {
        val / 1609.34
    }
}

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
