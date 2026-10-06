use std::{env, fs};

use anyhow::{Context, Ok, Result};
use common::{
    container::rle::png::{ColorType, PngEncoder},
    serde::{DynamicSerializer, SliceDeserializer},
};
use ctb_format::{shared::LayerDecoder, v4::File};

fn main() -> Result<()> {
    let path = env::args().skip(1).next().context("No path supplied")?;
    let layers = env::args().skip(2).any(|x| x == "--layers");
    let reencode = env::args().skip(2).any(|x| x == "--reencode");

    let file = fs::read(&path)?;
    let mut des = SliceDeserializer::new(&file);

    let file = File::deserialize(&mut des)?;
    println!("{file:#?}");

    if layers {
        for (i, layer) in file.layers.iter().enumerate() {
            let decoder = LayerDecoder::new(&layer.data);
            let runs = decoder.filter(|x| x.length > 0).collect();

            let mut ser = DynamicSerializer::new();
            let mut encoder = PngEncoder::new(&mut ser, ColorType::Grayscale, file.resolution);
            encoder.write_image_data(runs);
            encoder.write_end();

            fs::write(format!("working/layers/layer-{i}.png"), ser.into_inner())?;
            println!("{layer:?}");
        }
    }

    if reencode {
        let mut ser = DynamicSerializer::new();
        file.serialize(&mut ser);
        fs::write("working/CTBv4-rencode.ctb", ser.inner_mut())?;
    }

    Ok(())
}
