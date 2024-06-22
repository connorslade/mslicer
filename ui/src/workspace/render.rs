use std::mem;

use eframe::CreationContext;
use nalgebra::Vector3;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::Buffer;
use wgpu::Device;
use wgpu::{
    BindGroupEntry, BindGroupLayoutDescriptor, BufferAddress, BufferBinding, BufferDescriptor,
    BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, FragmentState,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPipelineDescriptor,
    ShaderModuleDescriptor, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
};

use slicer::mesh::Mesh;

use crate::workspace::WorkspaceRenderCallback;
use crate::workspace::WorkspaceRenderResources;
use crate::TEXTURE_FORMAT;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 4],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

pub struct RenderedMesh {
    pub name: String,
    pub mesh: Mesh,
    pub hidden: bool,
    vertices: Vec<ModelVertex>,
    buffers: Option<RenderedMeshBuffers>,
}

pub struct RenderedMeshBuffers {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl RenderedMesh {
    pub fn from_mesh(mesh: Mesh) -> Self {
        let normals = mesh
            .faces
            .iter()
            .enumerate()
            .map(|(idx, face)| {
                let (p1, p2, p3) = (
                    mesh.vertices[face[0] as usize],
                    mesh.vertices[face[1] as usize],
                    mesh.vertices[face[2] as usize],
                );
                let a = p2 - p1;
                let b = p3 - p1;
                (idx, a.cross(&b).normalize())
            })
            .collect::<Vec<_>>();

        let mut vertex_faces = vec![Vec::new(); mesh.vertices.len()];
        for (face_idx, face) in mesh.faces.iter().enumerate() {
            for vertex_idx in face.iter() {
                vertex_faces[*vertex_idx as usize].push(face_idx);
            }
        }

        let vertices = mesh
            .vertices
            .iter()
            .enumerate()
            .map(|(idx, v)| ModelVertex {
                position: [v.x, v.y, v.z, 1.0],
                tex_coords: [0.0, 0.0],
                normal: {
                    let mut normal = Vector3::new(0.0, 0.0, 0.0);
                    for face_idx in &vertex_faces[idx] {
                        normal += normals[*face_idx].1;
                    }
                    normal.normalize().into()
                },
            })
            .collect::<Vec<ModelVertex>>();

        Self {
            mesh,
            name: String::new(),
            hidden: false,
            vertices,
            buffers: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn try_get_buffers(&self) -> Option<&RenderedMeshBuffers> {
        self.buffers.as_ref()
    }

    pub fn get_buffers(&mut self, device: &Device) -> &RenderedMeshBuffers {
        if self.buffers.is_none() {
            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.mesh.faces),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            });

            self.buffers = Some(RenderedMeshBuffers {
                vertex_buffer,
                index_buffer,
            });
        }

        self.buffers.as_ref().unwrap()
    }
}

pub fn init_wgpu(cc: &CreationContext) {
    let render_state = cc.wgpu_render_state.as_ref().unwrap();
    let device = &render_state.device;

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/workspace.wgsl").into()),
    });

    let uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        size: mem::size_of::<WorkspaceRenderCallback>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let vertex_buffers = [VertexBufferLayout {
        array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &[
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 4 * 4,
                shader_location: 1,
            },
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 4 * 4 + 4 * 2,
                shader_location: 2,
            },
        ],
    }];

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
            entry_point: "vert",
            buffers: &vertex_buffers,
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
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(BufferBinding {
                buffer: &uniform_buffer,
                offset: 0,
                size: None,
            }),
        }],
    });

    render_state
        .renderer
        .write()
        .callback_resources
        .insert(WorkspaceRenderResources {
            uniform_buffer,

            render_pipeline,
            bind_group,
        });
}
