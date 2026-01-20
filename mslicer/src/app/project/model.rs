use std::{
    f32::consts::TAU,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use bitflags::bitflags;
use common::color::{LinearRgb, START_COLOR};
use nalgebra::Vector3;
use wgpu::{Buffer, Device};

use slicer::{
    geometry::bvh::Bvh, half_edge::HalfEdgeMesh, mesh::Mesh,
    supports::overhangs::detect_point_overhangs,
};

use crate::render::util::gpu_mesh_buffers;

pub struct Model {
    pub name: String,
    pub id: u32,

    pub mesh: Mesh,
    pub bvh: Option<Arc<Bvh>>,
    pub half_edge: Option<Arc<HalfEdgeMesh>>,

    pub warnings: MeshWarnings,
    pub overhangs: Option<Vec<u32>>,

    pub color: LinearRgb<f32>,
    pub hidden: bool,
    pub locked_scale: bool,

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

impl Model {
    pub fn from_mesh(mesh: Mesh) -> Self {
        Self {
            name: String::new(),
            id: next_id(),

            bvh: None,
            half_edge: None,
            mesh,

            warnings: MeshWarnings::empty(),
            overhangs: None,

            color: LinearRgb::repeat(1.0),
            hidden: false,
            locked_scale: true,
            buffers: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_color(mut self, color: LinearRgb<f32>) -> Self {
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
        self.color = START_COLOR.hue_shift(shift).to_linear_srgb();
        self
    }

    // Returns a list of vertices that are lower than all their neighbors.
    pub fn find_overhangs(&mut self) {
        self.overhangs = Some(detect_point_overhangs(
            &self.mesh,
            self.half_edge.as_ref().unwrap(),
            |origin, _, _| origin.origin_vertex,
        ));
    }

    pub fn try_get_buffers(&self) -> Option<&RenderedMeshBuffers> {
        self.buffers.as_ref()
    }

    pub fn get_buffers(&mut self, device: &Device) -> &RenderedMeshBuffers {
        if self.buffers.is_none() {
            let (vertex_buffer, index_buffer) = gpu_mesh_buffers(device, &self.mesh);
            self.buffers = Some(RenderedMeshBuffers {
                vertex_buffer,
                index_buffer,
            });
        }

        self.buffers.as_ref().unwrap()
    }
}

impl Model {
    pub fn align_to_bed(&mut self) {
        let (bottom, _) = self.mesh.bounds();

        let pos = self.mesh.position() - Vector3::z() * bottom.z;
        self.mesh.set_position(pos);
    }

    pub fn update_oob(&mut self, platform: &Vector3<f32>) {
        let (min, max) = self.mesh.bounds();
        let half = platform / 2.0;

        let oob = (min.x < -half.x || min.y < -half.y || min.z < 0.0)
            || (max.x > half.x || max.y > half.y || max.z > platform.z);
        self.warnings.set(MeshWarnings::OutOfBounds, oob);
    }

    pub fn set_position(&mut self, platform: &Vector3<f32>, pos: Vector3<f32>) {
        self.mesh.set_position(pos);
        self.update_oob(platform);
    }

    pub fn set_scale(&mut self, platform: &Vector3<f32>, scale: Vector3<f32>) {
        self.mesh.set_scale(scale);
        self.update_oob(platform);
        self.overhangs = None;
    }

    pub fn set_rotation(&mut self, platform: &Vector3<f32>, rotation: Vector3<f32>) {
        self.mesh.set_rotation(rotation);
        self.update_oob(platform);
        self.overhangs = None;
    }
}

// todo: this really bad...
impl Clone for Model {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            id: next_id(),

            mesh: self.mesh.clone(),
            bvh: self.bvh.clone(),
            half_edge: self.half_edge.clone(),

            warnings: self.warnings,
            overhangs: self.overhangs.clone(),

            color: self.color,
            hidden: self.hidden,
            locked_scale: self.locked_scale,
            buffers: None,
        }
    }
}

fn next_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
