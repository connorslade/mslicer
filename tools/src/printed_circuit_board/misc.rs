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
}
