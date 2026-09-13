use std::{
    env,
    fs::{self, File},
    io::BufReader,
};

use anyhow::{Context, Result};
use common::{progress::Progress, serde::ReaderDeserializer, slice::SliceConfig};
use mesh_format::Format;
use rayon::iter::{ParallelBridge, ParallelIterator};
use slicer::{
    mesh::Mesh,
    slicer::{Slicer, SlicerModel},
};

fn main() -> Result<()> {
    let dir = (env::args().skip(1).next()).context("No directory specified.")?;

    fs::read_dir(dir)?
        .par_bridge()
        .filter_map(|e| e.ok())
        .for_each(|entry| {
            if let Some(ext) = entry.path().extension()
                && let Some(format) = Format::from_extension(&ext.to_string_lossy())
            {
                let path = &entry.path();
                let des = ReaderDeserializer::new(BufReader::new(File::open(path).unwrap()));

                let mesh = mesh_format::load_mesh(des, format, &Progress::new()).unwrap();
                let mesh = Mesh::new_boxed(mesh.verts, mesh.faces);

                let name = path.file_name().unwrap().to_string_lossy();
                let detected_defective = mesh.is_defective(Progress::new());

                let models = vec![SlicerModel {
                    mesh,
                    exposure: 255,
                }];
                let slicer = Slicer::new(SliceConfig::default(), models);

                let layers = slicer.slice_raster();
                let defects = layers.iter().map(|x| x.defects).sum::<u64>();

                println!(
                    "{name}\t{}\t{}",
                    ["UNDETECTED", "DETECTED"][detected_defective as usize],
                    defects
                );
            }
        });

    Ok(())
}
