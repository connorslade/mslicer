pub trait FloatExt {
    fn saturate(&self) -> Self;
}

impl FloatExt for f32 {
    fn saturate(&self) -> Self {
        self.clamp(0.0, 1.0)
    }
}

impl FloatExt for f64 {
    fn saturate(&self) -> Self {
        self.clamp(0.0, 1.0)
    }
}
