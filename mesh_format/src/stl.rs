use std::collections::HashMap;

use anyhow::Result;
use common::serde::Deserializer;
use nalgebra::Vector3;

use crate::{Mesh, Progress};

pub fn parse<T: Deserializer>(des: &mut T, progress: Progress) -> Result<Mesh> {
    let is_ascii = &*des.read_bytes(5) == b"solid";
    des.jump_to(0);

    if is_ascii {
        ascii::parse(des, progress)
    } else {
        binary::parse(des, progress)
    }
}

/// From Wikipedia :eyes:
/// ```
/// UINT8[80]    – Header                 - 80 bytes
/// UINT32       – Number of triangles    - 04 bytes
/// foreach triangle                      - 50 bytes
///     REAL32[3] – Normal vector         - 12 bytes
///     REAL32[3] – Vertex 1              - 12 bytes
///     REAL32[3] – Vertex 2              - 12 bytes
///     REAL32[3] – Vertex 3              - 12 bytes
///     UINT16    – Attribute byte count  - 02 bytes
/// end
/// ```
mod binary {
    use super::*;

    pub fn parse<T: Deserializer>(des: &mut T, progress: Progress) -> Result<Mesh> {
        des.advance_by(80); // skip header
        let tri_count = des.read_u32_le();
        progress.set_total(tri_count as u64);

        let mut verts = HashMap::new();
        let mut faces = Vec::new();
        for i in 0..tri_count {
            progress.set_complete(i as u64);
            des.advance_by(4 * 3);
            faces.push([
                vert_idx(&mut verts, des_vec3f_bin(des)),
                vert_idx(&mut verts, des_vec3f_bin(des)),
                vert_idx(&mut verts, des_vec3f_bin(des)),
            ]);
            des.advance_by(2);
        }

        Ok(finish(verts, faces))
    }
}

/// ```
/// solid name
/// facet normal ni nj nk
///     outer loop
///         vertex v1x v1y v1z
///         vertex v2x v2y v2z
///         vertex v3x v3y v3z
///     endloop
/// endfacet
/// endsolid name
/// ```
mod ascii {
    use super::*;

    pub fn parse<T: Deserializer>(des: &mut T, progress: Progress) -> Result<Mesh> {
        progress.set_total(des.size() as u64);

        let mut verts = HashMap::new();
        let mut faces = Vec::new();

        let mut builder = [Vector3::zeros(); 3];
        let mut component = 9;

        tokenize(des, progress, |token| {
            if component < 9 {
                let Ok(value) = token.parse::<f32>() else {
                    return;
                };

                builder[component / 3][component % 3] = value;
                component += 1;
                return;
            }

            match token {
                "vertex" => component = 0,
                "endloop" => {
                    faces.push([
                        vert_idx(&mut verts, builder[0]),
                        vert_idx(&mut verts, builder[1]),
                        vert_idx(&mut verts, builder[2]),
                    ]);
                }
                _ => {}
            }
        });

        Ok(finish(verts, faces))
    }

    const WHITESPACE: [char; 4] = [' ', '\t', '\r', '\n'];
    fn tokenize<T: Deserializer>(des: &mut T, progress: Progress, mut callback: impl FnMut(&str)) {
        let mut complete = 0;
        let mut carry = String::new();
        loop {
            let next = des.read_bytes(8 * 1024);
            if next.is_empty() && carry.is_empty() {
                break;
            }

            complete += next.len() as u64;
            progress.set_complete(complete);

            let str = carry + str::from_utf8(&next).unwrap();
            let (str, new_carry) = str.rsplit_once(WHITESPACE).unwrap_or(("", &str));
            carry = new_carry.to_owned();

            for token in str.split(WHITESPACE).filter(|x| !x.is_empty()) {
                callback(token);
            }
        }
    }
}

fn vert_idx(verts: &mut HashMap<Vector3<u32>, u32>, vert: Vector3<f32>) -> u32 {
    let size = verts.len() as u32;
    *verts.entry(vert.map(f32::to_bits)).or_insert(size)
}

fn finish(verts: HashMap<Vector3<u32>, u32>, faces: Vec<[u32; 3]>) -> Mesh {
    let mut verts = verts.into_iter().collect::<Vec<_>>();
    verts.sort_by_key(|(_vert, idx)| *idx);
    let verts = (verts.into_iter())
        .map(|(vert, _idx)| vert.map(f32::from_bits))
        .collect();
    Mesh { verts, faces }
}

fn des_vec3f_bin<T: Deserializer>(des: &mut T) -> Vector3<f32> {
    Vector3::new(des.read_f32_le(), des.read_f32_le(), des.read_f32_le())
}
