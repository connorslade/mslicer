use std::fs;

use anyhow::Result;

fn main() -> Result<()> {
    let reader = fs::File::open("/home/connorslade/Downloads/dragon.nanodlp")?;
    let _file = nanodlp_format::File::deserialize(reader)?;

    Ok(())
}
