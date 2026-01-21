use std::fs;

use anyhow::Result;

use nanodlp_format::file;

fn main() -> Result<()> {
    let reader = fs::File::open("/home/connorslade/Downloads/dragon.nanodlp")?;
    let _file = file::File::deseralize(reader)?;

    Ok(())
}
