// Resources:
// - https://www.kabusa.com/frameset.htm?/needbelt.htm
// - https://en.wikipedia.org/wiki/Archimedean_spiral

use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, SQRT_2, TAU},
    fs,
};

use common::units::{Micrometers, Milimeter, Milimeters};
use nalgebra::{Rotation3, Vector2, Vector3};
use slicer::{builder::MeshBuilder, mesh::Mesh};

use crate::printed_circuit_board::polygons::Polygons;

const FRAC_3_PI_4: f32 = 3.0 * FRAC_PI_4;

pub struct PhonographRecord {
    outer_radius: Milimeters,
    inner_radius: Milimeters,
    pitch: Milimeters, // must be > width
    groove_resolution: u32,

    width: Milimeters,
    corner_radius: Milimeters,
    profile_resolution: u32,
}

impl PhonographRecord {
    pub fn debug_profile(&self) {
        let mut geo = Polygons::new();

        let profile = self.profile();
        let width = self.width.get::<Milimeter>() as f64;
        geo.rect([
            Vector2::new(-width / 2.0, 0.0),
            Vector2::new(width / 2.0, width / 2.0),
        ]);
        geo.trace(profile.iter().map(|x| x.xz().cast()).collect(), None);

        geo.nonuniform_scale_mut(Vector2::repeat(50.0));
        fs::write("debug.svg", geo.svg()).unwrap();
    }

    pub fn generate(&self) -> Mesh {
        let mut builder = MeshBuilder::new();

        let profile = self.profile();
        let points = profile.len() as u32;
        let b = self.pitch.get::<Milimeter>() / TAU;
        let outer_radius = self.outer_radius.get::<Milimeter>();
        let inner_radius = self.inner_radius.get::<Milimeter>();

        for i in 0..self.groove_resolution {
            let t = i as f32 / (self.groove_resolution - 1) as f32;
            let theta = (outer_radius - inner_radius) / b * t;
            let rotation = Rotation3::new(Vector3::z() * theta);

            let unit = Vector2::new(theta.cos(), theta.sin());
            let r = outer_radius - b * theta;
            let offset = unit * r;

            for point in profile.iter() {
                let vertex = rotation * point + offset.push(0.0);
                builder.add_vertex(vertex);
            }

            if i < self.groove_resolution - 1 {
                for j in 0..points - 1 {
                    let base = i * points;
                    let (a, b) = (base + j, base + j + 1);
                    let (c, d) = (a + points, b + points);
                    builder.add_quad([a, c, b, d]);
                }
            }
        }

        builder.add_cylinder(
            (Vector3::zeros(), Vector3::z() * -2.0),
            (outer_radius, outer_radius),
            1000,
        );

        builder.build()
    }

    fn profile(&self) -> Vec<Vector3<f32>> {
        let mut points = Vec::new();

        let half_width = self.width.get::<Milimeter>() / 2.0;
        let radius = self.corner_radius.get::<Milimeter>();
        points.push(Vector3::new(-half_width, 0.0, 0.0));

        if self.profile_resolution <= 1 || radius == 0.0 {
            points.push(Vector3::new(0.0, 0.0, -half_width));
        } else {
            for i in 0..self.profile_resolution {
                let t = i as f32 / (self.profile_resolution - 1) as f32;
                let theta = -FRAC_3_PI_4 + FRAC_PI_2 * t;
                let point = Vector3::new(theta.cos(), 0.0, theta.sin()) * radius
                    + Vector3::z() * (radius * SQRT_2 - half_width);
                points.push(point);
            }
        }

        points.push(Vector3::new(half_width, 0.0, 0.0));
        points
    }
}

impl Default for PhonographRecord {
    fn default() -> Self {
        Self {
            outer_radius: Milimeters::new(60.0),
            inner_radius: Milimeters::new(50.0),

            // pitch: Micrometers::new(125.0).convert(),
            pitch: Micrometers::new(220.0).convert(),
            groove_resolution: 10_000,

            // width: Micrometers::new(56.0).convert(),
            // corner_radius: Micrometers::new(6.0).convert(),
            width: Micrometers::new(160.0).convert(),
            corner_radius: Micrometers::new(40.0).convert(),
            profile_resolution: 10,
        }
    }
}
