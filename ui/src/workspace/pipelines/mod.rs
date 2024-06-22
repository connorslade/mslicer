use wgpu::{Device, PipelineLayout, RenderPass, RenderPipeline};

use super::WorkspaceRenderCallback;

pub mod build_plate;
pub mod model;

pub trait Pipeline {
    fn init(&self, device: &Device, pipeline_layout: &PipelineLayout) -> RenderPipeline;
    fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, resources: &WorkspaceRenderCallback);
}

pub struct CachedPipeline<T: Pipeline> {
    render_pipeline: Option<RenderPipeline>,
    pub pipeline: T,
}

impl<T: Pipeline> CachedPipeline<T> {
    pub fn new(pipeline: T) -> Self {
        Self {
            render_pipeline: None,
            pipeline,
        }
    }

    pub fn prepare(mut self, device: &Device, pipeline_layout: &PipelineLayout) -> Self {
        if self.render_pipeline.is_none() {
            self.render_pipeline = Some(self.pipeline.init(device, pipeline_layout));
        }

        self
    }

    pub fn get_render_pipeline(&self) -> &RenderPipeline {
        self.render_pipeline.as_ref().unwrap()
    }
}

#[macro_export]
macro_rules! include_shader {
    ($shader:literal) => {
        include_str!(concat!("../../shaders/", $shader))
    };
}
