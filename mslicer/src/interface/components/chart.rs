use std::{borrow::Cow, f32::consts::TAU};

use common::color::START_COLOR;
use egui::{Align2, Color32, FontId, Mesh, Sense, Ui, Vec2, vec2};
use tools::misc::circle_points;

pub struct PieChart<'a> {
    size: f32,
    sections: Vec<(Cow<'a, str>, f32)>,
    total: f32,
}

impl<'a> PieChart<'a> {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            sections: Vec::new(),
            total: 0.0,
        }
    }

    pub fn slice_mut(&mut self, name: impl Into<Cow<'a, str>>, value: f32) {
        self.sections.push((name.into(), value));
        self.total += value;
    }

    pub fn slice(mut self, name: impl Into<Cow<'a, str>>, value: f32) -> Self {
        self.slice_mut(name, value);
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::empty());
        let p = ui.painter_at(rect);

        let base = rect.min + Vec2::splat(self.size / 2.0);

        let mut text = vec![None; self.sections.len()];

        let mut mesh = Mesh::default();
        let mut theta = 0.0;
        let mut n = 0;
        for (i, (_name, value)) in self.sections.iter().enumerate() {
            let shift = i as f32 * 1.0;
            let color = START_COLOR.hue_shift(shift).to_linear_srgb();
            let color = Color32::from_rgb(
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
            );

            let delta_theta = TAU * *value / self.total;
            mesh.colored_vertex(base, color);

            let points = circle_points(0.1, self.size as f64 / 2.0);
            for i in 0..points {
                let t = i as f32 / (points - 1) as f32;
                let theta = theta + delta_theta * t;
                let point = vec2(theta.cos(), theta.sin()) * self.size / 2.0;
                mesh.colored_vertex(base + point, color);
            }

            if delta_theta > 0.0 {
                let theta_half = theta + delta_theta / 2.0;
                let center = vec2(theta_half.cos(), theta_half.sin()) * self.size / 4.0;
                text[i] = Some(base + center);
            }

            (1..points).for_each(|i| mesh.add_triangle(n, n + i, n + i + 1));
            theta += delta_theta;
            n += points + 1;
        }

        p.add(mesh);

        for ((name, _value), pos) in self.sections.iter().zip(text.iter()) {
            let Some(pos) = pos else { continue };
            p.text(
                *pos,
                Align2::CENTER_CENTER,
                name,
                FontId::proportional(12.0),
                Color32::WHITE,
            );
        }
    }
}
