use std::{
    fs,
    io::{stdout, Write},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use goo_format::{File, LayerDecoder, PreviewImage, Run};
use image::RgbImage;

#[derive(Parser)]
struct Args {
    /// Path to the .goo file
    input_file: PathBuf,

    /// Path to output the small and large preview images
    #[clap(short, long)]
    preview: Option<PathBuf>,

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

    if let Some(preview) = args.preview {
        fs::create_dir_all(&preview)?;

        let small_preview = preview_to_image(&goo.header.small_preview);
        let large_preview = preview_to_image(&goo.header.big_preview);

        small_preview.save(preview.join("small_preview.png"))?;
        large_preview.save(preview.join("large_preview.png"))?;
    }

    if let Some(layers) = args.layers {
        fs::create_dir_all(&layers)?;

        println!("Exporting layers as images:\n");
        for (i, layer) in goo.layers.iter().enumerate() {
            print!(
                "\r{i}/{} ({:.1}%)",
                goo.header.layer_count,
                i as f32 / goo.header.layer_count as f32 * 100.0
            );
            stdout().flush()?;

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

fn preview_to_image<const WIDTH: usize, const HEIGHT: usize>(
    preview: &PreviewImage<WIDTH, HEIGHT>,
) -> RgbImage {
    let mut out = RgbImage::new(WIDTH as u32, HEIGHT as u32);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let (r, g, b) = preview.get_pixel(x, y);
            let color = image::Rgb([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]);
            out.put_pixel(x as u32, y as u32, color);
        }
    }

    out
}
