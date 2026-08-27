use common::{
    progress::Progress,
    slice::{Layer, SliceConfig},
};
use nalgebra::{Matrix4, Vector2, Vector3};
use slicer::{
    mesh::Mesh,
    slicer::raster::{self, Segment},
};

#[derive(Clone)]
pub struct Graphics3D {
    pub meshes: Vec<Mesh>,
    pub angles: u32,
}

impl Default for Graphics3D {
    fn default() -> Self {
        Self {
            meshes: Vec::new(),
            angles: 30,
        }
    }
}

impl Graphics3D {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(
        &self,
        config: &SliceConfig,
        progress: &Progress,
        camera: impl Fn(f32, u32) -> (Matrix4<f32>, Vector3<f32>),
    ) -> Vec<Layer> {
        let n = self.angles.max(1);
        progress.set_total(n as u64);

        let platform_size = config.platform_size.map(|x| x.raw());
        let aspect = platform_size.x / platform_size.y;

        (0..n)
            .map(|i| {
                let (view_projection, light) = camera(aspect, i);

                let layer = render(config, self.meshes.iter(), view_projection, light, i);
                progress.add_complete(1);

                layer
            })
            .collect()
    }
}

pub fn render<'a>(
    config: &SliceConfig,
    meshes: impl Iterator<Item = &'a Mesh>,
    view_projection: Matrix4<f32>,
    light: Vector3<f32>,
    i: u32,
) -> Layer {
    let mut faces = Vec::new();
    let mut min_depth = f32::INFINITY;
    let mut max_depth = f32::NEG_INFINITY;

    let platform = config.platform_resolution * config.supersample as u32;

    for mesh in meshes {
        for (i, face) in mesh.faces().iter().enumerate() {
            let verts = face
                .map(|x| mesh.transform(&mesh.vertices()[x as usize]))
                .map(|x| view_projection * x.push(1.0));
            if verts.iter().any(|x| x.w < 0.1) {
                continue;
            }

            let depth = (verts[0].w + verts[1].w + verts[2].w) / 3.0;
            min_depth = min_depth.min(depth);
            max_depth = max_depth.max(depth);

            let [a, b, c] = verts.map(|x| x / x.w).map(|p| {
                Vector2::new(
                    (p.x * 0.5 + 0.5) * platform.x as f32,
                    (1.0 - (p.y * 0.5 + 0.5)) * platform.y as f32,
                )
            });

            let signed_area = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
            if signed_area > 0.0 {
                continue;
            }

            let normal = mesh.transform_normal(&mesh.normal(i)).normalize();

            let diffuse = normal.dot(&light).max(0.0);
            let reflect_dir = (-light) - 2.0 * normal.dot(&(-light)) * normal;
            let specular = light.dot(&reflect_dir).max(0.0).powi(32);
            let intensity = ((diffuse + specular + 0.1) * 255.0) as u8;

            faces.push((a, b, c, depth, intensity));
        }
    }

    let mut segments = Vec::new();
    for (a, b, c, avg_depth, intensity) in faces {
        let norm = (max_depth - avg_depth) / (max_depth - min_depth);
        let depth = (1.0 + norm * 254.0) as u8;

        for segment in [[a, b], [b, c], [c, a]] {
            segments.push(Segment {
                endpoints: segment,
                entering: segment[1].y < segment[0].y,
                priority: depth,
                exposure: intensity,
            });
        }
    }

    let runs = raster::layer(
        config.supersample,
        config.platform_resolution,
        segments.into_iter(),
    );

    Layer::new(
        runs,
        config.default_height(i),
        config.exposure_config(i).into_owned(),
    )
}
