use build_plate::BuildPlateDispatch;
use normals::NormalsDispatch;
use wgpu::{Device, Queue, RenderPass};

use crate::render::{
    pipelines::solid_line::{Line, SolidLinePipeline},
    workspace::WorkspaceRenderCallback,
};

mod build_plate;
mod normals;

pub struct SolidLineDispatch {
    render_pipeline: SolidLinePipeline,

    build_plate: BuildPlateDispatch,
    normals: NormalsDispatch,
}

impl SolidLineDispatch {
    pub fn new(device: &Device) -> Self {
        Self {
            render_pipeline: SolidLinePipeline::new(device),

            build_plate: BuildPlateDispatch::new(),
            normals: NormalsDispatch::new(),
        }
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue, resources: &WorkspaceRenderCallback) {
        let dispatches: &mut [&mut dyn LineDispatch] =
            &mut [&mut self.build_plate, &mut self.normals];

        let mut changed = false;
        for dispatch in dispatches.iter_mut() {
            changed |= dispatch.generate_lines(resources);
        }

        if changed {
            let lines = &[self.build_plate.lines(), self.normals.lines()][..];
            self.render_pipeline
                .prepare(device, queue, resources, Some(lines));
        } else {
            self.render_pipeline.prepare(device, queue, resources, None);
        }
    }

    pub fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.render_pipeline.paint(render_pass);
    }
}

trait LineDispatch {
    fn generate_lines(&mut self, resources: &WorkspaceRenderCallback) -> bool;
    fn lines(&self) -> &[Line];
}
