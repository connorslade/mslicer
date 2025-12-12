use std::{
    f32::consts::TAU,
    sync::atomic::{AtomicU32, Ordering},
};

use bitflags::bitflags;
use common::{config::SliceConfig, oklab::START_COLOR};
use egui::Color32;
use nalgebra::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use slicer::mesh::Mesh;

use crate::{app::App, render::ModelVertex};

pub struct RenderedMesh {
    pub name: String,
    pub id: u32,
    pub mesh: Mesh,
    pub color: Color32,
    pub hidden: bool,
    pub locked_scale: bool,
    pub warnings: MeshWarnings,

    buffers: Option<RenderedMeshBuffers>,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct MeshWarnings: u8 {
        const NonManifold = 1 << 0;
        const OutOfBounds = 1 << 1;
    }
}

pub struct RenderedMeshBuffers {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl RenderedMesh {
    pub fn from_mesh(mesh: Mesh) -> Self {
        let mut warnings = MeshWarnings::empty();
        (!mesh.is_manifold()).then(|| warnings.insert(MeshWarnings::NonManifold));

        Self {
            name: String::new(),
            id: next_id(),
            mesh,
            color: Color32::WHITE,
            hidden: false,
            locked_scale: true,
            warnings,
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
        let shift = rand::random::<f32>() * TAU;
        let color = START_COLOR
            .hue_shift(shift)
            .to_srgb()
            .map(|x| (x.clamp(0.0, 1.0) * 255.0) as u8);
        self.color = Color32::from_rgb(color.r, color.g, color.b);
        self
    }

    pub fn try_get_buffers(&self) -> Option<&RenderedMeshBuffers> {
        self.buffers.as_ref()
    }

    pub fn get_buffers(&mut self, device: &Device) -> &RenderedMeshBuffers {
        if self.buffers.is_none() {
            let (vertices, faces) = (self.mesh.vertices(), self.mesh.faces());

            let index = faces.iter().flatten().copied().collect::<Vec<_>>();
            let vertices = (vertices.iter())
                .map(|vert| ModelVertex::new(vert.push(1.0)))
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&index),
                usage: BufferUsages::INDEX,
            });

            self.buffers = Some(RenderedMeshBuffers {
                vertex_buffer,
                index_buffer,
            });
        }

        self.buffers.as_ref().unwrap()
    }
}

impl RenderedMesh {
    pub fn align_to_bed(&mut self) {
        let (bottom, _) = self.mesh.bounds();

        let pos = self.mesh.position() - Vector3::new(0.0, 0.0, bottom.z);
        self.mesh.set_position(pos);
    }

    pub fn update_oob(&mut self, config: &SliceConfig) {
        let (min, max) = self.mesh.bounds();
        let platform = config.platform_size.map(|x| x);
        let half = platform.map(|x| x / 2.0);

        let oob = (min.x < -half.x || min.y < -half.y || min.z < 0.0)
            || (max.x > half.x || max.y > half.y || max.z > platform.z);
        self.warnings.set(MeshWarnings::OutOfBounds, oob);
    }

    pub fn set_position(&mut self, app: &App, pos: Vector3<f32>) {
        self.mesh.set_position(pos);
        self.update_oob(&app.slice_config);
    }

    pub fn set_scale(&mut self, app: &App, scale: Vector3<f32>) {
        self.mesh.set_scale(scale);
        self.update_oob(&app.slice_config);
    }

    pub fn set_rotation(&mut self, app: &App, rotation: Vector3<f32>) {
        self.mesh.set_rotation(rotation);
        self.update_oob(&app.slice_config);
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
            warnings: self.warnings,
            buffers: None,
        }
    }
}

fn next_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
