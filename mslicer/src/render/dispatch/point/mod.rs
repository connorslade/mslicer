use wgpu::{Device, RenderPass, TextureFormat};

use crate::render::{
    dispatch::point::{overhangs::OverhangPointDispatch, target::TargetPointDispatch},
    pipelines::point::{Point, PointPipeline},
    workspace::{Gcx, WorkspaceRenderCallback},
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

    pub fn prepare(&mut self, gcx: &Gcx, resources: &WorkspaceRenderCallback) {
        let dispatches: &mut [&mut dyn PointGenerator] =
            &mut [&mut self.target_point, &mut self.overhangs];
        for dispatch in dispatches.iter_mut() {
            dispatch.generate_points(resources);
        }

        let points = &[self.target_point.points(), self.overhangs.points()][..];
        self.render_pipeline.prepare(gcx, resources, points);
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        self.render_pipeline.paint(render_pass);
    }
}

trait PointGenerator {
    fn generate_points(&mut self, resources: &WorkspaceRenderCallback);
    fn points(&self) -> &[Point];
}
