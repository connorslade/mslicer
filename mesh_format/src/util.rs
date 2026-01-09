use anyhow::Result;
use common::{progress::Progress, serde::Deserializer};

pub const WHITESPACE: [char; 4] = [' ', '\t', '\r', '\n'];

pub fn tokenize<T: Deserializer>(
    des: &mut T,
    delimiter: &[char],
    progress: Progress,
    mut callback: impl FnMut(&str) -> Result<()>,
) -> Result<()> {
    let mut complete = 0;
    let mut carry = String::new();
    loop {
        let next = des.read_bytes(8 * 1024);
        if next.is_empty() && carry.is_empty() {
            break;
        }

        complete += next.len() as u64;
        progress.set_complete(complete);

        let str = carry + str::from_utf8(&next).unwrap();
        let (str, new_carry) = str.rsplit_once(delimiter).unwrap_or(("", &str));
        carry = new_carry.to_owned();

        for token in str.split(delimiter).filter(|x| !x.is_empty()) {
            callback(token)?;
        }
    }

    Ok(())
}
