use std::{
    fs,
    io::{Write, stdout},
    path::PathBuf,
};

use anyhow::{Ok, Result};
use clap::Parser;
use image::RgbImage;

use common::{
    container::rle::png::{ColorType, PngEncoder, PngInfo},
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
    println!("{file:#?}");

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
        fs::create_dir_all(&layers)?;

        println!("Exporting layers as images:\n");
        for (i, layer) in file.layers.iter().enumerate() {
            let count = file.layers.len();
            print!(
                "\r{}/{count} ({:.1}%)",
                i + 1,
                (1.0 + i as f32) / count as f32 * 100.0
            );
            stdout().flush()?;

            let header = PngInfo {
                width: file.resolution.x,
                height: file.resolution.y,
                bit_depth: 8,
                color_type: ColorType::Grayscale,
            };

            let mut ser = DynamicSerializer::new();
            let mut encoder = PngEncoder::new(&mut ser, &header, 1);
            encoder.write_image_data(LayerDecoder::new(&layer.data).collect());
            encoder.write_end();

            let path = layers.join(format!("layer_{i:03}.png"));
            fs::write(path, ser.into_inner())?;
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
