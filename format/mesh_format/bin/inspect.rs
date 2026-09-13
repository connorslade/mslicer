use std::{env, fs::File, io::BufReader, path::Path};

use anyhow::{Context, Ok, Result};
use common::{progress::Progress, serde::ReaderDeserializer};
use mesh_format::Format;

fn main() -> Result<()> {
    let path = env::args().skip(1).next().context("No file specified")?;

    let path = Path::new(&path);
    let format = Format::from_extension(
        &path
            .extension()
            .context("File has no extension")?
            .to_string_lossy(),
    )
    .context("Unknown format")?;

    let des = ReaderDeserializer::new(BufReader::new(File::open(path)?));
    let mesh = mesh_format::load_mesh(des, format, &Progress::new())?;

    println!("{mesh:?}");
    Ok(())
}
