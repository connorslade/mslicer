use common::{
    misc::{VectorLayer, VectorSliceResult},
    serde::Serializer,
};
use nalgebra::Vector2;
use svg::{
    node::element::{Polygon, Rectangle},
    Document,
};

pub struct SvgFile {
    layers: Vec<VectorLayer>,
    area: Vector2<u32>,
}

impl SvgFile {
    pub fn new(result: VectorSliceResult) -> Self {
        Self {
            layers: result.layers,
            area: result.slice_config.platform_resolution,
        }
    }

    pub fn layer_count(&self) -> u32 {
        self.layers.len() as u32
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        let sides = self.layer_count().isqrt() as usize + 1;
        let (width, height) = (self.area.x as f32, self.area.y as f32);
        let size = (width * sides as f32, height * sides as f32);

        let mut svg = Document::new()
            .set("viewBox", (0, 0, size.0, size.1))
            .set("width", format!("{}mm", size.0))
            .set("height", format!("{}mm", size.1));

        for (idx, layer) in self.layers.iter().enumerate() {
            let (x, y) = (idx % sides, idx / sides);
            let offset = Vector2::new(x as f32 * width, y as f32 * height);

            svg = svg.add(
                Rectangle::new()
                    .set("x", offset.x)
                    .set("y", offset.y)
                    .set("width", width)
                    .set("height", height)
                    .set("fill", "none")
                    .set("stroke", "gray")
                    .set("stroke-width", "0.1"),
            );

            for polygon in layer.polygons.iter() {
                let points = polygon
                    .iter()
                    .map(|x| x + offset)
                    .map(|x| (x.x, x.y))
                    .collect::<Vec<_>>();
                let poly = Polygon::new()
                    .set("points", points)
                    .set("fill", "none")
                    .set("stroke", "black")
                    .set("stroke-width", "0.1");
                svg = svg.add(poly);
            }
        }

        ser.write_bytes(svg.to_string().as_bytes());
    }
}
