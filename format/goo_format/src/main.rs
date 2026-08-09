use std::{
    fs,
    io::{Write, stdout},
    path::PathBuf,
};

use anyhow::{Result, ensure};
use clap::Parser;
use common::{
    container::rle::png::{ColorType, PngEncoder},
    serde::{DynamicSerializer, SliceDeserializer},
};
use goo_format::{File, LayerDecoder};
use nalgebra::Vector2;

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

    let raw = fs::read(&args.input_file)?;
    let mut des = SliceDeserializer::new(&raw);
    let goo = File::deserialize(&mut des)?;

    if !args.no_header {
        println!("{:#?}", goo.header);
    }

    if let Some(preview) = args.preview {
        fs::create_dir_all(&preview)?;

        let small_preview = goo.header.small_preview.into_image();
        let large_preview = goo.header.big_preview.into_image();

        small_preview.save(preview.join("small_preview.png"))?;
        large_preview.save(preview.join("large_preview.png"))?;
    }

    if let Some(layers) = args.layers {
        fs::create_dir_all(&layers)?;

        println!("Exporting layers as images:\n");
        for (i, layer) in goo.layers.iter().enumerate() {
            print!(
                "\r{}/{} ({:.1}%)",
                i + 1,
                goo.header.layer_count,
                (1.0 + i as f32) / goo.header.layer_count as f32 * 100.0
            );
            stdout().flush()?;

            let decoder = LayerDecoder::new(&layer.data);
            ensure!(
                layer.checksum == decoder.checksum(),
                "Checksum mismatch for layer {i}"
            );

            let resolution =
                Vector2::new(goo.header.x_resolution, goo.header.y_resolution).map(|x| x as u32);

            let mut ser = DynamicSerializer::new();
            let mut encoder = PngEncoder::new(&mut ser, ColorType::Grayscale, resolution);
            encoder.write_image_data(decoder.filter(|x| x.length > 0).collect());
            encoder.write_end();

            let path = layers.join(format!("layer_{i:03}.png"));
            fs::write(path, ser.into_inner())?;
        }
    }

    Ok(())
}
