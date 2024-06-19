use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use goo_format::{File, LayerDecoder, Run};
use image::RgbImage;

#[derive(Parser)]
struct Args {
    /// Path to the .goo file
    input_file: PathBuf,

    /// Path to output each layer as an image
    #[clap(short, long)]
    layers: Option<PathBuf>,

    /// Do not print the header information
    #[clap(short, long)]
    no_header: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let raw_goo = fs::read(&args.input_file)?;
    let goo = File::deserialize(&raw_goo)?;

    if !args.no_header {
        println!("{:#?}", goo.header);
    }

    if let Some(layers) = args.layers {
        fs::create_dir_all(&layers)?;

        for (i, layer) in goo.layers.iter().enumerate() {
            let decoder = LayerDecoder::new(&layer.data);
            let mut pixel = 0;

            if layer.checksum != decoder.checksum() {
                eprintln!("WARN: Checksum mismatch for layer {}", i);
            }

            let path = layers.join(format!("layer_{:03}.png", i));
            let mut image = RgbImage::new(
                goo.header.x_resolution as u32,
                goo.header.y_resolution as u32,
            );

            for Run { length, value } in decoder {
                for _ in 0..length {
                    let x = pixel % goo.header.x_resolution as u32;
                    let y = pixel / goo.header.x_resolution as u32;

                    image.put_pixel(x, y, image::Rgb([value, value, value]));

                    pixel += 1;
                }
            }

            image.save(path)?;
        }
    }

    Ok(())
}
