use std::f64::consts::PI;

pub mod bounds;

/// Reference: https://en.wikipedia.org/wiki/Sagitta_(geometry)
///
/// ```plain
/// self.sagitta = r (1 - cos θ/2)
/// 2 cos⁻¹(-(self.sagitta / r - 1)) = θ
/// n = π / cos⁻¹(1 - self.sagitta / r)
/// ```
pub fn circle_points(sagitta: f64, r: f64) -> u32 {
    ((PI / (1.0 - sagitta / r).acos()).ceil() as u32).max(3)
}
