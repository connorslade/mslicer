use common::{container::BitVec, rle::Run};

pub struct Adler32 {
    a: u16,
    b: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LZ77Token {
    Literal(u8),
    Match { length: u16 }, // distance is assumed to be 1
}

pub fn lz77_compress(rle: impl Iterator<Item = Run>) -> Vec<LZ77Token> {
    let mut out = Vec::new();

    for run in rle {
        out.push(LZ77Token::Literal(run.value));
        let mut remaining = run.length.saturating_sub(1);

        while remaining >= 3 {
            let match_len = remaining.min(258);
            remaining -= match_len;
            out.push(LZ77Token::Match {
                length: match_len as u16,
            });
        }

        out.extend((0..remaining).map(|_| LZ77Token::Literal(run.value)));
    }

    out
}

pub fn huffman(out: &mut BitVec, tokens: &[LZ77Token]) {
    out.extend(0x78, 8);
    out.extend(0x01, 8);
    out.extend(0b011, 3);

    for token in tokens {
        match token {
            LZ77Token::Literal(val) => huffman_code(out, *val as u32),
            LZ77Token::Match { length } => {
                let (code, ebits, nbits) = length_code(*length);
                huffman_code(out, code as u32);
                (nbits >= 1).then(|| out.extend(ebits as u32, nbits));

                // For a more general encoder, we would have to get the distance
                // code + extra bits. But since we are only handling Matches
                // with a distance of 1, we can inline the values you would get.
                out.advance(5);
            }
        }
    }

    huffman_code(out, 256);
}

impl Adler32 {
    const MOD: u32 = 65521;

    pub fn new() -> Self {
        Self { a: 1, b: 0 }
    }

    // Efficiently update the checksum state for `length` bytes with value
    // `value`. This is done in batches of 380,368,439 to avoid the possibility
    // of overflows. This constant was derived by finding the largest integers
    // that satisfies the following equation. Note that 65520 is the maximum
    // value of `a` and 255 is the maximum value of `value`.
    //
    // (l * 65520) + (255 * (l * (l + 1) / 2)) < 2^64 - 1
    pub fn update_run(&mut self, run: &Run) {
        let (mut total_length, value) = (run.length, run.value as u64);
        while total_length > 0 {
            let length = total_length.min(380_368_439);
            total_length -= length;

            let (mut a, mut b) = (self.a as u64, self.b as u64);
            b += (length * a) + (value * (length * (length + 1) / 2));
            a += length * value;

            self.a = (a % Self::MOD as u64) as u16;
            self.b = (b % Self::MOD as u64) as u16;
        }
    }

    pub fn finish(self) -> u32 {
        (self.b as u32) << 16 | self.a as u32
    }
}

impl Default for Adler32 {
    fn default() -> Self {
        Self::new()
    }
}

fn huffman_code(vec: &mut BitVec, val: u32) {
    match val {
        0..144 => vec.extend_rev(val + 0x30, 8),
        144..256 => vec.extend_rev(val - 144 + 0x190, 9),
        256..280 => vec.extend_rev(val - 256, 7),
        280..288 => vec.extend_rev(val - 280 + 0xC0, 8),
        _ => unreachable!(),
    }
}

fn length_code(n: u16) -> (u16, u16, u8) {
    match n {
        0..=2 => (n, 0, 0),
        3..=10 => (254 + n, 0, 0),
        11..=18 => (265 + (n - 11) / 2, (n - 11) % 2, 1),
        19..=34 => (269 + (n - 19) / 4, (n - 19) % 4, 2),
        35..=66 => (273 + (n - 35) / 8, (n - 35) % 8, 3),
        67..=130 => (277 + (n - 67) / 16, (n - 67) % 16, 4),
        131..258 => (281 + (n - 131) / 32, (n - 131) % 32, 5),
        258 => (285, 0, 0),
        _ => unreachable!(),
    }
}
