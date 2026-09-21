use common::id_type;
use nalgebra::Vector3;

pub mod auto;
pub mod mesh;

pub struct Support {
    pub id: SupportId,
    pub points: [Vector3<f32>; 3],

    pub tip_radius: f32,
    pub radius: f32,
}

id_type!(SupportId, u32);
