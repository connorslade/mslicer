#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

impl Alignment {
    pub const ALL: [Self; 5] = [
        Self::TopLeft,
        Self::TopRight,
        Self::BottomLeft,
        Self::BottomRight,
        Self::Center,
    ];

    pub fn name(&self) -> &str {
        match self {
            Alignment::TopLeft => "Top Left",
            Alignment::TopRight => "Top Right",
            Alignment::BottomLeft => "Bottom Left",
            Alignment::BottomRight => "Bottom Right",
            Alignment::Center => "Center",
        }
    }
}
