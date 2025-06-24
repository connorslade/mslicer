use common::serde::Serializer;
use nalgebra::Vector2;
use svg::{
    node::element::{tag::Polygon, Polygon},
    Document,
};

pub struct SvgFile {
    layers: Vec<VectorLayer>,
}

pub struct VectorLayer {
    points: Vector2<f32>,
}

impl SvgFile {
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        let mut svg = Document::new();

        for layer in self.layers.iter().take(1) {
            let poly = Polygon::new().set(
                "points",
                layer.points.iter().map(|x| (x.x, x.y)).collect::<Vec<_>>(),
            );
            svg = svg.add(poly);
        }

        ser.write_bytes(svg.to_string().as_bytes());
    }
}
