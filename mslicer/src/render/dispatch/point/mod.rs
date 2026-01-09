use wgpu::{Device, Queue, RenderPass, TextureFormat};

use crate::render::{
    dispatch::point::{overhangs::OverhangPointDispatch, target::TargetPointDispatch},
    pipelines::point::{Point, PointPipeline},
    workspace::WorkspaceRenderCallback,
};

mod overhangs;
mod target;

pub struct PointDispatch {
    render_pipeline: PointPipeline,

    target_point: TargetPointDispatch,
    overhangs: OverhangPointDispatch,
}

impl PointDispatch {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        Self {
            render_pipeline: PointPipeline::new(device, texture),

            target_point: TargetPointDispatch::new(),
            overhangs: OverhangPointDispatch::new(),
        }
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue, resources: &WorkspaceRenderCallback) {
        let dispatches: &mut [&mut dyn PointGenerator] =
            &mut [&mut self.target_point, &mut self.overhangs];

        let mut changed = false;
        for dispatch in dispatches.iter_mut() {
            changed |= dispatch.generate_points(resources);
        }

        if changed {
            let points = &[self.target_point.points(), self.overhangs.points()][..];
            self.render_pipeline
                .prepare(device, queue, resources, Some(points));
        } else {
            self.render_pipeline.prepare(device, queue, resources, None);
        }
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        self.render_pipeline.paint(render_pass);
    }
}

trait PointGenerator {
    fn generate_points(&mut self, resources: &WorkspaceRenderCallback) -> bool;
    fn points(&self) -> &[Point];
}
