use nalgebra::Vector3;

use crate::render::{
    dispatch::line::LineGenerator, pipelines::line::Line, workspace::WorkspaceRenderCallback,
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
    fn generate_lines(&mut self, resources: &WorkspaceRenderCallback) -> bool {
        self.cached_lines = resources
            .line_support_debug
            .iter()
            .map(|[vertex, normal]| {
                Line::new(*vertex, *vertex + normal * 0.2).color(Vector3::new(1.0, 0.0, 0.0))
            })
            .collect();
        true
    }

    fn lines(&self) -> &[Line] {
        &self.cached_lines
    }
}
