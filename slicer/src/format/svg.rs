use common::{
    misc::{VectorLayer, VectorSliceResult},
    serde::Serializer,
};
use nalgebra::Vector3;
use svg::{node::element::Polygon, Document};

pub struct SvgFile {
    layers: Vec<VectorLayer>,

    volume: Vector3<f32>,
}

impl SvgFile {
    pub fn new(result: VectorSliceResult) -> Self {
        Self {
            layers: result.layers,

            volume: result.slice_config.platform_size,
        }
    }

    pub fn layer_count(&self) -> u32 {
        self.layers.len() as u32
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        let (width, height) = (self.volume.x, self.volume.y);
        let mut svg = Document::new()
            .set("viewBox", (0, 0, width, height))
            .set("width", format!("{width}mm"))
            .set("height", format!("{height}mm"));

        for layer in self.layers.iter().take(1) {
            let points = layer.points.iter().map(|x| (x.x, x.y)).collect::<Vec<_>>();
            let poly = Polygon::new()
                .set("points", points)
                .set("fill", "none")
                .set("stroke", "#0068FF")
                .set("stroke-width", "0.1");
            svg = svg.add(poly);
        }

        ser.write_bytes(svg.to_string().as_bytes());
    }
}
