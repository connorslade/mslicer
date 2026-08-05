use std::f32::consts::PI;

use nalgebra::{Vector2, Vector3};
use slicer::{builder::MeshBuilder, mesh::Mesh};
use tools::supports::{SupportConfig, build_raft_mesh};
use wgpu::Device;

use crate::{project::model::RenderedMeshBuffers, render::util::gpu_mesh_buffers};

#[derive(Default)]
pub struct Supports {
    auto: Vec<Support>,
    manual: Vec<Support>,

    mesh: Option<Mesh>,
    buffers: Option<RenderedMeshBuffers>,
}

pub struct Support {
    points: [Vector3<f32>; 3],
    tip_radius: f32,
    radius: f32,
}

impl Supports {
    pub fn invalidate_cache(&mut self) {
        self.mesh.take();
        self.buffers.take();
    }

    pub fn replace_auto(&mut self, config: &SupportConfig, supports: Vec<[Vector3<f32>; 3]>) {
        self.invalidate_cache();
        self.auto = supports
            .into_iter()
            .map(|points| Support {
                points,
                tip_radius: config.tip_radius,
                radius: config.support_radius,
            })
            .collect();
    }

    pub fn add_manual(&mut self, config: &SupportConfig, support: [Vector3<f32>; 3]) {
        self.invalidate_cache();
        self.manual.push(Support {
            points: support,
            tip_radius: config.tip_radius,
            radius: config.support_radius,
        });
    }

    pub fn mesh(&mut self) -> &Option<Mesh> {
        if self.mesh.is_some() || (self.auto.is_empty() && self.manual.is_empty()) {
            return &self.mesh;
        }

        let mut builder = MeshBuilder::new();
        let mut raft_points = Vec::new();
        for support in self.auto.iter().chain(self.manual.iter()) {
            let (r, p) = (support.radius, 20); // todo: make precision follow actual config...
            let points = &support.points;

            builder.add_cylinder((points[0], points[1]), (support.tip_radius, r), p);
            builder.add_cylinder((points[1], points[2]), (r, r), p);
            builder.add_cylinder((points[2], points[2].xy().push(0.0)), (r, r), p);

            for i in 0..(p * 2) {
                let angle = i as f32 / p as f32 * PI;
                let normal = Vector2::new(angle.cos(), angle.sin());
                raft_points.push(points[2].xy() + normal * r);
            }

            builder.add_sphere(points[0], 0.2, p);
            builder.add_sphere(points[1], r, p);
            builder.add_sphere(points[2], r, p);
        }

        build_raft_mesh(1.0, 1.0, &raft_points, &mut builder);
        self.mesh = (!builder.is_empty()).then(|| builder.build());
        &self.mesh
    }

    pub fn get_buffers(&mut self, device: &Device) -> &Option<RenderedMeshBuffers> {
        if self.buffers.is_none()
            && let Some(mesh) = self.mesh()
        {
            let (vertex_buffer, index_buffer) = gpu_mesh_buffers(device, mesh);
            self.buffers = Some(RenderedMeshBuffers {
                vertex_buffer,
                index_buffer,
            });
        }

        &self.buffers
    }

    pub fn try_get_buffers(&self) -> Option<(&RenderedMeshBuffers, u32)> {
        self.buffers
            .as_ref()
            .map(|x| (x, self.mesh.as_ref().unwrap().face_count() as u32 * 3))
    }
}
