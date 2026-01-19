use nalgebra::Vector3;

use crate::{
    app::App,
    render::workspace::line::{Line, LineGenerator},
};

pub struct LineSupportDebugDispatch {
    cached_lines: Vec<Line>,
}

impl LineSupportDebugDispatch {
    pub fn new() -> Self {
        Self {
            cached_lines: Vec::new(),
        }
    }
}

impl LineGenerator for LineSupportDebugDispatch {
    fn generate_lines(&mut self, app: &mut App) {
        self.cached_lines = (app.state.line_support_debug.iter())
            .map(|[vertex, normal]| {
                Line::new(*vertex, *vertex + normal).color(Vector3::new(1.0, 0.0, 0.0))
            })
            .collect();
    }

    fn lines(&self) -> &[Line] {
        &self.cached_lines
    }
}
