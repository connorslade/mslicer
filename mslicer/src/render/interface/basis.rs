use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3};
use slicer::{builder::MeshBuilder, mesh::Mesh};
use wgpu::{
    BindGroup, BlendState, Buffer, BufferDescriptor, ColorTargetState, ColorWrites, CommandBuffer,
    CommandEncoder, Device, FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, TextureFormat,
    VertexState,
};

use crate::{
    include_shader,
    render::{
        VERTEX_BUFFER_LAYOUT,
        camera::{Camera, FAR},
        consts::{
            BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, BASE_UNIFORM_DESCRIPTOR, DEPTH_STENCIL_STATE,
            bind_group,
        },
        util::gpu_mesh_buffers,
    },
};

pub struct BasisRenderCallback {
    pub camera: Camera,
}

pub struct BasisPipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    index_count: u32,
}

#[derive(ShaderType)]
struct BasisUniforms {
    transform: Matrix4<f32>,
}

impl BasisPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!("basis.wgsl", "common.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: BasisUniforms::SHADER_SIZE.get(),
            ..BASE_UNIFORM_DESCRIPTOR
        });

        let (bind_group_layout, bind_group) = bind_group(
            device,
            BASE_BIND_GROUP_LAYOUT_DESCRIPTOR,
            [uniform_buffer.as_entire_binding()],
        );

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[VERTEX_BUFFER_LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: texture,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DEPTH_STENCIL_STATE),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        let mesh = generate_axis_mesh();
        let (vertex_buffer, index_buffer) = gpu_mesh_buffers(device, &mesh);

        Self {
            render_pipeline,
            bind_group,

            vertex_buffer,
            index_buffer,
            uniform_buffer,

            index_count: mesh.face_count() as u32 * 3,
        }
    }
}

impl BasisPipeline {
    pub fn prepare(&mut self, queue: &Queue, camera: &Camera) {
        let project = Matrix4::new_orthographic(-1.0, 1.0, -1.0, 1.0, -FAR, FAR);
        let view = Matrix4::look_at_rh(
            &camera.position(1.0).into(),
            &Vector3::zeros().into(),
            &camera.up(),
        );

        let uniform = BasisUniforms {
            transform: project * view,
        };

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&uniform).unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

impl CallbackTrait for BasisRenderCallback {
    fn prepare(
        &self,
        _device: &Device,
        queue: &Queue,
        _screen: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let pipeline = resources.get_mut::<BasisPipeline>().unwrap();
        pipeline.prepare(queue, &self.camera);

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass,
        resources: &CallbackResources,
    ) {
        let pipeline = resources.get::<BasisPipeline>().unwrap();
        pipeline.paint(render_pass);
    }
}

fn generate_axis_mesh() -> Mesh {
    let mut builder = MeshBuilder::new();
    let p = 10;
    let r = 0.08;

    for axis in [0, 1, 2] {
        let [a, b, c] = [0.0, 0.5, 1.0].map(|x| {
            let mut out = Vector3::zeros();
            out[axis] = x;
            out
        });

        builder.add_sphere(a, r, p);
        builder.add_cylinder((a, b), (r, r), p);
        builder.add_cylinder((b, c), (0.2, 0.0), p);
    }

    builder.build()
}
