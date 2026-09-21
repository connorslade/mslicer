use nalgebra::Vector3;

pub mod auto;
pub mod mesh;

pub struct Support {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub radius: f32,
}

pub struct SupportConfig {
    /// Support generation
    pub support_radius: f32,
    pub tip_radius: f32,
    pub tip_length: f32,
    pub precision: u32,

    pub min_spacing: f32,

    pub max_angle: f32,
    pub face_support_spacing: f32,
    pub edge_support_spacing: f32,
    pub edge_angle_delta: f32,
}

pub struct SupportPlacement {
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
}

impl Default for SupportConfig {
    fn default() -> Self {
        Self {
            support_radius: 1.0,
            tip_radius: 0.2,
            tip_length: 3.0,
            min_spacing: 5.0,
            precision: 10,
            max_angle: 30.0,
            face_support_spacing: 50.0,
            edge_angle_delta: 0.1,
            edge_support_spacing: 20.0,
        }
    }
}
