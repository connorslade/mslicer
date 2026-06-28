//! References:
//! - [The Unreasonable Effectiveness of Quasirandom Sequences](https://extremelearning.com.au/unreasonable-effectiveness-of-quasirandom-sequences/)
//! - [weyl sequence](https://www.shadertoy.com/view/4dtBWH)

use nalgebra::Vector2;

const PLASTIC_RATIO: f32 = 1.32471795724474602596;
const A1: f32 = PLASTIC_RATIO.recip();
const A2: f32 = (PLASTIC_RATIO * PLASTIC_RATIO).recip();

// Output range: [0,1)
pub fn quazirandom_2d(i: usize) -> Vector2<f32> {
    Vector2::new(A1, A2).map(|a| (0.5 + i as f32 * a).fract())
}

// Output range: ⟨[0, width), [0, height)⟩
pub fn quazirandom_rect_2d(size: Vector2<f32>, spacing: f32) -> impl Iterator<Item = Vector2<f32>> {
    let scale = size.x.max(size.y);
    let count = (scale.powi(2) / spacing) as usize;

    (0..count)
        .map(move |i| quazirandom_2d(i) * scale)
        .filter(move |p| p.x < size.x && p.y < size.y)
}
