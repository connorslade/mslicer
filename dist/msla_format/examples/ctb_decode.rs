use std::fs;

use msla_format::{
    container::rle::png::{ColorType, PngEncoder},
    ctb,
    serde::{DynamicSerializer, SliceDeserializer},
};

fn main() {
    let bytes = fs::read("out.ctb").unwrap();
    let mut des = SliceDeserializer::new(&bytes);
    let file = ctb::File::deserialize(&mut des).unwrap();
    println!("{file:?}");

    for (i, layer) in file.layers.iter().enumerate() {
        let decoder = ctb::LayerDecoder::new(&layer.data);

        let mut ser = DynamicSerializer::new();
        let mut png = PngEncoder::new(&mut ser, ColorType::Grayscale, file.resolution);
        png.write_image_data(decoder.collect());
        png.write_end();

        fs::write(format!("layer_{i}.png"), ser.into_inner()).unwrap();
    }
}
