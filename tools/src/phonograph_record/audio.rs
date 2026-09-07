use std::io::BufRead;

use anyhow::{Ok, Result};
use hound::{SampleFormat, WavReader};

pub struct AudioBuffer {
    audio: AudioData,
    sample_rate: u32,
}

pub enum AudioData {
    Mono(Vec<f32>),
    Stereo(Vec<(f32, f32)>),
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Equalization {
    #[default]
    None,
    Riaa,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    Mono,
    #[default]
    Stereo,
}

impl AudioBuffer {
    pub fn load(reader: impl BufRead, channels: Channels) -> Result<Self> {
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

        if matches!(channels, Channels::Mono) || spec.channels < 2 {
            let audio = audio
                .chunks_exact(spec.channels as usize)
                .map(|s| s.iter().sum())
                .collect();
            Ok(Self {
                audio: AudioData::Mono(audio),
                sample_rate: spec.sample_rate,
            })
        } else {
            let audio = audio
                .chunks_exact(spec.channels as usize)
                .map(|s| (s[0], s[1]))
                .collect();
            Ok(Self {
                audio: AudioData::Stereo(audio),
                sample_rate: spec.sample_rate,
            })
        }
    }

    pub fn apply_equalization(&mut self, equalization: Equalization) {
        match equalization {
            Equalization::None => {}
            Equalization::Riaa => todo!(),
        }
    }

    pub fn get(&self, t: f32) -> (f32, f32) {
        let sample = (t * self.sample_rate as f32).round() as usize;
        match &self.audio {
            AudioData::Mono(items) => items.get(sample).copied().map(|s| (s, s)),
            AudioData::Stereo(items) => items.get(sample).copied(),
        }
        .unwrap_or((0.0, 0.0))
    }
}

impl Equalization {
    pub const ALL: [Self; 2] = [Self::None, Self::Riaa];

    pub fn name(&self) -> &str {
        match self {
            Equalization::None => "None",
            Equalization::Riaa => "RIAA",
        }
    }
}

impl Channels {
    pub const ALL: [Self; 2] = [Self::Mono, Self::Stereo];

    pub fn name(&self) -> &str {
        match self {
            Channels::Mono => "Mono",
            Channels::Stereo => "Stereo",
        }
    }
}
