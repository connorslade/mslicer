use anyhow::{Context, Result};
use common::{progress::Progress, serde::Deserializer};
use nalgebra::Vector3;

use crate::{Mesh, util::tokenize};

pub fn parse<T: Deserializer>(des: &mut T, progress: Progress) -> Result<Mesh> {
    progress.set_total(des.size() as u64);

    let mut mesh = Mesh::default();
    tokenize(des, &['\n', '\r'], progress, |line| {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                let vert = next_vertex(parts).context("Invalid vertex")?;
                mesh.verts.push(vert);
            }
            Some("f") => {
                let face = next_face(parts).context("Invalid face")?;
                mesh.faces.push(face);
            }
            _ => {}
        }
        Ok(())
    })?;

    Ok(mesh)
}

fn next_vertex<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<Vector3<f32>> {
    Some(Vector3::new(
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ))
}

fn next_face<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<[u32; 3]> {
    fn next_idx<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Option<u32> {
        let str = parts.next()?;
        let number = str.split_once('/').map(|x| x.0).unwrap_or(str);
        Some(number.parse::<u32>().ok()? - 1)
    }

    Some([
        next_idx(&mut parts)?,
        next_idx(&mut parts)?,
        next_idx(&mut parts)?,
    ])
}
