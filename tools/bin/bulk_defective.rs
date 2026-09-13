use std::{
    env,
    fs::{self, File},
    io::BufReader,
};

use anyhow::{Context, Result};
use common::{progress::Progress, serde::ReaderDeserializer};
use mesh_format::Format;
use slicer::mesh::Mesh;

fn main() -> Result<()> {
    let dir = (env::args().skip(1).next()).context("No directory specified.")?;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if let Some(ext) = entry.path().extension()
            && let Some(format) = Format::from_extension(&ext.to_string_lossy())
        {
            let path = &entry.path();
            let des = ReaderDeserializer::new(BufReader::new(File::open(path)?));

            let mesh = mesh_format::load_mesh(des, format, &Progress::new())?;
            let mesh = Mesh::new_boxed(mesh.verts, mesh.faces);

            if mesh.is_defective(Progress::new()) {
                let name = path.file_name().unwrap().to_string_lossy();
                println!("[!] {name} is defective.");
            }
        }
    }

    Ok(())
}
