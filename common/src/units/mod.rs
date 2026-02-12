mod defs;
mod value;
pub use defs::{Base, Centi, Kilo, Meter, Micro, Milli, Second};
pub use value::{Length, Time, Velocity};

use crate::units::{
    defs::Minute,
    value::{Area, Volume},
};

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

pub type SquareMilimeters = Area<Milimeter>;
pub type CubicMilimeters = Volume<Milimeter>;
pub type CubicCentimeters = Volume<Centimeter>;
pub type Milliliters = CubicCentimeters;

pub type Milisecond = Second<Milli>;
pub type Miliseconds = Time<Milisecond>;
pub type Seconds = Time<Second>;
pub type Minutes = Time<Minute>;

pub type CentimetersPerSecond = Velocity<Centimeter, Second>;
pub type MilimetersPerMinute = Velocity<Milimeter, Minute>;

pub trait Unit {
    const FACTOR: f32;

    fn apply(val: f32, power: i32) -> f32 {
        val * Self::FACTOR.powi(power)
    }
}

pub trait LengthUnit: Unit {}
pub trait TimeUnit: Unit {}

pub trait MetricPrefix {
    const FACTOR: f32;
}
