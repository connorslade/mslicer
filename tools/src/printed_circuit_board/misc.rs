use nalgebra::Vector2;

use crate::misc::bounds::Bounds2D;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,

    CenterLeft,
    Center,
    CenterRight,

    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Alignment {
    pub const ALL: [Self; 9] = [
        Self::TopLeft,
        Self::TopCenter,
        Self::TopRight,
        Self::CenterLeft,
        Self::Center,
        Self::CenterRight,
        Self::BottomLeft,
        Self::BottomCenter,
        Self::BottomRight,
    ];

    pub fn name(&self) -> &str {
        match self {
            Self::TopLeft => "Top Left",
            Self::TopCenter => "Top Center",
            Self::TopRight => "Top Right",
            Self::CenterLeft => "Center Left",
            Self::Center => "Center",
            Self::CenterRight => "Center Right",
            Self::BottomLeft => "Bottom Left",
            Self::BottomCenter => "Bottom Center",
            Self::BottomRight => "Bottom Right",
        }
    }

    pub fn offset(&self, size: Vector2<f64>, bounds: Bounds2D<f64>) -> Vector2<f64> {
        let [min, max] = [bounds.min, bounds.max];
        let center = (size - min - max) / 2.0;
        match self {
            Alignment::TopLeft => Vector2::new(-min.x, -min.y),
            Alignment::TopCenter => Vector2::new(center.x, -min.y),
            Alignment::TopRight => Vector2::new(size.x - max.x, -min.y),

            Alignment::CenterLeft => Vector2::new(-min.x, center.y),
            Alignment::Center => center,
            Alignment::CenterRight => Vector2::new(size.x - max.x, center.y),

            Alignment::BottomLeft => Vector2::new(-min.x, size.y - max.y),
            Alignment::BottomCenter => Vector2::new(center.x, size.y - max.y),
            Alignment::BottomRight => Vector2::new(size.x - max.x, size.y - max.y),
        }
    }
}
