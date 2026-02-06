mod defs;
mod value;
pub use defs::{Base, Centi, Kilo, Micro, Milli};
pub use value::Value;

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

pub type Meter<P = Base> = defs::Length<defs::Meter<P>>;
pub type Mircometer = Meter<Micro>;
pub type Milimeter = Meter<Milli>;
pub type Micrometers = Value<Mircometer>;
pub type Milimeters = Value<Milimeter>;

marker_structs![
    Div<T: Unit, K: Unit>,
    Mul<T: Unit, K: Unit>
];

pub trait Unit {
    fn to_base(val: f32) -> f32;
    fn from_base(val: f32) -> f32;
}

pub trait MetricPrefix {
    const FACTOR: f32;

    fn to_base(val: f32) -> f32 {
        val * Self::FACTOR
    }

    fn from_base(val: f32) -> f32 {
        val / Self::FACTOR
    }
}

impl<A: Unit, B: Unit> Unit for Div<A, B> {
    fn to_base(val: f32) -> f32 {
        B::from_base(A::to_base(val))
    }

    fn from_base(val: f32) -> f32 {
        A::from_base(B::to_base(val))
    }
}

impl<A: Unit, B: Unit> Unit for Mul<A, B> {
    fn to_base(val: f32) -> f32 {
        B::to_base(A::to_base(val))
    }

    fn from_base(val: f32) -> f32 {
        A::from_base(B::from_base(val))
    }
}
