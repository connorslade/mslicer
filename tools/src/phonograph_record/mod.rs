// Resources:
// - https://www.kabusa.com/frameset.htm?/needbelt.htm
// - https://en.wikipedia.org/wiki/Archimedean_spiral

use std::{
    f32::consts::{PI, TAU},
    fs::File,
    io::BufReader,
    path::PathBuf,
};

use anyhow::Result;
use common::units::{Micrometers, Milimeter, Milimeters};
use nalgebra::{Rotation3, Vector2, Vector3};
use slicer::{
    builder::{MeshBuilder, orthogonal_basis},
    mesh::Mesh,
};

use crate::phonograph_record::audio::AudioBuffer;

mod audio;

#[derive(Clone)]
pub struct PhonographRecord {
    pub outer_radius: Milimeters,
    pub inner_radius: Milimeters,
    pub thickness: Milimeters,

    pub pitch: Milimeters, // must be > width
    pub width: Milimeters,
    pub groove_resolution: f32,
    pub rpm: f32,
    pub modulation: f32,

    pub audio: PathBuf,
}

impl PhonographRecord {
    // todo: generate manifold mesh
    pub fn generate(&self) -> Result<Mesh> {
        let reader = BufReader::new(File::open(&self.audio)?);
        let audio = AudioBuffer::load(reader)?;

        let mut builder = MeshBuilder::new();

        let pitch = self.pitch.get::<Milimeter>();
        let outer_radius = self.outer_radius.get::<Milimeter>();
        let inner_radius = self.inner_radius.get::<Milimeter>();
        let thickness = self.thickness.get::<Milimeter>();

        let duration = (outer_radius - inner_radius) / pitch * 60.0 / self.rpm;
        let resolution = (self.groove_resolution * duration).round() as u32;
        let b = pitch / TAU;

        for i in 0..resolution {
            let t = i as f32 / (resolution - 1) as f32;
            let theta = (outer_radius - inner_radius) / b * t;
            let rotation = Rotation3::new(Vector3::z() * theta);

            let unit = Vector2::new(theta.cos(), theta.sin());
            let r = outer_radius - b * theta;
            let offset = unit * r;

            let (l, r) = audio.get(t * duration);
            let profile = self.profile((self.pitch - self.width) * 0.5 * self.modulation, l, r);
            for point in profile.iter() {
                let vertex = rotation * point + offset.push(thickness);
                builder.add_vertex(vertex);
            }

            if i < resolution - 1 {
                for j in 0..2 {
                    let base = i * 3;
                    let (a, b) = (base + j, base + j + 1);
                    let (c, d) = (a + 3, b + 3);
                    builder.add_quad([a, c, b, d]);
                }
            }
        }

        for j in 1..2 {
            builder.add_face([0, j, j + 1]);
        }

        let base = (resolution - 1) * 3;
        for j in 1..2 {
            builder.add_face([base, base + j + 1, base + j]);
        }

        let radius = outer_radius + pitch;
        builder._add_cylinder(
            (Vector3::zeros(), Vector3::z() * thickness),
            (radius, radius),
            (false, true),
            1000,
        );

        add_cylinder_inner(&mut builder, Vector3::zeros(), thickness, 3.62, 100);

        Ok(builder.build())
    }

    fn profile(&self, modulation: Milimeters, l: f32, r: f32) -> Vec<Vector3<f32>> {
        let mut points = Vec::new();

        let modulation = modulation.get::<Milimeter>();
        let half_width = self.width.get::<Milimeter>() / 2.0;

        let (l, r) = (l * modulation, r * modulation);

        points.push(Vector3::new(-half_width + l, 0.0, 0.0));
        points.push(Vector3::new((l + r) / 2.0, 0.0, -half_width));
        points.push(Vector3::new(half_width + r, 0.0, 0.0));
        points
    }
}

impl Default for PhonographRecord {
    fn default() -> Self {
        Self {
            outer_radius: Milimeters::new(60.0),
            inner_radius: Milimeters::new(50.0),
            thickness: Milimeters::new(2.0),

            pitch: Micrometers::new(150.0).convert(),
            width: Micrometers::new(80.0).convert(),
            groove_resolution: 20_000.0,
            rpm: 33.3333,
            modulation: 2.0,

            audio: PathBuf::new(),
        }
    }
}

fn add_cylinder_inner(
    builder: &mut MeshBuilder,
    bottom: Vector3<f32>,
    height: f32,
    radius: f32,
    precision: u32,
) {
    let (a, b) = (bottom, bottom + Vector3::z() * height);
    let [u, v] = orthogonal_basis((a - b).normalize());

    let mut first = None;
    let mut last = None;
    for i in 0..(precision * 2) {
        let angle = i as f32 / precision as f32 * PI;
        let normal = u * angle.sin() + v * angle.cos();

        let top = builder.add_vertex(b + normal * radius);
        let bottom = builder.add_vertex(a + normal * radius);

        if let Some((last_top, last_bottom)) = last {
            builder.add_quad_flipped([last_bottom, last_top, bottom, top]);
        }

        last = Some((top, bottom));
        first.is_none().then(|| first = last);
    }

    if let Some((last_top, last_bottom)) = last
        && let Some((first_top, first_bottom)) = first
    {
        builder.add_quad_flipped([last_bottom, last_top, first_bottom, first_top]);
    }
}
