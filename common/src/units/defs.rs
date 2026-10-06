use std::fmt;

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
    const FACTOR: f64 = P::FACTOR;

    fn write_unit(fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(P::UNIT)?;
        fmt.write_str("m")?;
        Ok(())
    }
}

impl<P: MetricPrefix> TimeUnit for Second<P> {}
impl<P: MetricPrefix> Unit for Second<P> {
    const FACTOR: f64 = P::FACTOR;

    fn write_unit(fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(P::UNIT)?;
        fmt.write_str("s")?;
        Ok(())
    }
}

impl TimeUnit for Minute {}
impl Unit for Minute {
    const FACTOR: f64 = 60.0;

    fn write_unit(fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("min")
    }
}

pub struct Kilo;
pub struct Base;
pub struct Centi;
pub struct Milli;
pub struct Micro;

impl MetricPrefix for Kilo {
    const FACTOR: f64 = 1e3;
    const UNIT: &str = "k";
}

impl MetricPrefix for Base {
    const FACTOR: f64 = 1e0;
    const UNIT: &str = "";
}

impl MetricPrefix for Centi {
    const FACTOR: f64 = 1e-2;
    const UNIT: &str = "c";
}

impl MetricPrefix for Milli {
    const FACTOR: f64 = 1e-3;
    const UNIT: &str = "m";
}

impl MetricPrefix for Micro {
    const FACTOR: f64 = 1e-6;
    const UNIT: &str = "μ";
}
