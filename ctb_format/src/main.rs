use std::{fs, path::PathBuf};

use anyhow::{Ok, Result};
use clap::Parser;
use image::RgbImage;

use common::{
    misc::Run,
    serde::{DynamicSerializer, SliceDeserializer},
};
use ctb_format::{File, LayerDecoder};

#[derive(Parser)]
struct Args {
    path: PathBuf,

    #[clap(short, long)]
    layers: Option<PathBuf>,

    #[clap(short, long)]
    export: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let file = fs::read(&args.path)?;
    let mut des = SliceDeserializer::new(&file);

    let file = File::deserialize(&mut des)?;
    dbg!(&file);

    if let Some(export) = args.export {
        let mut ser = DynamicSerializer::new();
        file.serialize(&mut ser);
        fs::write(export, ser.into_inner())?;
    }

    if let Some(layers) = args.layers {
        for (i, layer) in file.layers.iter().enumerate() {
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
