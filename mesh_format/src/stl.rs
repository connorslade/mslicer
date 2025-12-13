use std::collections::HashMap;

use anyhow::Result;
use common::serde::Deserializer;
use nalgebra::Vector3;

use crate::{Mesh, Progress};

// From Wikipedia :eyes:
// UINT8[80]    – Header                 - 80 bytes
// UINT32       – Number of triangles    - 04 bytes
// foreach triangle                      - 50 bytes
//     REAL32[3] – Normal vector         - 12 bytes
//     REAL32[3] – Vertex 1              - 12 bytes
//     REAL32[3] – Vertex 2              - 12 bytes
//     REAL32[3] – Vertex 3              - 12 bytes
//     UINT16    – Attribute byte count  - 02 bytes
// end

pub fn parse<T: Deserializer>(des: &mut T, progress: Progress) -> Result<Mesh> {
    des.advance_by(80); // skip header
    let tri_count = des.read_u32_le();
    progress.set_total(tri_count);

    // let mut out = Vec::new();
    let mut verts = HashMap::new();
    let mut vert_idx = |vert: Vector3<f32>| {
        let size = verts.len() as u32;
        *verts.entry(vert.map(f32::to_bits)).or_insert(size)
    };

    let mut mesh = Mesh::default();
    for i in 0..tri_count {
        progress.set_complete(i);
        des.advance_by(4 * 3);
        mesh.faces.push([
            vert_idx(des_vec3f(des)),
            vert_idx(des_vec3f(des)),
            vert_idx(des_vec3f(des)),
        ]);
        des.advance_by(2);
    }

    let mut faces = verts.into_iter().collect::<Vec<_>>();
    faces.sort_by_key(|(_vert, idx)| *idx);
    mesh.verts = (faces.into_iter())
        .map(|(vert, _idx)| vert.map(f32::from_bits))
        .collect();

    Ok(mesh)
}

fn des_vec3f<T: Deserializer>(des: &mut T) -> Vector3<f32> {
    Vector3::new(des.read_f32_le(), des.read_f32_le(), des.read_f32_le())
}
