use std::{
    io::{Cursor, Read, Seek, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use common::{progress::Progress, serde::Serializer, slice::SliceResult, units::Milimeter};
use image::DynamicImage;
use serde::Serialize;
use zip::{ZipArchive, ZipWriter, write::FileOptions};

use crate::{
    Layer, decode_png, encode_png, read_to_bytes,
    types::{
        Color, LayerInfo, Meta, Options, Plate, Profile, SHIELD_AFTER_LAYER, SHIELD_BEFORE_LAYER,
    },
};

pub struct File {
    pub meta: Meta,
    pub plate: Plate,
    pub options: Options,
    pub profile: Profile,
    pub preview: DynamicImage,

    pub layer_info: Vec<LayerInfo>,
    pub layers: Vec<Vec<u8>>, // layers encoded as png
}

impl File {
    pub fn from_slice_result(result: SliceResult<Layer>) -> Self {
        let (layers, layer_info): (Vec<_>, Vec<_>) =
            result.layers.into_iter().map(|x| (x.inner, x.info)).unzip();

        let config = result.slice_config;
        let pixel_size = (config.platform_size.xy())
            .map(|x| x.get::<Milimeter>())
            .component_div(&config.platform_resolution.cast());

        let timestamp = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap()).as_secs();

        Self {
            meta: Meta {
                version: "0.4.0".into(),
                ..Default::default()
            },
            plate: Plate {
                processed: true,
                total_solid_area: Default::default(), // ←
                layers_count: layers.len() as u32,
                x_min: Default::default(), // ←
                x_max: Default::default(), // ←
                y_min: Default::default(), // ←
                y_max: Default::default(), // ←
                z_min: Default::default(), // ←
                z_max: Default::default(), // ←
                ..Default::default()
            },
            options: Options {
                p_width: config.platform_resolution.x,
                p_height: config.platform_resolution.y,
                thickness: config.slice_height.convert(),
                x_offset: config.platform_resolution.x / 2,
                y_offset: config.platform_resolution.y / 2,
                x_pixel_size: pixel_size.x,
                y_pixel_size: pixel_size.y,
                ignore_mask: 1,
                image_mirror: 1,
                display_controller: 1,
                support_layer_number: config.first_layers,
                fill_color: "#ffffff".into(),
                blank_color: "#000000".into(),
                fill_color_rgb: Color::repeat(255),
                blank_color_rgb: Color::repeat(0),
                ..Default::default()
            },
            profile: Profile {
                title: "mslicer Config".into(),
                depth: config.slice_height.convert(),
                support_depth: config.slice_height.convert(),
                transitional_layer: config.transition_layers,
                updated: timestamp as u32,
                cure_time: config.exposure_config.exposure_time,
                support_cure_time: config.first_exposure_config.exposure_time,
                fill_color: "#ffffff".into(),
                blank_color: "#000000".into(),
                ignore_mask: 1,
                shield_before_layer: SHIELD_BEFORE_LAYER.into(),
                shield_after_layer: SHIELD_AFTER_LAYER.into(),
                ..Default::default()
            },
            preview: Default::default(), // overwritten later

            layer_info,
            layers,
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T, progress: Progress) -> Result<()> {
        let mut bytes = Vec::new();
        let mut zip = ZipWriter::new(Cursor::new(&mut bytes));

        fn serialize_file<W, T>(zip: &mut ZipWriter<W>, name: &str, value: &T) -> Result<()>
        where
            W: Write + Seek,
            T: Serialize,
        {
            zip.start_file(name, FileOptions::DEFAULT)?;
            serde_json::to_writer(zip, &value)?;
            Ok(())
        }

        serialize_file(&mut zip, "meta.json", &self.meta)?;
        serialize_file(&mut zip, "info.json", &self.layer_info)?;
        serialize_file(&mut zip, "plate.json", &self.plate)?;
        serialize_file(&mut zip, "profile.json", &self.profile)?;

        // The actual NanoDLP software seems to work with options.json, but this
        // is not currently recognized by UVTools like slicer.json is. Not sure
        // whats going on here, but to maximize compatibility Ill just output
        // both?
        serialize_file(&mut zip, "options.json", &self.options)?;
        serialize_file(&mut zip, "slicer.json", &self.options)?;

        zip.start_file("3d.png", FileOptions::DEFAULT)?;
        zip.write_all(&encode_png(&self.preview)?)?;

        progress.set_total(self.layers.len() as u64);
        for (i, layer) in self.layers.iter().enumerate() {
            progress.set_complete(i as u64);
            zip.start_file(format!("{}.png", i + 1), FileOptions::DEFAULT)?;
            zip.write_all(layer)?;
        }

        drop(zip);
        ser.write_bytes(&bytes);
        Ok(())
    }

    pub fn deserialize<T: Read + Seek>(reader: T) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let meta = (zip.by_name("meta.json").ok())
            .map(serde_json::from_reader::<_, Meta>)
            .transpose()?
            .unwrap_or_default();
        let layer_info = serde_json::from_reader::<_, Vec<LayerInfo>>(zip.by_name("info.json")?)?;
        let plate = serde_json::from_reader::<_, Plate>(zip.by_name("plate.json")?)?;
        let profile = serde_json::from_reader::<_, Profile>(zip.by_name("profile.json")?)?;

        let mut options = zip.by_name("options.json");
        if options.is_err() {
            drop(options);
            options = zip.by_name("slicer.json");
        }

        let options = serde_json::from_reader::<_, Options>(options?)?;

        let preview = decode_png(&read_to_bytes(zip.by_name("3d.png")?)?)?;
        let layers = (0..layer_info.len())
            .map(|i| read_to_bytes(zip.by_name(&format!("{}.png", i + 1))?))
            .collect::<Result<Vec<_>>>()?;

        Ok(File {
            meta,
            plate,
            options,
            profile,
            preview,

            layer_info,
            layers,
        })
    }
}
