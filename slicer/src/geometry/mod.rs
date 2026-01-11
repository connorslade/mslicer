use nalgebra::Vector3;

pub mod segments_1d;
pub use segments_1d::Segments1D;
pub mod bvh;
pub mod triangle;

pub struct Hit {
    pub position: Vector3<f32>,
    pub t: f32,
    pub face: usize,
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

pub trait Primitive {
    const MIN_T: Option<f32>;
    const MAX_T: Option<f32>;

    #[inline]
    fn in_range(val: f32) -> bool {
        val >= Self::min_t() && val <= Self::max_t()
    }

    #[inline]
    fn min_t() -> f32 {
        Self::MIN_T.unwrap_or(f32::MIN)
    }

    #[inline]
    fn max_t() -> f32 {
        Self::MAX_T.unwrap_or(f32::MAX)
    }
}

pub mod primitive {
    use crate::geometry::Primitive;

    pub struct Ray;
    pub struct Segment;
    pub struct Line;

    impl Primitive for Ray {
        const MIN_T: Option<f32> = Some(0.0);
        const MAX_T: Option<f32> = None;
    }

    impl Primitive for Segment {
        const MIN_T: Option<f32> = Some(-1.0);
        const MAX_T: Option<f32> = Some(1.0);
    }

    impl Primitive for Line {
        const MIN_T: Option<f32> = None;
        const MAX_T: Option<f32> = None;
    }
}

impl Default for Hit {
    fn default() -> Self {
        Self {
            position: Vector3::repeat(f32::NAN),
            t: f32::MAX,
            face: usize::MAX,
        }
    }
}
