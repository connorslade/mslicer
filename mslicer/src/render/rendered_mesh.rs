use std::sync::atomic::{AtomicU32, Ordering};

use common::oklab::START_COLOR;
use egui::Color32;
use nalgebra::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use slicer::mesh::Mesh;

use super::ModelVertex;

pub struct RenderedMesh {
    pub name: String,
    pub id: u32,
    pub mesh: Mesh,
    pub color: Color32,
    pub hidden: bool,
    pub locked_scale: bool,

    vertices: Vec<ModelVertex>,
    buffers: Option<RenderedMeshBuffers>,
}

pub struct RenderedMeshBuffers {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl RenderedMesh {
    pub fn from_mesh(mesh: Mesh) -> Self {
        let mut out = Vec::new();

        let (vertices, normals) = (mesh.vertices(), mesh.normals());
        for (i, face) in mesh.faces().iter().enumerate() {
            let (a, b, c) = (
                vertices[face[0] as usize],
                vertices[face[1] as usize],
                vertices[face[2] as usize],
            );
            let normal = normals[i];

            out.extend_from_slice(&[
                ModelVertex {
                    position: [a.x, a.y, a.z, 1.0],
                    normal: normal.into(),
                },
                ModelVertex {
                    position: [b.x, b.y, b.z, 1.0],
                    normal: normal.into(),
                },
                ModelVertex {
                    position: [c.x, c.y, c.z, 1.0],
                    normal: normal.into(),
                },
            ]);
        }

        Self {
            name: String::new(),
            id: next_id(),
            mesh,
            color: Color32::WHITE,
            hidden: false,
            locked_scale: true,
            vertices: out,
            buffers: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn with_random_color(mut self) -> Self {
        self.randomize_color();
        self
    }

    pub fn randomize_color(&mut self) -> &mut Self {
        let shift = rand::random::<f32>() * std::f32::consts::PI * 2.0;
        let color = START_COLOR
            .hue_shift(shift)
            .to_srgb()
            .map(|x| (x.clamp(0.0, 1.0) * 255.0) as u8);
        self.color = Color32::from_rgb(color.r, color.g, color.b);
        self
    }

    pub fn align_to_bed(&mut self) {
        let (bottom, _) = self.mesh.minmax_point();

        let pos = self.mesh.position() - Vector3::new(0.0, 0.0, bottom.z);
        self.mesh.set_position(pos);
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
                contents: bytemuck::cast_slice(
                    &(0..self.mesh.face_count() as u32 * 3).collect::<Vec<u32>>(),
                ),
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

impl Clone for RenderedMesh {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            id: next_id(),
            mesh: self.mesh.clone(),
            color: self.color,
            hidden: self.hidden,
            locked_scale: self.locked_scale,
            vertices: self.vertices.clone(),
            buffers: None,
        }
    }
}

fn next_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
