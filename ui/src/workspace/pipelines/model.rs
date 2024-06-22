use wgpu::{
    ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, FragmentState, IndexFormat,
    MultisampleState, PrimitiveState, RenderPass, RenderPipelineDescriptor, ShaderModuleDescriptor,
    TextureFormat, VertexState,
};

use crate::{
    include_shader,
    workspace::{
        render::VERTEX_BUFFER_LAYOUT, rendered_mesh::RenderedMeshBuffers, WorkspaceRenderCallback,
    },
    TEXTURE_FORMAT,
};

use super::Pipeline;

pub struct ModelPipeline {}

impl ModelPipeline {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pipeline for ModelPipeline {
    fn init(
        &self,
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_shader!("model.wgsl").into()),
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vert",
                buffers: &[VERTEX_BUFFER_LAYOUT],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "frag",
                targets: &[Some(ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: None,
                    write_mask: ColorWrites::all(),
                })],
            }),
            primitive: PrimitiveState {
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
        })
    }

    fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, resources: &WorkspaceRenderCallback) {
        let modals = resources.modals.read().unwrap();
        for modal in modals.iter().filter(|x| !x.hidden) {
            // SAFETY: im really tired and i dont care anymore
            let buffers: &RenderedMeshBuffers =
                unsafe { &*(modal.try_get_buffers().unwrap() as *const _) };
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..(3 * modal.mesh.faces.len() as u32), 0, 0..1);
        }
    }
}
