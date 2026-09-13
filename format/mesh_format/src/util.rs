use anyhow::Result;
use common::{progress::Progress, serde::Deserializer};

pub const WHITESPACE: [char; 4] = [' ', '\t', '\r', '\n'];

pub fn tokenize<T: Deserializer>(
    des: &mut T,
    delimiter: &[char],
    progress: &Progress,
    mut callback: impl FnMut(&str) -> Result<()>,
) -> Result<()> {
    let size = des.size();
    let mut complete = 0;
    let mut carry = String::new();

    while complete < size {
        let next = des.read_bytes(8 * 1024);
        if next.is_empty() && carry.is_empty() {
            break;
        }

        complete += next.len();
        progress.set_complete(complete as u64);

        let str = carry + &String::from_utf8_lossy(&next);
        let (str, new_carry) = str.rsplit_once(delimiter).unwrap_or(("", &str));
        carry = new_carry.to_owned();

        for token in str.split(delimiter).filter(|x| !x.is_empty()) {
            callback(token)?;
        }
    }

    (!carry.is_empty()).then(|| callback(&carry));
    Ok(())
}
