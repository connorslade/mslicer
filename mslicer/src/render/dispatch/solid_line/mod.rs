use build_plate::BuildPlateDispatch;
use wgpu::{Device, Queue, RenderPass};

use crate::render::{pipelines::solid_line::SolidLinePipeline, workspace::WorkspaceRenderCallback};

mod build_plate;

pub struct SolidLineDispatch {
    render_pipeline: SolidLinePipeline,

    build_plate: BuildPlateDispatch,
}

impl SolidLineDispatch {
    pub fn new(device: &Device) -> Self {
        Self {
            render_pipeline: SolidLinePipeline::new(device),

            build_plate: BuildPlateDispatch::new(),
        }
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue, resources: &WorkspaceRenderCallback) {
        let changed = self
            .build_plate
            .generate_lines(resources.bed_size, resources.grid_size);

        if changed {
            let lines = &[self.build_plate.lines()][..];
            self.render_pipeline
                .prepare(device, &queue, resources, Some(lines));
        } else {
            self.render_pipeline
                .prepare(device, &queue, resources, None);
        }
    }

    pub fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.render_pipeline.paint(render_pass);
    }
}
