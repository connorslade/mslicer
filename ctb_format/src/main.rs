use std::{fs, path::PathBuf};

use anyhow::{Ok, Result};
use clap::Parser;
use image::RgbImage;

use common::{
    misc::Run,
    serde::{Deserializer, DynamicSerializer},
};
use ctb_format::{file::File, layer::Layer, layer_coding::LayerDecoder};

#[derive(Parser)]
struct Args {
    path: PathBuf,
    layers: Option<PathBuf>,
    export: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let file = fs::read(&args.path)?;
    let mut des = Deserializer::new(&file);

    let file = File::deserialize(&mut des)?;
    dbg!(&file);

    if let Some(export) = args.export {
        let mut ser = DynamicSerializer::new();
        file.serialize(&mut ser);
        fs::write(export, ser.into_inner())?;
    }

    if let Some(layers) = args.layers {
        const PAGE_SIZE: u64 = 1 << 32;
        for (i, layer) in file.layers.iter().enumerate() {
            dbg!(i as f32 / file.layers.len() as f32 * 100.0);
            des.jump_to(
                layer.page_number as usize * PAGE_SIZE as usize + layer.layer_offset as usize,
            );
            let layer = Layer::deserialize(&mut des, file.layer_xor_key, i as u32)?;

            let mut image = RgbImage::new(file.resolution.x, file.resolution.y);

            let mut pixel = 0;
            for Run { length, value } in LayerDecoder::new(&layer.data) {
                let color = image::Rgb([value, value, value]);
                for _ in 0..length {
                    let x = pixel % file.resolution.x;
                    let y = pixel / file.resolution.x;

                    image.put_pixel(x, y, color);
                    pixel += 1;
                }
            }

            image.save(layers.join(format!("layer_{i:03}.png")))?;
        }
    }

    Ok(())
}
