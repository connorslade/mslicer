use std::{collections::HashMap, f32::consts::PI, range::Range, time::Instant};

use common::units::Milimeter;
use nalgebra::{Vector2, Vector3};
use slicer::{builder::MeshBuilder, mesh::Mesh};
use tools::supports::{
    Support, SupportId,
    mesh::{FaceMap, SupportPreset, Transform, build_raft},
};
use tracing::info;
use wgpu::Device;

use crate::render::util::RenderedMeshBuffers;

#[derive(Default)]
pub struct Supports {
    auto: Vec<Support>,
    manual: Vec<Support>,
    transform: Transform,

    resolution: u32,
    mesh: Option<(Mesh, FaceMap)>,
    buffers: Option<RenderedMeshBuffers>,
}

impl Supports {
    pub fn set_resolution(&mut self, resolution: u32) {
        self.resolution = resolution;
    }

    pub fn invalidate_cache(&mut self) {
        self.mesh.take();
        self.buffers.take();
    }

    pub fn clear(&mut self) {
        self.auto.clear();
        self.manual.clear();
        self.transform = Default::default();
        self.invalidate_cache();
    }

    pub fn replace_auto(&mut self, config: &SupportPreset, supports: Vec<[Vector3<f32>; 3]>) {
        self.invalidate_cache();
        self.auto = supports
            .into_iter()
            .map(|points| Support {
                id: SupportId::new(),
                points,
                tip_radius: config.tip_radius.get::<Milimeter>(),
                radius: config.support_radius.get::<Milimeter>(),
            })
            .collect();
    }

    pub fn add_manual(&mut self, config: &SupportPreset, support: [Vector3<f32>; 3]) {
        self.invalidate_cache();
        self.manual.push(Support {
            id: SupportId::new(),
            points: support,
            tip_radius: config.tip_radius.get::<Milimeter>(),
            radius: config.support_radius.get::<Milimeter>(),
        });
    }

    pub fn mesh(&mut self) -> &Option<(Mesh, FaceMap)> {
        if self.mesh.is_some() || (self.auto.is_empty() && self.manual.is_empty()) {
            return &self.mesh;
        }

        let start = Instant::now();
        let mut builder = MeshBuilder::new();
        let mut raft_points = Vec::new();
        let mut map = HashMap::new();

        for support in self.auto.iter().chain(self.manual.iter()) {
            let (r, p) = (support.radius, self.resolution);
            let mut points = support.points;

            //rotate!?

            // ↓ incorrect!
            points[0] = points[0].component_mul(&self.transform.scale);
            points[1] = points[1].component_mul(&self.transform.scale);
            points[2] = points[2].component_mul(&self.transform.scale);

            points[0].z += self.transform.position.z;
            points[1].z += self.transform.position.z;
            points[2].z += self.transform.position.z;

            let start = builder.next_face_idx();
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

            let end = builder.next_face_idx();
            map.insert(support.id, Range::from(start..end));
        }

        build_raft(1.0, 1.0, &raft_points, &mut builder);
        if !builder.is_empty() {
            let mesh = builder.build();
            let (faces, duration) = (mesh.face_count(), start.elapsed());
            info!("Generated support mesh with {faces} faces in {duration:?}");
            self.mesh = Some((mesh, map));
        }

        &self.mesh
    }

    pub fn support_for_face(&mut self, face: u32) -> Option<SupportId> {
        let (_, map) = self.mesh().as_ref()?;
        (map.iter().find(|(_, r)| r.contains(&face))).map(|(id, _)| *id)
    }

    pub fn get_buffers(&mut self, device: &Device) -> &Option<RenderedMeshBuffers> {
        if self.buffers.is_none()
            && let Some((mesh, _)) = self.mesh()
        {
            self.buffers = Some(RenderedMeshBuffers::get_for(device, mesh));
        }

        &self.buffers
    }

    pub fn try_get_buffers(&self) -> Option<(&RenderedMeshBuffers, u32)> {
        self.buffers.as_ref().map(|x| {
            let face_count = self.mesh.as_ref().unwrap().0.face_count() as u32 * 3;
            (x, face_count)
        })
    }

    // kickin it old school rn
    pub fn set_transform(
        &mut self,
        position: Vector3<f32>,
        scale: Vector3<f32>,
        rotation: Vector3<f32>,
    ) {
        if let Some((mesh, _)) = &mut self.mesh {
            if mesh.rotation().xy() != rotation.xy() {
                self.clear();
                return;
            }

            mesh.set_position(position.xy().push(mesh.position().z));
            mesh.set_rotation(mesh.rotation().xy().push(rotation.z));

            let old = self.transform;
            self.transform = Transform {
                position,
                scale,
                _rotation: rotation.z,
            };

            if scale != old.scale || position.z != old.position.z {
                self.invalidate_cache();
            }
        }
    }

    pub fn remove(&mut self, id: SupportId) {
        self.auto.retain(|x| x.id != id);
        self.manual.retain(|x| x.id != id);
    }
}
