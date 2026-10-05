use std::{env, fs};

use anyhow::{Context, Ok, Result};
use common::{
    container::rle::png::{ColorType, PngEncoder},
    serde::{DynamicSerializer, SliceDeserializer},
};
use ctb_format::{shared::LayerDecoder, v4::File};

fn main() -> Result<()> {
    let path = env::args().skip(1).next().context("No path supplied")?;
    let file = fs::read(&path)?;
    let mut des = SliceDeserializer::new(&file);

    let file = File::deserialize(&mut des)?;
    println!("{file:#?}");

    let layer_count = file.layers.len();
    for (i, layer) in file.layers.into_iter().enumerate() {
        let decoder = LayerDecoder::new(&layer.data);
        let runs = decoder.filter(|x| x.length > 0).collect();

        let mut ser = DynamicSerializer::new();
        let mut encoder = PngEncoder::new(&mut ser, ColorType::Grayscale, file.resolution);
        encoder.write_image_data(runs);
        encoder.write_end();

        fs::write(format!("working/layers/layer-{i}.png"), ser.into_inner())?;
        print!("\r{}/{layer_count}", i + 1);
    }

    println!();
    Ok(())
}
