mod defs;
mod value;
pub use defs::{Base, Centi, Kilo, Meter, Micro, Milli, Second};
pub use value::{Length, Time, Velocity};

use crate::units::defs::Minute;

// gotta love macros. this is just so readable.
#[macro_export]
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

pub type Mircometer = Meter<Micro>;
pub type Milimeter = Meter<Milli>;
pub type Centimeter = Meter<Centi>;
pub type Meters = Length<Meter>;
pub type Micrometers = Length<Mircometer>;
pub type Milimeters = Length<Milimeter>;
pub type Centimeters = Length<Centimeter>;

pub type Seconds = Time<Second>;
pub type Minutes = Time<Minute>;

pub type CentimetersPerSecond = Velocity<Centimeter, Second>;
pub type MilimetersPerMinute = Velocity<Milimeter, Minute>;

pub trait Unit {
    fn to_base(val: f32) -> f32;
    fn from_base(val: f32) -> f32;
}

pub trait LengthUnit: Unit {}
pub trait TimeUnit: Unit {}

pub trait MetricPrefix {
    const FACTOR: f32;
}
