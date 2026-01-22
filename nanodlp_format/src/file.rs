use std::io::{Cursor, Read, Seek, Write};

use anyhow::{Ok, Result};
use common::{misc::SliceResult, serde::Serializer};
use image::DynamicImage;
use serde::Serialize;
use zip::{ZipArchive, ZipWriter, write::FileOptions};

use crate::{
    Layer, decode_png, encode_png, read_to_bytes,
    types::{LayerInfo, Meta, Plate, Profile, Slicer},
};

pub struct File {
    pub meta: Meta,
    pub plate: Plate,
    pub slicer: Slicer,
    pub profile: Profile,
    pub preview: DynamicImage,

    pub layer_info: Vec<LayerInfo>,
    pub layers: Vec<Vec<u8>>, // layers encoded as png
}

impl File {
    pub fn from_slice_result(result: SliceResult<Layer>) -> Self {
        let (layers, layer_info) = result.layers.into_iter().map(|x| (x.inner, x.info)).unzip();

        Self {
            meta: Default::default(),
            plate: Default::default(),
            slicer: Slicer {
                p_width: result.slice_config.platform_resolution.x,
                p_height: result.slice_config.platform_resolution.y,
                ..Default::default()
            },
            profile: Default::default(),
            preview: Default::default(),

            layer_info,
            layers,
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) -> Result<()> {
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
        serialize_file(&mut zip, "slicer.json", &self.slicer)?;
        serialize_file(&mut zip, "profile.json", &self.profile)?;

        zip.start_file("3d.png", FileOptions::DEFAULT)?;
        zip.write_all(&encode_png(&self.preview)?)?;

        for (i, layer) in self.layers.iter().enumerate() {
            zip.start_file(format!("{}.png", i + 1), FileOptions::DEFAULT)?;
            zip.write_all(layer)?;
        }

        drop(zip);
        ser.write_bytes(&bytes);
        Ok(())
    }

    pub fn deserialize<T: Read + Seek>(reader: T) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let meta = serde_json::from_reader::<_, Meta>(zip.by_name("meta.json")?)?;
        let layer_info = serde_json::from_reader::<_, Vec<LayerInfo>>(zip.by_name("info.json")?)?;
        let plate = serde_json::from_reader::<_, Plate>(zip.by_name("plate.json")?)?;
        let slicer = serde_json::from_reader::<_, Slicer>(zip.by_name("slicer.json")?)?;
        let profile = serde_json::from_reader::<_, Profile>(zip.by_name("profile.json")?)?;

        let preview = decode_png(&read_to_bytes(zip.by_name("3d.png")?)?)?;
        let layers = (0..layer_info.len())
            .map(|i| read_to_bytes(zip.by_name(&format!("{}.png", i + 1))?))
            .collect::<Result<Vec<_>>>()?;

        Ok(File {
            meta,
            plate,
            slicer,
            profile,
            preview,

            layer_info,
            layers,
        })
    }
}
