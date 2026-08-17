use std::{f32::consts::TAU, sync::Arc};

use bitflags::bitflags;
use common::{
    color::{LinearRgb, START_COLOR},
    id_type,
    serde::{Deserializer, Serializer},
    units::{CubicMilimeters, Milimeters},
};
use nalgebra::Vector3;
use wgpu::{Buffer, Device};

use slicer::{geometry::bvh::Bvh, half_edge::HalfEdgeMesh, mesh::Mesh};

use crate::{
    project::{CollectionId, RenameState, supports::Supports},
    render::util::gpu_mesh_buffers,
};

pub struct Model {
    pub name: String,
    pub id: ModelId,
    pub collection: Option<CollectionId>,

    pub mesh: Mesh,
    pub bvh: Option<Arc<Bvh>>,
    pub half_edge: Option<Arc<HalfEdgeMesh>>,
    base_volume: CubicMilimeters,

    pub unit: MeshUnit,
    pub color: LinearRgb<f32>,
    pub exposure: u8,
    pub hidden: bool,
    pub ui: ModelUi,

    pub warnings: MeshWarnings,
    buffers: Option<RenderedMeshBuffers>,
    pub supports: Supports,
}

#[derive(Clone)]
pub struct ModelUi {
    pub rename: RenameState,
    pub locked_scale: bool,
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

#[derive(Clone, Copy, PartialEq)]
pub enum MeshUnit {
    Millimeters,
    Centimeters,
    Meters,
    Inches,
    Custom(f32),
}

impl Model {
    pub fn from_mesh(mesh: Mesh) -> Self {
        Self {
            name: String::new(),
            id: ModelId::new(),
            collection: None,

            base_volume: mesh_volume(&mesh),
            bvh: None,
            half_edge: None,
            mesh,

            unit: MeshUnit::Millimeters,
            color: LinearRgb::repeat(1.0),
            hidden: false,
            exposure: 255,
            ui: ModelUi::default(),

            warnings: MeshWarnings::empty(),
            buffers: None,
            supports: Supports::default(),
        }
    }

    pub fn volume(&self) -> CubicMilimeters {
        let scale = self.mesh.scale();
        self.base_volume * scale.x * scale.y * scale.z
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_color(mut self, color: LinearRgb<f32>) -> Self {
        self.color = color;
        self
    }

    pub fn with_exposure(mut self, exposure: u8) -> Self {
        self.exposure = exposure;
        self
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn with_collection(mut self, collection: Option<CollectionId>) -> Self {
        self.collection = collection;
        self
    }

    pub fn with_unit(mut self, unit: MeshUnit) -> Self {
        self.unit = unit;
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

    pub fn update_oob(&mut self, platform: &Vector3<Milimeters>) {
        let (min, max) = self.mesh.bounds();
        let half = platform.map(|x| x.raw()) / 2.0;

        let oob = (min.x < -half.x || min.y < -half.y || min.z < 0.0)
            || (max.x > half.x || max.y > half.y || max.z > platform.z.raw());
        self.warnings.set(MeshWarnings::OutOfBounds, oob);
    }

    pub fn set_position(&mut self, platform: &Vector3<Milimeters>, pos: Vector3<f32>) {
        self.mesh.set_position(pos);
        self.update_oob(platform);
    }

    pub fn set_scale(&mut self, platform: &Vector3<Milimeters>, scale: Vector3<f32>) {
        self.mesh.set_scale(scale);
        self.update_oob(platform);
    }

    pub fn set_rotation(&mut self, platform: &Vector3<Milimeters>, rotation: Vector3<f32>) {
        self.mesh.set_rotation(rotation);
        self.update_oob(platform);
    }
}

impl MeshUnit {
    pub const ALL: [Self; 5] = [
        Self::Millimeters,
        Self::Centimeters,
        Self::Meters,
        Self::Inches,
        Self::Custom(1.0),
    ];

    pub fn name(&self) -> &str {
        match self {
            Self::Millimeters => "Millimeters",
            Self::Centimeters => "Centimeters",
            Self::Meters => "Meters",
            Self::Inches => "Inches",
            Self::Custom(_) => "Custom",
        }
    }

    /// Conversion factor from self to millimeters
    pub fn conversion(&self) -> f32 {
        match self {
            Self::Millimeters => 1.0,
            Self::Centimeters => 10.0,
            Self::Meters => 1000.0,
            Self::Inches => 2.54,
            Self::Custom(x) => *x,
        }
    }

    pub fn ordinal(&self) -> u8 {
        match self {
            Self::Custom(_) => 0,
            Self::Millimeters => 1,
            Self::Centimeters => 2,
            Self::Meters => 3,
            Self::Inches => 4,
        }
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Some(match ordinal {
            1 => Self::Millimeters,
            2 => Self::Centimeters,
            3 => Self::Meters,
            4 => Self::Inches,
            _ => return None,
        })
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u8(self.ordinal());
        if let MeshUnit::Custom(factor) = self {
            ser.write_f32_be(*factor);
        }
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        let ordinal = des.read_u8();
        if ordinal == 0 {
            Self::Custom(des.read_f32_be())
        } else {
            Self::from_ordinal(ordinal).unwrap_or(Self::Millimeters)
        }
    }
}

impl Clone for Model {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            id: ModelId::new(),
            collection: self.collection,

            mesh: self.mesh.clone(),
            bvh: self.bvh.clone(),
            half_edge: self.half_edge.clone(),
            base_volume: self.base_volume,

            unit: self.unit,
            color: self.color,
            hidden: self.hidden,
            ui: self.ui.clone(),

            exposure: self.exposure,

            warnings: self.warnings,
            buffers: None,
            supports: Supports::default(),
        }
    }
}

impl Default for ModelUi {
    fn default() -> Self {
        Self {
            rename: RenameState::None,
            locked_scale: true,
        }
    }
}

id_type!(ModelId, u32);

// Reference: https://stackoverflow.com/a/13927691
fn mesh_volume(mesh: &Mesh) -> CubicMilimeters {
    let mut volume = 0.0;

    for face in 0..mesh.face_count() {
        let [a, b, c] = mesh.face_verts(face);
        volume += a.dot(&b.cross(&c)) / 6.0;
    }

    CubicMilimeters::new(volume.abs())
}
