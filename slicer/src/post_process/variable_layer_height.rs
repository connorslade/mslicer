use common::{
    progress::Progress,
    serde::{Deserializer, Serializer},
    slice::{Layer, SliceConfig},
    units::{Second, Seconds},
};
use tracing::info;

#[derive(Clone)]
pub struct VariableLayerHeight {
    pub enabled: bool,

    /// Mex number of layers to be merged
    pub max_layers: u8,
    /// Exposure time added per layer (sec)
    pub exposure: Seconds,
}

impl VariableLayerHeight {
    pub fn post_slice(&self, _config: &SliceConfig, layers: &mut Vec<Layer>, progress: Progress) {
        if !self.enabled {
            return;
        }

        progress.set_total(layers.len() as _);

        let mut i = 0;
        let mut merged = 0;
        let mut counter = 0;

        while i < layers.len() - 1 {
            // If next layer is identical, remove and merge into current
            if merged < self.max_layers && layers[i].data == layers[i + 1].data {
                let old = layers.remove(i + 1);
                let new = &mut layers[i];

                new.exposure.exposure_time = new.exposure.exposure_time + self.exposure;
                new.height = old.height;

                merged += 1;
                counter += 1;
            } else {
                merged = 0;
                i += 1;
            }
        }

        info!("Merged {counter} layers");
        progress.set_finished();
    }
}

impl Default for VariableLayerHeight {
    fn default() -> Self {
        Self {
            enabled: false,
            max_layers: 2,
            exposure: Seconds::new(0.5),
        }
    }
}

impl VariableLayerHeight {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_bool(self.enabled);
        ser.write_u8(self.max_layers);
        ser.write_f32_be(self.exposure.get::<Second>());
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        Self {
            enabled: des.read_bool(),
            max_layers: des.read_u8(),
            exposure: Seconds::new(des.read_f32_be()),
        }
    }
}
