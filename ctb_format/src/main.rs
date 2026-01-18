use std::{fs, path::PathBuf};

use anyhow::{Ok, Result};
use clap::Parser;
use image::RgbImage;

use common::{
    misc::Run,
    serde::{DynamicSerializer, SliceDeserializer},
};
use ctb_format::{File, LayerDecoder, PreviewImage};

#[derive(Parser)]
struct Args {
    path: PathBuf,

    #[clap(short, long)]
    preview: Option<PathBuf>,

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

    if let Some(preview) = args.preview {
        fs::create_dir_all(&preview)?;

        let small_preview = preview_to_image(&file.small_preview);
        let large_preview = preview_to_image(&file.large_preview);

        small_preview.save(preview.join("small_preview.png"))?;
        large_preview.save(preview.join("large_preview.png"))?;
    }

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

fn preview_to_image(preview: &PreviewImage) -> RgbImage {
    let size = preview.size();
    let mut out = RgbImage::new(size.x, size.y);

    for y in 0..size.y {
        for x in 0..size.y {
            let pixel = preview.get_pixel(x, y);
            out.put_pixel(x, y, image::Rgb([pixel.x, pixel.y, pixel.z]));
        }
    }

    out
}
