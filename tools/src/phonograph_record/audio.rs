use std::io::BufRead;

use anyhow::Result;
use hound::{SampleFormat, WavReader};
use itertools::Itertools;

pub struct AudioBuffer {
    samples: Vec<(f32, f32)>,
    sample_rate: u32,
}

impl AudioBuffer {
    pub fn load(reader: impl BufRead) -> Result<Self> {
        let mut audio = WavReader::new(reader)?;
        let spec = audio.spec();

        let audio = match spec.sample_format {
            SampleFormat::Float => audio
                .samples::<f32>()
                .collect::<Result<Vec<_>, hound::Error>>(),
            SampleFormat::Int => {
                let denominator = (1u32 << (spec.bits_per_sample - 1)) as f32;
                audio
                    .samples::<i32>()
                    .map(|x| x.map(|x| x as f32 / denominator))
                    .collect::<Result<Vec<_>, hound::Error>>()
            }
        }?;

        assert_eq!(spec.channels, 2);

        Ok(Self {
            samples: audio.into_iter().tuples().collect(),
            sample_rate: spec.sample_rate,
        })
    }

    pub fn get(&self, t: f32) -> (f32, f32) {
        let sample = (t * self.sample_rate as f32).round() as usize;
        self.samples.get(sample).copied().unwrap_or((0.0, 0.0))
    }
}
