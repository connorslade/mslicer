// References:
// - https://en.wikipedia.org/wiki/Archimedean_spiral
// - https://en.wikipedia.org/wiki/Phonograph_record
// - https://en.wikipedia.org/wiki/RIAA_equalization
// - https://www.kabusa.com/frameset.htm?/needbelt.htm

use std::{f32::consts::TAU, fs::File, io::BufReader, path::PathBuf};

use anyhow::Result;
use common::{
    progress::Progress,
    units::{Micrometers, Milimeter, Milimeters},
};
use nalgebra::{Rotation3, Vector2, Vector3};
use slicer::{builder::MeshBuilder, mesh::Mesh};

use crate::phonograph_record::{
    audio::{AudioBuffer, Channels, Equalization},
    mesh::add_disk,
};

pub mod audio;
mod mesh;

// todo:
// - allow configuring inner hole size
// - clip audio gain to [0, 1]
// - ensure groove can't self intersect (pitch * modulation > width)
// - generate closed mesh (optionally)
// - either implement RIAA or remove the option

#[derive(Clone)]
pub struct PhonographRecord {
    pub outer_radius: Milimeters,
    pub inner_radius: Milimeters,
    pub hole_radius: Milimeters,
    pub thickness: Milimeters,
    pub watertight: bool,

    pub pitch: Milimeters, // must be >width
    pub width: Milimeters,
    pub groove_resolution: f32,
    pub rpm: f32,
    pub modulation: f32,

    pub audio: PathBuf,
    pub channels: Channels,
    pub equalization: Equalization,
}

impl PhonographRecord {
    pub fn generate(&self, progress: &Progress) -> Result<Mesh> {
        let reader = BufReader::new(File::open(&self.audio)?);
        let audio = AudioBuffer::load(reader, self.channels)?;

        let mut builder = MeshBuilder::new();
        let (mut hole_a, mut hole_b) = (Vec::new(), Vec::new());

        let pitch = self.pitch.get::<Milimeter>();
        let outer_radius = self.outer_radius.get::<Milimeter>();
        let inner_radius = self.inner_radius.get::<Milimeter>();
        let hole_radius = self.hole_radius.get::<Milimeter>();
        let thickness = self.thickness.get::<Milimeter>();

        let delta_radius = outer_radius - inner_radius;
        let duration = delta_radius / pitch * 60.0 / self.rpm;
        let resolution = (self.groove_resolution * duration).round() as u32;
        let n = (resolution - 1) as f32;
        let points_per_disk = (n * pitch / delta_radius) as u32;
        let b = pitch / TAU;

        progress.set_total(resolution as u64);
        for i in 0..resolution {
            let t = i as f32 / n;
            let theta = delta_radius / b * t;

            let rotation = Rotation3::new(Vector3::z() * theta);
            let unit = Vector2::new(theta.cos(), theta.sin());
            let r = outer_radius - b * theta;
            let offset = unit * r;

            let (l, r) = audio.get(t * duration);
            let profile = self.profile((self.pitch - self.width) * 0.5 * self.modulation, l, r);
            for (i, point) in profile.iter().enumerate() {
                let vertex = rotation * point + offset.push(thickness);
                let idx = builder.add_vertex(vertex);

                if i == 0 {
                    hole_a.push(idx);
                } else if i + 1 == profile.len() {
                    hole_b.push(idx);
                }
            }

            if i < resolution - 1 {
                for j in 0..2 {
                    let base = i * 3;
                    let (a, b) = (base + j, base + j + 1);
                    let (c, d) = (a + 3, b + 3);
                    builder.add_quad([a, c, b, d]);
                }

                let across = i + points_per_disk;
                if self.watertight && across + 1 < resolution {
                    let (a, c) = (i * 3, across * 3);
                    let (b, d) = (a + 3, c + 3);
                    builder.add_quad_flipped([a, b, c + 2, d + 2]);
                }
            }

            progress.set_complete(i as u64);
        }

        // grove end caps
        let base = (resolution - 1) * 3;
        builder.add_face([0, 2, 1]);
        builder.add_face([base, base + 2, base + 1]);

        add_disk(
            &mut builder,
            thickness,
            outer_radius + pitch,
            hole_radius,
            0.1,
        );

        // if self.watertight {
        //     // fill to outer cylinder
        //     for i in 0..points_per_disk {
        //         let t = i as f32 / points_per_disk as f32;
        //         let (a, b) = (i * 3 + 2, (i + 1) * 3 + 2);
        //         let across = x + (t * p as f32) as u32 * 4;

        //         builder.add_face([a, across, b]);
        //     }
        // }

        progress.set_finished();
        Ok(builder.build())
    }

    fn profile(&self, modulation: Milimeters, l: f32, r: f32) -> [Vector3<f32>; 3] {
        let modulation = modulation.get::<Milimeter>();
        let half_width = self.width.get::<Milimeter>() / 2.0;
        let (l, r) = (l * modulation, r * modulation);

        [
            Vector3::new(-half_width + l, 0.0, 0.0),
            Vector3::new((l + r) / 2.0, 0.0, -half_width),
            Vector3::new(half_width + r, 0.0, 0.0),
        ]
    }
}

impl Default for PhonographRecord {
    fn default() -> Self {
        Self {
            outer_radius: Milimeters::new(60.0),
            inner_radius: Milimeters::new(50.0),
            hole_radius: Milimeters::new(3.62),
            thickness: Milimeters::new(2.0),
            watertight: true,

            pitch: Micrometers::new(150.0).convert(),
            width: Micrometers::new(80.0).convert(),
            groove_resolution: 20_000.0,
            rpm: 33.3333,
            modulation: 2.0,

            audio: PathBuf::new(),
            channels: Default::default(),
            equalization: Default::default(),
        }
    }
}
