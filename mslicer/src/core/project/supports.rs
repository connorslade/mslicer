use std::{collections::HashMap, f32::consts::PI, range::Range, time::Instant};

use common::id_type;
use nalgebra::{Vector2, Vector3};
use slicer::{builder::MeshBuilder, mesh::Mesh};
use tools::supports::{SupportConfig, build_raft_mesh};
use tracing::info;
use wgpu::Device;

use crate::{core::project::model::RenderedMeshBuffers, render::util::gpu_mesh_buffers};

#[derive(Default)]
pub struct Supports {
    auto: Vec<Support>,
    manual: Vec<Support>,
    transform: Transform,

    mesh: Option<(Mesh, FaceMap)>,
    buffers: Option<RenderedMeshBuffers>,
}

pub struct Support {
    id: SupportId,
    points: [Vector3<f32>; 3],
    tip_radius: f32,
    radius: f32,
}

#[derive(Clone, Copy)]
struct Transform {
    position: Vector3<f32>,
    scale: Vector3<f32>,
    _rotation: f32,
}

id_type!(SupportId, u32);

type FaceMap = HashMap<SupportId, Range<u32>>;

impl Supports {
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

    pub fn replace_auto(&mut self, config: &SupportConfig, supports: Vec<[Vector3<f32>; 3]>) {
        self.invalidate_cache();
        self.auto = supports
            .into_iter()
            .map(|points| Support {
                id: SupportId::new(),
                points,
                tip_radius: config.tip_radius,
                radius: config.support_radius,
            })
            .collect();
    }

    pub fn add_manual(&mut self, config: &SupportConfig, support: [Vector3<f32>; 3]) {
        self.invalidate_cache();
        self.manual.push(Support {
            id: SupportId::new(),
            points: support,
            tip_radius: config.tip_radius,
            radius: config.support_radius,
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
            let (r, p) = (support.radius, 20); // todo: make precision follow actual config...
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

        build_raft_mesh(1.0, 1.0, &raft_points, &mut builder);
        if !builder.is_empty() {
            let mesh = builder.build();
            let (faces, duration) = (mesh.face_count(), start.elapsed());
            info!("Generated support mesh with {faces} faces in {duration:?}");
            self.mesh = Some((mesh, map));
        }

        &self.mesh
    }

    pub fn get_buffers(&mut self, device: &Device) -> &Option<RenderedMeshBuffers> {
        if self.buffers.is_none()
            && let Some((mesh, _)) = self.mesh()
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

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            scale: Vector3::repeat(1.0),
            _rotation: 0.0,
        }
    }
}
