use std::io::{Read, Seek};

use anyhow::{Ok, Result};
use image::DynamicImage;
use zip::ZipArchive;

use crate::{
    decode_png, read_to_bytes,
    types::{LayerInfo, Meta, Plate, Profile, Slicer},
};

pub struct File {
    pub meta: Meta,
    pub plate: Plate,
    pub slicer: Slicer,
    pub profile: Profile,
    pub preview: DynamicImage,

    pub layer_info: Vec<LayerInfo>,
    layers: Vec<Vec<u8>>, // layers encoded as png
}

impl File {
    pub fn deseralize<T: Read + Seek>(reader: T) -> Result<Self> {
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
