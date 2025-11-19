use std::fs;

use anyhow::{Ok, Result};

use common::{misc::Run, serde::Deserializer};
use ctb_format::{file::File, layer::Layer, layer_coding::LayerDecoder};
use image::RgbImage;

fn main() -> Result<()> {
    let file = fs::read("Skull_v1.stl_0.05_2.5_2025_11_18_19_44_00.ctb")?;
    let mut des = Deserializer::new(&file);

    let file = File::deserialize(&mut des)?;
    dbg!(&file);

    const PAGE_SIZE: u64 = 1 << 32;
    for (i, layer) in file.layers.iter().skip(100).take(1).enumerate() {
        des.jump_to(layer.page_number as usize * PAGE_SIZE as usize + layer.layer_offset as usize);
        let layer = Layer::deserialize(&mut des, file.settings.layer_xor_key, i as u32)?;

        let path = format!("layer_{i:03}.png");
        let mut image = RgbImage::new(file.settings.resolution.x, file.settings.resolution.y);

        let mut pixel = 0;
        for Run { length, value } in LayerDecoder::new(&layer.data) {
            let color = image::Rgb([value, value, value]);
            for _ in 0..length {
                let x = pixel % file.settings.resolution.x;
                let y = pixel / file.settings.resolution.y;

                image.put_pixel(x, y, color);
                pixel += 1;
            }
        }

        image.save(path)?;
    }

    Ok(())
}
