use build_plate::BuildPlateDispatch;
use line_support_debug::LineSupportDebugDispatch;
use normals::NormalsDispatch;
use wgpu::{Device, Queue, RenderPass, TextureFormat};

use crate::render::{
    pipelines::solid_line::{Line, SolidLinePipeline},
    workspace::WorkspaceRenderCallback,
};

mod build_plate;
mod line_support_debug;
mod normals;

pub struct SolidLineDispatch {
    render_pipeline: SolidLinePipeline,

    build_plate: BuildPlateDispatch,
    normals: NormalsDispatch,
    line_support_debug: LineSupportDebugDispatch,
}

impl SolidLineDispatch {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        Self {
            render_pipeline: SolidLinePipeline::new(device, texture),

            build_plate: BuildPlateDispatch::new(),
            normals: NormalsDispatch::new(),
            line_support_debug: LineSupportDebugDispatch::new(),
        }
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue, resources: &WorkspaceRenderCallback) {
        let dispatches: &mut [&mut dyn LineDispatch] = &mut [
            &mut self.build_plate,
            &mut self.normals,
            &mut self.line_support_debug,
        ];

        let mut changed = false;
        for dispatch in dispatches.iter_mut() {
            changed |= dispatch.generate_lines(resources);
        }

        if changed {
            let lines = &[
                self.build_plate.lines(),
                self.normals.lines(),
                self.line_support_debug.lines(),
            ][..];
            self.render_pipeline
                .prepare(device, queue, resources, Some(lines));
        } else {
            self.render_pipeline.prepare(device, queue, resources, None);
        }
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        self.render_pipeline.paint(render_pass);
    }
}

trait LineDispatch {
    fn generate_lines(&mut self, resources: &WorkspaceRenderCallback) -> bool;
    fn lines(&self) -> &[Line];
}
