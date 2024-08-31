use crate::misc::Run;

pub struct RleImage {
    runs: Vec<ImageRun>,
    width: usize,
    height: usize,
}

pub struct RleImageIterator<'a> {
    image: &'a RleImage,
    run_index: usize,
    last_idx: usize,
}

pub struct ImageRun {
    color: u8,

    row: usize,
    start: usize,
    end: usize,
}

impl RleImage {
    pub fn from_decoder(width: usize, height: usize, decoder: impl Iterator<Item = Run>) -> Self {
        let mut runs = Vec::new();

        let mut pixel = 0;
        for run in decoder {
            let length = run.length as usize;
            if run.value != 0 {
                let (y, x) = (pixel / width, pixel % width);
                runs.push(ImageRun {
                    color: run.value,
                    row: y,
                    start: x,
                    end: x + length,
                });
            }
            pixel += length;
        }

        Self {
            runs,
            width,
            height,
        }
    }

    pub fn to_runs(&self) -> RleImageIterator {
        RleImageIterator {
            image: self,
            run_index: 0,
            last_idx: 0,
        }
    }
}

impl<'a> Iterator for RleImageIterator<'a> {
    type Item = Run;

    fn next(&mut self) -> Option<Self::Item> {
        if self.run_index >= self.image.runs.len() {
            return None;
        }

        let run = &self.image.runs[self.run_index];
        let row_offset = run.row * self.image.width;
        let start_idx = row_offset + run.start;

        if self.last_idx < start_idx {
            let length = start_idx - self.last_idx;
            self.last_idx = start_idx;
            return Some(Run {
                length: length as u64,
                value: 0,
            });
        }

        let length = run.end - run.start;
        self.last_idx = row_offset + run.end;
        self.run_index += 1;

        Some(Run {
            length: length as u64,
            value: run.color,
        })
    }
}
